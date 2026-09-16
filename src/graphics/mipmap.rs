use wesl::include_wesl;

use crate::graphics::{
    Texture,
    builder::{BindGroupExt, BindGroupLayoutExt, RenderPassExt, RenderPipelineExt},
};

pub struct MipmapGenerator {
    color_pipeline: wgpu::RenderPipeline,
    normal_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl MipmapGenerator {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mipmap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("mipmap").into()),
        });

        #[rustfmt::skip]
        let bind_group_layout = device
            .bind_group_layout_builder()
            .label("Mipmap Bind Group Layout")
            .entry(0)
                .texture()
                .fragment()
            .entry(1)
                .sampler()
                .fragment()
            .build();

        let color_pipeline = device
            .render_pipeline_builder()
            .label("Color Mipmap Pipeline")
            .shader(&shader)
            .bind_group_layout(&bind_group_layout)
            .fragment_entry_point("fs_color")
            .color_format(wgpu::TextureFormat::Rgba8UnormSrgb)
            .build();

        let normal_pipeline = device
            .render_pipeline_builder()
            .label("Normal Mipmap Pipeline")
            .shader(&shader)
            .bind_group_layout(&bind_group_layout)
            .fragment_entry_point("fs_normal")
            .color_format(wgpu::TextureFormat::Rgba8Unorm)
            .build();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Mipmap Generation Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            color_pipeline,
            normal_pipeline,
            bind_group_layout,
            sampler,
        }
    }

    pub fn generate(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &Texture,
    ) {
        let format = texture.format();
        let pipeline = match format {
            wgpu::TextureFormat::Rgba8UnormSrgb => &self.color_pipeline,
            wgpu::TextureFormat::Rgba8Unorm => &self.normal_pipeline,
            format => panic!("unsupported mipmap texture format {format:?}"),
        };

        for level in 1..texture.mip_level_count() {
            let source_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Mipmap Source View"),
                format: Some(format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: level - 1,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
            });

            let target_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Mipmap Target View"),
                format: Some(format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: level,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
            });

            #[rustfmt::skip]
            let bind_group = device
                .bind_group_builder()
                .label("Mipmap Bind Group")
                .layout(&self.bind_group_layout)
                .entry(0)
                    .texture(&source_view)
                .entry(1)
                    .sampler(&self.sampler)
                .build();

            let mut rpass = encoder
                .render_pass_builder()
                .label("Mipmap Generation Pass")
                .color(&target_view)
                .clear(wgpu::Color::BLACK)
                .store()
                .build();

            rpass.set_pipeline(pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }
    }
}
