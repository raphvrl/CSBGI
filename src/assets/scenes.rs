use glam::{Vec2, Vec3, Vec4};
use rayon::prelude::*;

use crate::{
    assets::{
        data::{MeshData, NodeData, SceneData},
        handles::{MaterialHandle, MeshHandle},
        materials::MaterialAssets,
        meshes::MeshAssets,
    },
    graphics::{Mesh, MeshletData, MeshletDescriptor, Vertex},
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
    meshlets: MeshletData,
}

fn build_meshlet_data(mesh: &MeshData) -> anyhow::Result<MeshletData> {
    if !mesh.indices.len().is_multiple_of(3) {
        anyhow::bail!(
            "mesh index count {} is not divisible by 3",
            mesh.indices.len()
        );
    }

    if mesh.indices.is_empty() {
        return Ok(MeshletData::default());
    }

    if mesh
        .indices
        .iter()
        .any(|&index| index as usize >= mesh.vertices.len())
    {
        anyhow::bail!("mesh contains an index outside its vertex array");
    }

    let positions = mesh
        .vertices
        .iter()
        .map(|vertex| vertex.position)
        .collect::<Vec<[f32; 3]>>();

    let adapter = meshopt::utilities::VertexDataAdapter::new(
        bytemuck::cast_slice(&positions),
        std::mem::size_of::<[f32; 3]>(),
        0,
    )?;

    let generated = meshopt::clusterize::build_meshlets(&mesh.indices, &adapter, 64, 124, 0.0);

    Ok(MeshletData {
        descriptors: generated
            .meshlets
            .iter()
            .enumerate()
            .map(|(meshlet_index, meshlet)| {
                let bounds = meshopt::clusterize::compute_meshlet_bounds(
                    generated.get(meshlet_index),
                    &adapter,
                );

                MeshletDescriptor {
                    vertex_offset: meshlet.vertex_offset,
                    triangle_offset: meshlet.triangle_offset,
                    vertex_count: meshlet.vertex_count,
                    triangle_count: meshlet.triangle_count,
                    bounds_center: bounds.center,
                    bounds_radius: bounds.radius,
                }
            })
            .collect(),
        vertex_indices: generated.vertices,
        triangle_indices: generated.triangles,
    })
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
            let meshlets = build_meshlet_data(&mesh_data)?;

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
                meshlets,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut next_meshlet_id = 0;
    let meshes = prepared_meshes
        .into_iter()
        .map(|prepared| {
            let first_meshlet_id = next_meshlet_id;
            next_meshlet_id += prepared.meshlets.descriptors.len();

            let mesh = Mesh::new_with_meshlets(
                device,
                &prepared.vertices,
                &prepared.indices,
                prepared.meshlets,
                first_meshlet_id,
            );

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
