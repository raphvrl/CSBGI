use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::{
    camera::Camera,
    graphics::{
        GraphicsContext,
        builder::{BindGroupExt, BindGroupLayoutExt},
        lighting::DirectionalLight,
        lut::Lut,
        targets::{DepthTarget, ShadowTarget},
    },
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CameraUniforms {
    pub view_projection: Mat4,
    pub inverse_view_projection: Mat4,
    pub camera_position: Vec3,

    pub _pad0: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct LightUniforms {
    pub view_projection: Mat4,

    pub light_direction: Vec3,
    pub light_illuminance_lux: f32,

    pub light_color: Vec3,
    pub ambient_illuminance_lux: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct FrameUniforms {
    pub camera: CameraUniforms,
    pub light: LightUniforms,
}

pub struct FrameGpu {
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    lut_layout: wgpu::BindGroupLayout,
    lut_bind_group: wgpu::BindGroup,
    sky_layout: wgpu::BindGroupLayout,
    sky_bind_group: wgpu::BindGroup,
    shadow_layout: wgpu::BindGroupLayout,
    shadow_bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

impl FrameGpu {
    pub fn new(
        graphics: &GraphicsContext,
        shadow_target: &ShadowTarget,
        depth_target: &DepthTarget,
        lut: &Lut,
    ) -> Self {
        let device = graphics.device();

        #[rustfmt::skip]
        let layout = device
            .bind_group_layout_builder()
            .label("Frame Bind Group Layout")
            .entry(0)
                .uniform()
                .vertex()
                .fragment()
            .entry(1)
                .texture()
                .depth()
                .fragment()
            .entry(2)
                .sampler()
                .comparison()
                .fragment()
            .build();

        #[rustfmt::skip]
        let sky_layout = device
            .bind_group_layout_builder()
            .label("Sky Bind Group Layout")
            .entry(0)
                .uniform()
                .vertex()
                .fragment()
            .entry(3)
                .texture_with(
                    wgpu::TextureSampleType::Depth,
                    wgpu::TextureViewDimension::D2,
                    true,
                )
                .fragment()
            .entry(4)
                .texture()
                .fragment()
            .entry(5)
                .sampler()
                .fragment()
            .build();

        #[rustfmt::skip]
        let lut_layout = device
            .bind_group_layout_builder()
            .label("LUT Bind Group Layout")
            .entry(0)
                .uniform()
                .vertex()
                .fragment()
            .build();

        #[rustfmt::skip]
        let shadow_layout = device
            .bind_group_layout_builder()
            .label("Shadow Frame Bind Group Layout")
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
            .entry(1)
                .texture(shadow_target.view())
            .entry(2)
                .sampler(shadow_target.sampler())
            .build();

        #[rustfmt::skip]
        let lut_bind_group = device
            .bind_group_builder()
            .label("LUT Bind Group")
            .layout(&lut_layout)
            .entry(0)
                .buffer(&buffer)
            .build();

        #[rustfmt::skip]
        let sky_bind_group = device
            .bind_group_builder()
            .label("Sky Bind Group")
            .layout(&sky_layout)
            .entry(0)
                .buffer(&buffer)
            .entry(3)
                .texture(depth_target.view())
            .entry(4)
                .texture(lut.view())
            .entry(5)
                .sampler(lut.sampler())
            .build();

        #[rustfmt::skip]
        let shadow_bind_group = device
            .bind_group_builder()
            .label("Shadow Frame Bind Group")
            .layout(&shadow_layout)
            .entry(0)
                .buffer(&buffer)
            .build();

        Self {
            layout,
            bind_group,
            lut_layout,
            lut_bind_group,
            sky_layout,
            sky_bind_group,
            shadow_layout,
            shadow_bind_group,
            buffer,
        }
    }

    pub fn update_depth_target(
        &mut self,
        graphics: &GraphicsContext,
        depth_target: &DepthTarget,
        lut: &Lut,
    ) {
        self.sky_bind_group = graphics
            .device()
            .bind_group_builder()
            .label("Sky Bind Group")
            .layout(&self.sky_layout)
            .entry(0)
            .buffer(&self.buffer)
            .entry(3)
            .texture(depth_target.view())
            .entry(4)
            .texture(lut.view())
            .entry(5)
            .sampler(lut.sampler())
            .build();
    }

    pub fn update(
        &self,
        queue: &wgpu::Queue,
        camera: &Camera,
        light: &DirectionalLight,
        light_view_projection: Mat4,
    ) {
        let view = camera.view_matrix();
        let projection = camera.projection_matrix();
        let view_projection = projection * view;
        let inverse_view_projection = view_projection.inverse();

        let uniforms = FrameUniforms {
            camera: CameraUniforms {
                view_projection,
                inverse_view_projection,
                camera_position: camera.position,
                _pad0: 0.0,
            },
            light: LightUniforms {
                view_projection: light_view_projection,
                light_direction: light.direction,
                light_illuminance_lux: light.illuminance_lux,
                light_color: light.color,
                ambient_illuminance_lux: light.ambient_illuminance_lux,
            },
        };

        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn sky_layout(&self) -> &wgpu::BindGroupLayout {
        &self.sky_layout
    }

    pub fn lut_layout(&self) -> &wgpu::BindGroupLayout {
        &self.lut_layout
    }

    pub fn lut_bind_group(&self) -> &wgpu::BindGroup {
        &self.lut_bind_group
    }

    pub fn sky_bind_group(&self) -> &wgpu::BindGroup {
        &self.sky_bind_group
    }

    pub fn shadow_layout(&self) -> &wgpu::BindGroupLayout {
        &self.shadow_layout
    }

    pub fn shadow_bind_group(&self) -> &wgpu::BindGroup {
        &self.shadow_bind_group
    }
}
