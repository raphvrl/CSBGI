use wesl::include_wesl;

use crate::graphics::{
    GraphicsContext,
    builder::{RenderPassExt, RenderPipelineExt},
};

pub struct TrianglePass {
    pipeline: wgpu::RenderPipeline,
}

impl TrianglePass {
    pub fn new(graphics: &GraphicsContext) -> Self {
        let device = graphics.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("triangle").into()),
        });

        let pipeline = device
            .render_pipeline_builder()
            .label("Triangle Pipeline")
            .shader(&shader)
            .color_format(graphics.surface_format())
            .build();

        Self { pipeline }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder
            .render_pass_builder()
            .label("Triangle Render Pass")
            .color(view)
            .clear(wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            })
            .build();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..3, 0..1);
    }
}
