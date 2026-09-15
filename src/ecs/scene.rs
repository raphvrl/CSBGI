use glam::Mat4;
use hecs::World;

use crate::{
    assets::scenes::SceneAsset,
    ecs::components::{MeshRenderer, Transform},
};

pub fn spawn_scene(world: &mut World, scene: &SceneAsset) {
    for &root in &scene.roots {
        spawn_node(world, scene, root, Mat4::IDENTITY);
    }
}

fn spawn_node(world: &mut World, scene: &SceneAsset, node_index: usize, parent_transform: Mat4) {
    let node = &scene.nodes[node_index];
    let world_transform = parent_transform * node.transform;

    for &mesh_index in &node.meshes {
        let primitive = &scene.meshes[mesh_index];

        world.spawn((
            Transform::from_matrix(world_transform),
            MeshRenderer {
                mesh: primitive.mesh,
                material: primitive.material,
            },
        ));
    }

    for &child_index in &node.children {
        spawn_node(world, scene, child_index, world_transform);
    }
}
