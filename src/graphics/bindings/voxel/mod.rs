use std::collections::HashMap;

use anyhow::{Result, anyhow};
use bytemuck::Pod;
use rayon::prelude::*;
use wgpu::util::DeviceExt;

use crate::{
    graphics::{
        GraphicsContext,
        builder::{BindGroupExt, BindGroupLayoutExt},
    },
    voxel::{BRICK_SIZE, Brick, SparseVoxelStorage, VoxelGrid},
};

mod types;

pub use types::{
    EMPTY_BRICK_INDEX, GpuBrick, GpuVoxel, GpuVoxelFace, GpuVoxelGrid, GpuVoxelMipInfo, GpuVoxelMipLevel,
    GpuVoxelRadiance, GpuVoxelTraceSettings, VOXEL_MIP_LEVEL_COUNT,
};

const VOXEL_FACES: [([i32; 3], u32); 6] = [
    ([0, 0, -1], 0),
    ([0, 0, 1], 1),
    ([-1, 0, 0], 2),
    ([1, 0, 0], 3),
    ([0, -1, 0], 4),
    ([0, 1, 0], 5),
];

struct GpuVoxelData {
    bricks: Vec<GpuBrick>,
    occupied_voxels: Vec<GpuVoxel>,
    visible_faces: Vec<GpuVoxelFace>,
    brick_lookup: Vec<u32>,
}

impl GpuVoxelData {
    fn from_storage(storage: &SparseVoxelStorage, grid: &VoxelGrid) -> Result<Self> {
        let brick_entries = storage.iter_bricks().collect::<Vec<_>>();
        u32::try_from(brick_entries.len())
            .map_err(|_| anyhow!("voxel brick count exceeds the GPU index limit"))?;

        let mut bricks = brick_entries
            .par_iter()
            .map(|(key, brick)| GpuBrick {
                key: [key.x, key.y, key.z, 0],
                occupancy: brick.occupancy,
                albedo: brick.albedo,
                normal: brick.normal,
                radiance_index: [EMPTY_BRICK_INDEX; crate::voxel::VOXELS_PER_BRICK],
            })
            .collect::<Vec<_>>();

        let brick_indices = brick_entries
            .iter()
            .enumerate()
            .map(|(index, (key, _))| (*key, index))
            .collect::<HashMap<_, _>>();

        let per_brick = brick_entries
            .par_iter()
            .enumerate()
            .map(|(brick_index, (key, brick))| {
                let mut occupied_voxels = Vec::new();
                let mut visible_faces = Vec::new();
                let brick_index = brick_index as u32;

                for voxel_index in brick.occupied_indices() {
                    let voxel_index_u32 = voxel_index as u32;
                    occupied_voxels.push(GpuVoxel {
                        brick_index_and_voxel_index: [brick_index, voxel_index_u32, 0, 0],
                    });

                    if let Some(local) = Brick::local_from_index(voxel_index) {
                        for (delta, face_index) in VOXEL_FACES {
                            let exposed = key
                                .adjacent(local, delta)
                                .and_then(|(neighbor_key, neighbor_local)| {
                                    brick_indices
                                        .get(&neighbor_key)
                                        .map(|&neighbor_index| (neighbor_index, neighbor_local))
                                })
                                .map(|(neighbor_index, neighbor_local)| {
                                    !brick_entries[neighbor_index].1.is_occupied(neighbor_local)
                                })
                                .unwrap_or(true);

                            if exposed {
                                visible_faces.push(GpuVoxelFace {
                                    brick_index_and_voxel_index_and_face: [
                                        brick_index,
                                        voxel_index_u32,
                                        face_index,
                                        0,
                                    ],
                                });
                            }
                        }
                    }
                }

                (occupied_voxels, visible_faces)
            })
            .collect::<Vec<_>>();

        let occupied_voxels = per_brick
            .iter()
            .flat_map(|(voxels, _)| voxels.iter().copied())
            .collect::<Vec<_>>();
        let visible_faces = per_brick
            .iter()
            .flat_map(|(_, faces)| faces.iter().copied())
            .collect::<Vec<_>>();

        let mut radiance_index = 0u32;
        for (brick_index, (voxels, _)) in per_brick.iter().enumerate() {
            for voxel in voxels {
                let voxel_index = voxel.brick_index_and_voxel_index[1] as usize;
                bricks[brick_index].radiance_index[voxel_index] = radiance_index;
                radiance_index = radiance_index
                    .checked_add(1)
                    .ok_or_else(|| anyhow!("voxel radiance count exceeds the GPU index limit"))?;
            }
        }

        let brick_dimensions = grid.brick_dimensions();
        let lookup_size = checked_volume(brick_dimensions)?;
        let mut brick_lookup = vec![EMPTY_BRICK_INDEX; lookup_size];

        for (brick_index, (key, _)) in brick_entries.iter().enumerate() {
            let table_index = grid
                .brick_table_index(key.as_uvec3())
                .ok_or_else(|| anyhow!("voxel brick {key:?} is outside the configured voxel grid"))?;

            brick_lookup[table_index] = u32::try_from(brick_index)
                .map_err(|_| anyhow!("voxel brick index exceeds the GPU index limit"))?;
        }

        Ok(Self {
            bricks,
            occupied_voxels,
            visible_faces,
            brick_lookup,
        })
    }
}

