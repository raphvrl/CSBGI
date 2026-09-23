#[derive(Clone, Copy, Default)]
pub struct DebugSnapshot {
    pub delta_time: f32,
    pub mesh_count: usize,
    pub draw_count: usize,
    pub camera_position: glam::Vec3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoxelDebugMode {
    #[default]
    WhiteEdges,
    AlbedoFaces,
    NormalFaces,
    RadianceFaces,
}
