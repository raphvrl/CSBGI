use wesl::include_wesl;

use crate::graphics::{
    GraphicsContext,
    bindings::ToneMappingBinding,
    builder::{RenderPassExt, RenderPipelineExt},
};

pub struct ToneMappingPass {
    pipeline: wgpu::RenderPipeline,
    meshlet_debug_pipeline: wgpu::RenderPipeline,
}

pub struct ToneMappingPassInput<'a> {
    pub binding: &'a ToneMappingBinding,
    pub target_view: &'a wgpu::TextureView,
    pub meshlet_debug_enabled: bool,
}

impl ToneMappingPass {
    pub fn new(graphics: &GraphicsContext, binding: &ToneMappingBinding) -> Self {
        let device = graphics.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tone Mapping Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("tonemapping").into()),
        });

        let pipeline = device
            .render_pipeline_builder()
            .label("Tone Mapping Pipeline")
            .shader(&shader)
            .bind_group_layout(binding.layout())
            .color_format(graphics.surface_format())
            .build();

        let meshlet_debug_pipeline = device
            .render_pipeline_builder()
            .label("Meshlet Debug Output Pipeline")
            .shader(&shader)
            .bind_group_layout(binding.layout())
            .fragment_entry_point("fs_debug")
            .color_format(graphics.surface_format())
            .build();

        Self {
            pipeline,
            meshlet_debug_pipeline,
        }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: ToneMappingPassInput<'_>) {
        let mut rpass = encoder
            .render_pass_builder()
            .label("Tone Mapping Render Pass")
            .color(input.target_view)
            .clear(wgpu::Color::BLACK)
            .store()
            .build();

        let pipeline = if input.meshlet_debug_enabled {
            &self.meshlet_debug_pipeline
        } else {
            &self.pipeline
        };
        rpass.set_pipeline(pipeline);
        rpass.set_bind_group(0, input.binding.bind_group(), &[]);
        rpass.draw(0..3, 0..1);
    }
}
