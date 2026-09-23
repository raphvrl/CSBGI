use bytemuck::{Zeroable, bytes_of};
use glam::UVec3;
use wgpu::util::DeviceExt;

use crate::{
    graphics::{GpuVoxelMipInfo, GpuVoxelMipLevel, GraphicsContext, VOXEL_MIP_LEVEL_COUNT},
    voxel::VoxelGrid,
};

// The eight-level texture spans the complete 256^3 grid from 128^3 to 1^3.
pub const BASE_CELL_VOXELS: u32 = 2;
pub const MIP_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub struct MipHierarchy {
    _mip_texture: wgpu::Texture,
    mip_view: wgpu::TextureView,
    mip_level_views: [wgpu::TextureView; VOXEL_MIP_LEVEL_COUNT],
    mip_sampler: wgpu::Sampler,
    info_buffer: wgpu::Buffer,
    info: GpuVoxelMipInfo,
    level_counts: [u32; VOXEL_MIP_LEVEL_COUNT],
}

impl MipHierarchy {
    pub fn new(graphics: &GraphicsContext, grid: &VoxelGrid) -> Self {
        let device = graphics.device();
        let level_dimensions = dimensions(grid.dimensions);
        let level_counts = level_dimensions.map(volume);

        let mut levels = [GpuVoxelMipLevel::zeroed(); VOXEL_MIP_LEVEL_COUNT];
        let mut offset = 0;
        let mut cell_size = grid.voxel_size * BASE_CELL_VOXELS as f32;
        for level in 0..VOXEL_MIP_LEVEL_COUNT {
            let dimensions = level_dimensions[level];
            levels[level] = GpuVoxelMipLevel {
                offset_count_dimensions: [offset, level_counts[level], dimensions.x, dimensions.y],
                dimensions_and_padding: [dimensions.z, 0, 0, 0],
                cell_size_and_padding: [cell_size, 0.0, 0.0, 0.0],
            };
            offset += level_counts[level];
            cell_size *= 2.0;
        }

        let info = GpuVoxelMipInfo { levels };
        let base_dimensions = level_dimensions[0];
        let mip_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Voxel Prefiltered Mip Texture"),
            size: wgpu::Extent3d {
                width: base_dimensions.x,
                height: base_dimensions.y,
                depth_or_array_layers: base_dimensions.z,
            },
            mip_level_count: VOXEL_MIP_LEVEL_COUNT as u32,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: MIP_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let mip_view = mip_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Voxel Prefiltered Mip Texture View"),
            format: Some(MIP_TEXTURE_FORMAT),
            dimension: Some(wgpu::TextureViewDimension::D3),
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(VOXEL_MIP_LEVEL_COUNT as u32),
            base_array_layer: 0,
            array_layer_count: Some(1),
        });
        let mip_level_views = std::array::from_fn(|level| {
            mip_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Voxel Prefiltered Mip Level View"),
                format: Some(MIP_TEXTURE_FORMAT),
                dimension: Some(wgpu::TextureViewDimension::D3),
                usage: Some(wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: level as u32,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
            })
        });
        let mip_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Voxel Prefiltered Mip Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Prefiltered Mip Info Buffer"),
            contents: bytes_of(&info),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        Self {
            _mip_texture: mip_texture,
            mip_view,
            mip_level_views,
            mip_sampler,
            info_buffer,
            info,
            level_counts,
        }
    }

    pub fn mip_view(&self) -> &wgpu::TextureView {
        &self.mip_view
    }

    pub fn mip_level_view(&self, level: usize) -> &wgpu::TextureView {
        &self.mip_level_views[level]
    }

    pub fn mip_sampler(&self) -> &wgpu::Sampler {
        &self.mip_sampler
    }

    pub fn info_buffer(&self) -> &wgpu::Buffer {
        &self.info_buffer
    }

    pub fn level_count(&self, level: u32) -> u32 {
        self.level_counts.get(level as usize).copied().unwrap_or(0)
    }

    pub fn level(&self, level: usize) -> &GpuVoxelMipLevel {
        &self.info.levels[level]
    }
}

fn half_dimensions(dimensions: UVec3) -> UVec3 {
    UVec3::new(
        (dimensions.x / 2).max(1),
        (dimensions.y / 2).max(1),
        (dimensions.z / 2).max(1),
    )
}

fn volume(dimensions: UVec3) -> u32 {
    dimensions.x * dimensions.y * dimensions.z
}

fn dimensions(voxel_dimensions: UVec3) -> [UVec3; VOXEL_MIP_LEVEL_COUNT] {
    let mut dimensions = [UVec3::ONE; VOXEL_MIP_LEVEL_COUNT];
    dimensions[0] = UVec3::new(
        voxel_dimensions.x.div_ceil(BASE_CELL_VOXELS),
        voxel_dimensions.y.div_ceil(BASE_CELL_VOXELS),
        voxel_dimensions.z.div_ceil(BASE_CELL_VOXELS),
    );

    for level in 1..VOXEL_MIP_LEVEL_COUNT {
        dimensions[level] = half_dimensions(dimensions[level - 1]);
    }

    dimensions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_dimensions_round_up_from_voxels() {
        let dimensions = dimensions(UVec3::new(257, 129, 65));

        assert_eq!(dimensions[0], UVec3::new(129, 65, 33));
        assert_eq!(dimensions[1], UVec3::new(64, 32, 16));
        assert_eq!(dimensions[7], UVec3::new(1, 1, 1));
    }
}