pub struct VoxelGpu {
    brick_buffer: wgpu::Buffer,
    occupied_voxel_buffer: wgpu::Buffer,
    visible_face_buffer: wgpu::Buffer,
    radiance_buffer: wgpu::Buffer,
    lookup_buffer: wgpu::Buffer,
    grid_buffer: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    radiance_layout: wgpu::BindGroupLayout,
    radiance_bind_group: wgpu::BindGroup,
    lighting_layout: wgpu::BindGroupLayout,
    lighting_bind_group: wgpu::BindGroup,
    brick_count: u32,
    occupied_voxel_count: u32,
    visible_face_count: u32,
}

impl VoxelGpu {
    pub fn new(graphics: &GraphicsContext, storage: &SparseVoxelStorage, grid: &VoxelGrid) -> Result<Self> {
        if !grid.is_valid() {
            return Err(anyhow!(
                "voxel GPU resources require a finite grid with a positive voxel size"
            ));
        }

        let device = graphics.device();
        let voxel_data = GpuVoxelData::from_storage(storage, grid)?;

        let brick_count = u32::try_from(voxel_data.bricks.len())
            .map_err(|_| anyhow!("voxel brick count exceeds the GPU index limit"))?;
        let occupied_voxel_count = u32::try_from(voxel_data.occupied_voxels.len())
            .map_err(|_| anyhow!("occupied voxel count exceeds the GPU index limit"))?;
        let visible_face_count = u32::try_from(voxel_data.visible_faces.len())
            .map_err(|_| anyhow!("visible voxel face count exceeds the GPU index limit"))?;

        let brick_buffer = create_storage_buffer(
            device,
            "Voxel Brick Buffer",
            &voxel_data.bricks,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );
        let occupied_voxel_buffer = create_storage_buffer(
            device,
            "Occupied Voxel Buffer",
            &voxel_data.occupied_voxels,
            wgpu::BufferUsages::STORAGE,
        );
        let visible_face_buffer = create_storage_buffer(
            device,
            "Visible Voxel Face Buffer",
            &voxel_data.visible_faces,
            wgpu::BufferUsages::STORAGE,
        );

        let radiance_values = vec![GpuVoxelRadiance { value: [0.0; 4] }; occupied_voxel_count as usize];
        let radiance_buffer = create_storage_buffer(
            device,
            "Voxel Radiance Buffer",
            &radiance_values,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );
        let lookup_buffer = create_storage_buffer(
            device,
            "Voxel Brick Lookup Buffer",
            &voxel_data.brick_lookup,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );

        let brick_dimensions = grid.brick_dimensions();
        let grid_uniform = GpuVoxelGrid {
            origin_and_voxel_size: [grid.origin.x, grid.origin.y, grid.origin.z, grid.voxel_size],
            dimensions_and_brick_size: [
                grid.dimensions.x,
                grid.dimensions.y,
                grid.dimensions.z,
                BRICK_SIZE,
            ],
            brick_dimensions_and_padding: [brick_dimensions.x, brick_dimensions.y, brick_dimensions.z, 0],
        };
        let grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Grid Uniform Buffer"),
            contents: bytemuck::bytes_of(&grid_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = create_voxel_layout(device);
        let bind_group = device
            .bind_group_builder()
            .label("Voxel Bind Group")
            .layout(&layout)
            .buffer(0, &brick_buffer)
            .buffer(1, &grid_buffer)
            .buffer(2, &lookup_buffer)
            .buffer(3, &occupied_voxel_buffer)
            .buffer(4, &visible_face_buffer)
            .build();

        let radiance_layout = device
            .bind_group_layout_builder()
            .label("Voxel Radiance Bind Group Layout")
            .entry(0)
            .storage()
            .read_only()
            .vertex()
            .fragment()
            .build();
        let lighting_layout = device
            .bind_group_layout_builder()
            .label("Voxel Lighting Bind Group Layout")
            .entry(0)
            .storage()
            .compute()
            .build();
        let radiance_bind_group = device
            .bind_group_builder()
            .label("Voxel Radiance Bind Group")
            .layout(&radiance_layout)
            .buffer(0, &radiance_buffer)
            .build();
        let lighting_bind_group = device
            .bind_group_builder()
            .label("Voxel Lighting Bind Group")
            .layout(&lighting_layout)
            .buffer(0, &radiance_buffer)
            .build();

        Ok(Self {
            brick_buffer,
            occupied_voxel_buffer,
            visible_face_buffer,
            radiance_buffer,
            lookup_buffer,
            grid_buffer,
            layout,
            bind_group,
            radiance_layout,
            radiance_bind_group,
            lighting_layout,
            lighting_bind_group,
            brick_count,
            occupied_voxel_count,
            visible_face_count,
        })
    }

    pub fn brick_buffer(&self) -> &wgpu::Buffer {
        &self.brick_buffer
    }

    pub fn occupied_voxel_buffer(&self) -> &wgpu::Buffer {
        &self.occupied_voxel_buffer
    }

    pub fn visible_face_buffer(&self) -> &wgpu::Buffer {
        &self.visible_face_buffer
    }

    pub fn radiance_buffer(&self) -> &wgpu::Buffer {
        &self.radiance_buffer
    }

    pub fn grid_buffer(&self) -> &wgpu::Buffer {
        &self.grid_buffer
    }

    pub fn lookup_buffer(&self) -> &wgpu::Buffer {
        &self.lookup_buffer
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn radiance_layout(&self) -> &wgpu::BindGroupLayout {
        &self.radiance_layout
    }

    pub fn radiance_bind_group(&self) -> &wgpu::BindGroup {
        &self.radiance_bind_group
    }

    pub fn lighting_layout(&self) -> &wgpu::BindGroupLayout {
        &self.lighting_layout
    }

    pub fn lighting_bind_group(&self) -> &wgpu::BindGroup {
        &self.lighting_bind_group
    }

    pub fn brick_count(&self) -> u32 {
        self.brick_count
    }

    pub fn occupied_voxel_count(&self) -> u32 {
        self.occupied_voxel_count
    }

    pub fn visible_face_count(&self) -> u32 {
        self.visible_face_count
    }
}

fn create_voxel_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device
        .bind_group_layout_builder()
        .label("Voxel Bind Group Layout")
        .entry(0)
        .storage()
        .read_only()
        .vertex()
        .compute()
        .entry(1)
        .uniform()
        .vertex()
        .compute()
        .entry(2)
        .storage()
        .read_only()
        .vertex()
        .fragment()
        .compute()
        .entry(3)
        .storage()
        .read_only()
        .vertex()
        .compute()
        .entry(4)
        .storage()
        .read_only()
        .vertex()
        .compute()
        .build()
}

fn create_storage_buffer<T: Pod>(
    device: &wgpu::Device,
    label: &str,
    values: &[T],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    if values.is_empty() {
        return device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: std::mem::size_of::<T>() as u64,
            usage,
            mapped_at_creation: false,
        });
    }

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(values),
        usage,
    })
}

