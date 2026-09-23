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
    pbr_factors: [f32; 4],
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
                .texture()
                .fragment()
            .entry(4)
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

pub struct MaterialParameters {
    pub base_color_factor: [f32; 4],
    pub normal_scale: f32,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub occlusion_factor: f32,
}

impl MaterialGpu {
    pub fn new(
        graphics: &GraphicsContext,
        layout: &MaterialLayout,
        color_texture: &Texture,
        normal_texture: &Texture,
        orm_texture: &Texture,
        parameters: MaterialParameters,
    ) -> Self {
        let device = graphics.device();
        let uniform = MaterialUniform {
            base_color_factor: parameters.base_color_factor,
            normal_scale: [parameters.normal_scale, 0.0, 0.0, 0.0],
            pbr_factors: [
                parameters.metallic_factor,
                parameters.roughness_factor,
                parameters.occlusion_factor,
                0.0,
            ],
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
            .buffer(0, &buffer)
            .texture(1, color_texture.view())
            .texture(2, normal_texture.view())
            .texture(3, orm_texture.view())
            .sampler(4, color_texture.sampler())
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
