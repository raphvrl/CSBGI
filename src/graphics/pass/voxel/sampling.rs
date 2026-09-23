use wgpu::util::DeviceExt;

use crate::graphics::{
    GraphicsContext, VoxelGpu,
    bindings::GpuVoxelTraceSettings,
    builder::{BindGroupExt, BindGroupLayoutExt},
};

use super::hierarchy::MipHierarchy;

pub struct VoxelSampling {
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    trace_settings_buffer: wgpu::Buffer,
    trace_settings: GpuVoxelTraceSettings,
}

impl VoxelSampling {
    pub fn new(graphics: &GraphicsContext, voxel_gpu: &VoxelGpu, mip_hierarchy: &MipHierarchy) -> Self {
        let device = graphics.device();
        let layout = device
            .bind_group_layout_builder()
            .label("Geometry Voxel Sampling Layout")
            .entry(0)
            .texture()
            .view_dimension(wgpu::TextureViewDimension::D3)
            .fragment()
            .entry(1)
            .sampler()
            .fragment()
            .entry(2)
            .uniform()
            .fragment()
            .entry(3)
            .uniform()
            .fragment()
            .entry(4)
            .uniform()
            .fragment()
            .build();

        let trace_settings = GpuVoxelTraceSettings::default();
        let trace_settings_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Trace Settings Buffer"),
            contents: bytemuck::bytes_of(&trace_settings),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device
            .bind_group_builder()
            .label("Geometry Voxel Sampling Bind Group")
            .layout(&layout)
            .texture(0, mip_hierarchy.mip_view())
            .sampler(1, mip_hierarchy.mip_sampler())
            .buffer(2, mip_hierarchy.info_buffer())
            .buffer(3, voxel_gpu.grid_buffer())
            .buffer(4, &trace_settings_buffer)
            .build();

        Self {
            layout,
            bind_group,
            trace_settings_buffer,
            trace_settings,
        }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn trace_settings_mut(&mut self) -> &mut GpuVoxelTraceSettings {
        &mut self.trace_settings
    }

    pub fn upload_trace_settings(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.trace_settings_buffer,
            0,
            bytemuck::bytes_of(&self.trace_settings),
        );
    }
}
