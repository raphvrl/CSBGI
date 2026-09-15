use glam::Mat4;

#[derive(Clone, Copy, Debug)]
pub struct VertexData {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Debug)]
pub struct MeshData {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

#[derive(Debug)]
pub struct NodeData {
    pub transform: Mat4,
    pub meshes: Vec<usize>,
    pub children: Vec<usize>,
}

#[derive(Default, Debug)]
pub struct SceneData {
    pub meshes: Vec<MeshData>,
    pub nodes: Vec<NodeData>,
    pub roots: Vec<usize>,
}
