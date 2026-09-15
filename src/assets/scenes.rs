use glam::Vec3;

use crate::{
    assets::{
        data::{NodeData, SceneData},
        handles::MeshHandle,
        meshes::MeshAssets,
    },
    graphics::{Mesh, Vertex},
};

pub struct SceneAsset {
    pub meshes: Vec<MeshHandle>,
    pub nodes: Vec<NodeData>,
    pub roots: Vec<usize>,
}

pub fn upload_scene(
    device: &wgpu::Device,
    mesh_assets: &mut MeshAssets,
    scene_data: SceneData,
) -> SceneAsset {
    let meshes = scene_data
        .meshes
        .into_iter()
        .map(|mesh_data| {
            let vertices = mesh_data
                .vertices
                .into_iter()
                .map(|vertex| Vertex {
                    position: Vec3::from_array(vertex.position),
                    normal: Vec3::from_array(vertex.normal),
                })
                .collect::<Vec<_>>();

            let mesh = Mesh::new(device, &vertices, &mesh_data.indices);

            mesh_assets.add(mesh)
        })
        .collect();

    SceneAsset {
        meshes,
        nodes: scene_data.nodes,
        roots: scene_data.roots,
    }
}
