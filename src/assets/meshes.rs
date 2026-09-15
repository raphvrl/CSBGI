use crate::graphics::Mesh;

use super::handles::MeshHandle;

#[derive(Default)]
pub struct MeshAssets {
    meshes: Vec<Mesh>,
}

impl MeshAssets {
    pub fn add(&mut self, mesh: Mesh) -> MeshHandle {
        let handle = MeshHandle::new(self.meshes.len());
        self.meshes.push(mesh);
        handle
    }

    pub fn get(&self, handle: MeshHandle) -> Option<&Mesh> {
        self.meshes.get(handle.index())
    }
}
