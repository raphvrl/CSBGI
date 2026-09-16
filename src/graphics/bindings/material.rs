use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::graphics::{
    GraphicsContext, Texture,
    builder::{BindGroupExt, BindGroupLayoutExt},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MaterialUniform {
    base_color_factor: [f32; 4],
    normal_scale: [f32; 4],
}

pub struct MaterialLayout {
    layout: wgpu::BindGroupLayout,
}

impl MaterialLayout {
    pub fn new(graphics: &GraphicsContext) -> Self {
        #[rustfmt::skip]
        let layout = graphics
            .device()
            .bind_group_layout_builder()
            .label("Material Bind Group Layout")
            .entry(0)
                .uniform()
                .fragment()
            .entry(1)
                .texture()
                .fragment()
            .entry(2)
                .texture()
                .fragment()
            .entry(3)
                .sampler()
                .fragment()
            .build();

        Self { layout }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
}

pub struct MaterialGpu {
    _buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl MaterialGpu {
    pub fn new(
        graphics: &GraphicsContext,
        layout: &MaterialLayout,
        color_texture: &Texture,
        normal_texture: &Texture,
        base_color_factor: [f32; 4],
        normal_scale: f32,
    ) -> Self {
        let device = graphics.device();
        let uniform = MaterialUniform {
            base_color_factor,
            normal_scale: [normal_scale, 0.0, 0.0, 0.0],
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device
            .bind_group_builder()
            .label("Material Bind Group")
            .layout(layout.layout())
            .entry(0)
            .buffer(&buffer)
            .entry(1)
            .texture(color_texture.view())
            .entry(2)
            .texture(normal_texture.view())
            .entry(3)
            .sampler(color_texture.sampler())
            .build();

        Self {
            _buffer: buffer,
            bind_group,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
