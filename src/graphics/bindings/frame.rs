use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use glam::{Mat4, Vec3};

use crate::{
    camera::Camera,
    graphics::{
        GraphicsContext,
        builder::{BindGroupExt, BindGroupLayoutExt},
    },
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct FrameUniforms {
    pub view_projection: Mat4,
    pub camera_position: Vec3,
    pub _pad: f32,
}

pub struct FrameGpu {
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

impl FrameGpu {
    pub fn new(graphics: &GraphicsContext) -> Self {
        let device = graphics.device();

        #[rustfmt::skip]
        let layout = device
            .bind_group_layout_builder()
            .label("Frame Bind Group Layout")
            .entry(0)
                .uniform()
                .vertex()
            .build();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Frame Uniform Buffer"),
            contents: bytemuck::bytes_of(&FrameUniforms::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        #[rustfmt::skip]
        let bind_group = device
            .bind_group_builder()
            .label("Frame Bind Group")
            .layout(&layout)
            .entry(0)
                .buffer(&buffer)
            .build();

        Self {
            layout,
            bind_group,
            buffer,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, camera: &Camera) {
        let view = camera.view_matrix();
        let projection = camera.projection_matrix();
        let view_projection = projection * view;

        let uniforms = FrameUniforms {
            view_projection: view_projection,
            camera_position: camera.position,
            _pad: 0.0,
        };

        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