fn checked_volume(dimensions: glam::UVec3) -> Result<usize> {
    let volume = u64::from(dimensions.x)
        .checked_mul(u64::from(dimensions.y))
        .and_then(|volume| volume.checked_mul(u64::from(dimensions.z)))
        .ok_or_else(|| anyhow!("voxel grid dimensions overflow the lookup buffer size"))?;

    usize::try_from(volume).map_err(|_| anyhow!("voxel lookup buffer is too large"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::BrickKey;
    use glam::{UVec3, Vec3};

    fn test_grid() -> VoxelGrid {
        VoxelGrid {
            origin: Vec3::ZERO,
            voxel_size: 1.0,
            dimensions: UVec3::splat(16),
        }
    }

    #[test]
    fn gpu_data_keeps_bricks_in_key_order() {
        let mut storage = SparseVoxelStorage::new();
        storage.set_occupied(BrickKey::new(1, 0, 0), UVec3::ZERO);
        storage.set_occupied(BrickKey::new(0, 0, 0), UVec3::ZERO);

        let data = GpuVoxelData::from_storage(&storage, &test_grid()).unwrap();

        assert_eq!(data.bricks[0].key, [0, 0, 0, 0]);
        assert_eq!(data.bricks[1].key, [1, 0, 0, 0]);
    }

    #[test]
    fn gpu_data_marks_only_external_faces_as_visible() {
        let mut storage = SparseVoxelStorage::new();
        storage.set_occupied(BrickKey::new(0, 0, 0), UVec3::new(7, 0, 0));
        storage.set_occupied(BrickKey::new(1, 0, 0), UVec3::new(0, 0, 0));

        let data = GpuVoxelData::from_storage(&storage, &test_grid()).unwrap();

        assert_eq!(data.occupied_voxels.len(), 2);
        assert_eq!(data.visible_faces.len(), 10);
    }
}
