#[derive(Clone, Copy, Default)]
pub struct DebugSnapshot {
    pub delta_time: f32,
    pub mesh_count: usize,
    pub draw_count: usize,
    pub camera_position: glam::Vec3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(usize)]
pub enum ProfileStage {
    #[default]
    CameraUpdate,
    RendererUpdate,
    ScenePreparation,
    DebugUi,
    RendererPreparation,
    Shadow,
    VoxelLighting,
    Sky,
    DepthPrepass,
    Geometry,
    VoxelDebug,
    Lut,
    Luminance,
    Exposure,
    Tonemapping,
    Egui,
}

impl ProfileStage {
    pub const COUNT: usize = 16;
    pub const RENDER_START: Self = Self::Shadow;

    pub const ALL: [Self; Self::COUNT] = [
        Self::CameraUpdate,
        Self::RendererUpdate,
        Self::ScenePreparation,
        Self::DebugUi,
        Self::RendererPreparation,
        Self::Shadow,
        Self::VoxelLighting,
        Self::Sky,
        Self::DepthPrepass,
        Self::Geometry,
        Self::VoxelDebug,
        Self::Lut,
        Self::Luminance,
        Self::Exposure,
        Self::Tonemapping,
        Self::Egui,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::CameraUpdate => "Camera update",
            Self::RendererUpdate => "Renderer update",
            Self::ScenePreparation => "Scene preparation",
            Self::DebugUi => "Debug UI",
            Self::RendererPreparation => "Renderer preparation",
            Self::Shadow => "Shadow pass",
            Self::VoxelLighting => "Voxel lighting + mipmaps",
            Self::Sky => "Sky pass",
            Self::DepthPrepass => "Depth prepass",
            Self::Geometry => "Geometry pass",
            Self::VoxelDebug => "Voxel debug pass",
            Self::Lut => "Atmosphere LUT",
            Self::Luminance => "Luminance pass",
            Self::Exposure => "Exposure pass",
            Self::Tonemapping => "Tonemapping pass",
            Self::Egui => "egui pass",
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct ProfileSample {
    pub time: f64,
    pub frame_time_ms: f32,
    pub cpu_ms: [f32; ProfileStage::COUNT],
    pub gpu_ms: [f32; ProfileStage::COUNT],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoxelDebugMode {
    #[default]
    WhiteEdges,
    AlbedoFaces,
    NormalFaces,
    RadianceFaces,
}
