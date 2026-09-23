struct GpuBrick {
    key: vec4<u32>,
    occupancy: array<u32, 16>,
    albedo: array<u32, 512>,
    normal: array<u32, 512>,
    radiance_index: array<u32, 512>,
};

struct GpuVoxel {
    brick_index_and_voxel_index: vec4<u32>,
};

struct GpuVoxelFace {
    brick_index_and_voxel_index_and_face: vec4<u32>,
};

struct GpuVoxelRadiance {
    value: vec4<f32>,
};

struct GpuVoxelGrid {
    origin_and_voxel_size: vec4<f32>,
    dimensions_and_brick_size: vec4<u32>,
    brick_dimensions_and_padding: vec4<u32>,
};

struct GpuVoxelTraceSettings {
    cone_count: u32,
    padding0: u32,
    padding1: u32,
    padding2: u32,
    max_distance: f32,
    cone_angle: f32,
    intensity: f32,
    padding3: f32,
};

struct VoxelSample {
    radiance: vec3<f32>,
    opacity: f32,
};
