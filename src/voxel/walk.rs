use anyhow::{Context, Result, anyhow};
use glam::Mat4;

use crate::assets::data::{MeshData, SceneData};

pub(super) struct MeshInstance<'a> {
    pub mesh: &'a MeshData,
    pub transform: Mat4,
}

pub(super) fn visit_meshes(
    scene: &SceneData,
    mut on_mesh: impl FnMut(MeshInstance<'_>) -> Result<()>,
) -> Result<usize> {
    let mut nodes_visited = 0;

    for &root in &scene.roots {
        nodes_visited += visit_node(scene, root, Mat4::IDENTITY, &mut on_mesh)
            .with_context(|| format!("failed to visit root node {root}"))?;
    }

    Ok(nodes_visited)
}

fn visit_node(
    scene: &SceneData,
    node_index: usize,
    parent_transform: Mat4,
    on_mesh: &mut impl FnMut(MeshInstance<'_>) -> Result<()>,
) -> Result<usize> {
    let node = scene
        .nodes
        .get(node_index)
        .ok_or_else(|| anyhow!("node index {node_index} is out of bounds"))?;

    let world_transform = parent_transform * node.transform;

    for &mesh_index in &node.meshes {
        let mesh = scene
            .meshes
            .get(mesh_index)
            .ok_or_else(|| anyhow!("mesh index {mesh_index} is out of bounds"))?;

        on_mesh(MeshInstance {
            mesh,
            transform: world_transform,
        })
        .with_context(|| format!("failed to process mesh {mesh_index} from node {node_index}"))?;
    }

    let mut nodes_visited = 1;
    for &child in &node.children {
        nodes_visited += visit_node(scene, child, world_transform, on_mesh)
            .with_context(|| format!("failed to visit child node {child} from node {node_index}"))?;
    }

    Ok(nodes_visited)
}
