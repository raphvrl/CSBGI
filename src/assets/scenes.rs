use glam::{Vec2, Vec3, Vec4};
use rayon::prelude::*;

use crate::{
    assets::{
        data::{NodeData, SceneData},
        handles::{MaterialHandle, MeshHandle},
        materials::MaterialAssets,
        meshes::MeshAssets,
    },
    graphics::{Mesh, Vertex},
};

pub struct SceneAsset {
    pub meshes: Vec<PrimitiveAsset>,
    pub nodes: Vec<NodeData>,
    pub roots: Vec<usize>,
}

pub struct PrimitiveAsset {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
}

struct PreparedPrimitive {
    material: MaterialHandle,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

pub fn upload_scene(
    device: &wgpu::Device,
    mesh_assets: &mut MeshAssets,
    material_assets: &MaterialAssets,
    scene_data: SceneData,
) -> anyhow::Result<SceneAsset> {
    let prepared_meshes = scene_data
        .meshes
        .into_par_iter()
        .map(|mesh_data| -> anyhow::Result<PreparedPrimitive> {
            let material = material_assets.resolve(mesh_data.material)?;

            let vertices = mesh_data
                .vertices
                .into_iter()
                .map(|vertex| Vertex {
                    position: Vec3::from_array(vertex.position),
                    normal: Vec3::from_array(vertex.normal),
                    uv: Vec2::from_array(vertex.uv),
                    tangent: Vec4::from_array(vertex.tangent),
                })
                .collect::<Vec<_>>();

            Ok(PreparedPrimitive {
                material,
                vertices,
                indices: mesh_data.indices,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let meshes = prepared_meshes
        .into_iter()
        .map(|prepared| {
            let mesh = Mesh::new(device, &prepared.vertices, &prepared.indices);

            Ok(PrimitiveAsset {
                mesh: mesh_assets.add(mesh),
                material: prepared.material,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(SceneAsset {
        meshes,
        nodes: scene_data.nodes,
        roots: scene_data.roots,
    })
}
