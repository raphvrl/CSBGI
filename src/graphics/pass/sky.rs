use wesl::include_wesl;

use crate::graphics::{
    GraphicsContext,
    builder::{RenderPassExt, RenderPipelineExt},
    targets::SceneColorTarget,
};

pub struct SkyPass {
    pipeline: wgpu::RenderPipeline,
}

pub struct SkyPassInput<'a> {
    pub color_view: &'a wgpu::TextureView,
    pub frame_bind_group: &'a wgpu::BindGroup,
}

impl SkyPass {
    pub fn new(graphics: &GraphicsContext, frame_layout: &wgpu::BindGroupLayout) -> Self {
        let device = graphics.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sky Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("sky").into()),
        });

        let pipeline = device
            .render_pipeline_builder()
            .label("Sky Pipeline")
            .shader(&shader)
            .bind_group_layout(frame_layout)
            .color_format(SceneColorTarget::FORMAT)
            .blend(wgpu::BlendState::ALPHA_BLENDING)
            .build();

        Self { pipeline }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: SkyPassInput<'_>) {
        let mut render_pass = encoder
            .render_pass_builder()
            .label("Sky Render Pass")
            .color(input.color_view)
            .load()
            .store()
            .build();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, input.frame_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
