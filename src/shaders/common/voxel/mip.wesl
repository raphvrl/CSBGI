struct GpuVoxelMipLevel {
    offset_count_dimensions: vec4<u32>,
    dimensions_and_padding: vec4<u32>,
    cell_size_and_padding: vec4<f32>,
};

struct GpuVoxelMipInfo {
    levels: array<GpuVoxelMipLevel, 8>,
};
