use bytemuck::{Pod, Zeroable};

use crate::voxel::{OCCUPANCY_WORDS, VOXELS_PER_BRICK};

pub const EMPTY_BRICK_INDEX: u32 = u32::MAX;
pub const VOXEL_MIP_LEVEL_COUNT: usize = 8;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuBrick {
    pub key: [u32; 4],
    pub occupancy: [u32; OCCUPANCY_WORDS],
    pub albedo: [u32; VOXELS_PER_BRICK],
    pub normal: [u32; VOXELS_PER_BRICK],
    pub radiance_index: [u32; VOXELS_PER_BRICK],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuVoxel {
    pub brick_index_and_voxel_index: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuVoxelFace {
    pub brick_index_and_voxel_index_and_face: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuVoxelRadiance {
    pub value: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuVoxelMipLevel {
    pub offset_count_dimensions: [u32; 4],
    pub dimensions_and_padding: [u32; 4],
    pub cell_size_and_padding: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuVoxelMipInfo {
    pub levels: [GpuVoxelMipLevel; VOXEL_MIP_LEVEL_COUNT],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuVoxelGrid {
    pub origin_and_voxel_size: [f32; 4],
    pub dimensions_and_brick_size: [u32; 4],
    pub brick_dimensions_and_padding: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
pub struct GpuVoxelTraceSettings {
    pub cone_count: u32,
    pub _padding0: [u32; 3],
    pub max_distance: f32,
    pub cone_angle: f32,
    pub intensity: f32,
    pub _padding1: f32,
}

impl Default for GpuVoxelTraceSettings {
    fn default() -> Self {
        Self {
            cone_count: 5,
            _padding0: [0; 3],
            max_distance: 100.0,
            cone_angle: 1.0471976,
            intensity: 1.0,
            _padding1: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::GpuVoxelMipLevel;

    #[test]
    fn mip_layout_matches_wgsl_storage_stride() {
        assert_eq!(size_of::<GpuVoxelMipLevel>(), 48);
    }
}
