use glam::Mat4;

pub type ImageIndex = usize;
pub type MaterialIndex = usize;

#[derive(Clone, Copy, Debug)]
pub struct VertexData {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

#[derive(Debug)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

#[derive(Debug)]
pub struct MaterialData {
    pub color_factor: [f32; 4],
    pub color_image: Option<ImageIndex>,
    pub normal_image: Option<ImageIndex>,
    pub normal_scale: f32,
}

#[derive(Debug)]
pub struct MeshData {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub material: Option<MaterialIndex>,
}

#[derive(Debug)]
pub struct NodeData {
    pub transform: Mat4,
    pub meshes: Vec<usize>,
    pub children: Vec<usize>,
}

#[derive(Default, Debug)]
pub struct SceneData {
    pub images: Vec<ImageData>,
    pub materials: Vec<MaterialData>,
    pub meshes: Vec<MeshData>,
    pub nodes: Vec<NodeData>,
    pub roots: Vec<usize>,
}
