use wesl::include_wesl;

use crate::graphics::{
    GraphicsContext,
    builder::{RenderPassExt, RenderPipelineExt},
    lut::Lut,
};

pub struct LutPass {
    pipeline: wgpu::RenderPipeline,
}

pub struct LutPassInput<'a> {
    pub lut_view: &'a wgpu::TextureView,
    pub frame_bind_group: &'a wgpu::BindGroup,
}

impl LutPass {
    pub fn new(graphics: &GraphicsContext, frame_layout: &wgpu::BindGroupLayout) -> Self {
        let device = graphics.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("LUT Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("lut").into()),
        });

        let pipeline = device
            .render_pipeline_builder()
            .label("LUT Pipeline")
            .shader(&shader)
            .bind_group_layout(frame_layout)
            .color_format(Lut::FORMAT)
            .build();

        Self { pipeline }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: LutPassInput<'_>) {
        let mut render_pass = encoder
            .render_pass_builder()
            .label("LUT Render Pass")
            .color(input.lut_view)
            .clear(wgpu::Color::BLACK)
            .store()
            .build();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, input.frame_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
