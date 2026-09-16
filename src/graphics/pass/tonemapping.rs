use wesl::include_wesl;

use crate::graphics::{
    GraphicsContext,
    bindings::ToneMappingBinding,
    builder::{RenderPassExt, RenderPipelineExt},
};

pub struct ToneMappingPass {
    pipeline: wgpu::RenderPipeline,
}

pub struct ToneMappingPassInput<'a> {
    pub binding: &'a ToneMappingBinding,
    pub target_view: &'a wgpu::TextureView,
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

        Self { pipeline }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: ToneMappingPassInput<'_>) {
        let mut rpass = encoder
            .render_pass_builder()
            .label("Tone Mapping Render Pass")
            .color(input.target_view)
            .clear(wgpu::Color::BLACK)
            .store()
            .build();

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, input.binding.bind_group(), &[]);
        rpass.draw(0..3, 0..1);
    }
}
