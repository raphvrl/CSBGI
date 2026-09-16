use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::{
    camera::Camera,
    graphics::{
        GraphicsContext,
        builder::{BindGroupExt, BindGroupLayoutExt},
        lighting::DirectionalLight,
    },
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct FrameUniforms {
    pub view_projection: Mat4,
    pub camera_position: Vec3,
    pub _pad0: f32,
    pub light_direction: Vec3,
    pub light_intensity: f32,
    pub light_color: Vec3,
    pub ambient_light: f32,
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
                .fragment()
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

    pub fn update(&self, queue: &wgpu::Queue, camera: &Camera, light: &DirectionalLight) {
        let view = camera.view_matrix();
        let projection = camera.projection_matrix();
        let view_projection = projection * view;

        let uniforms = FrameUniforms {
            view_projection,
            camera_position: camera.position,
            _pad0: 0.0,
            light_direction: light.direction,
            light_intensity: light.intensity,
            light_color: light.color,
            ambient_light: light.ambient,
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
