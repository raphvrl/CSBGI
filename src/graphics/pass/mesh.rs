use wesl::include_wesl;

use crate::graphics::{
    GraphicsContext, Mesh, Vertex,
    builder::{RenderPassExt, RenderPipelineExt},
};

pub struct MeshPass {
    pipeline: wgpu::RenderPipeline,
    mesh: Mesh,
}

impl MeshPass {
    pub fn new(graphics: &GraphicsContext, frame_layout: &wgpu::BindGroupLayout) -> Self {
        let device = graphics.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("triangle").into()),
        });

        let pipeline = device
            .render_pipeline_builder()
            .label("Triangle Pipeline")
            .shader(&shader)
            .bind_group_layout(frame_layout)
            .vertex_layout(Vertex::layout())
            .color_format(graphics.surface_format())
            .build();

        let mesh = Mesh::cube(device, 0.5);

        Self { pipeline, mesh }
    }

    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        frame_bind_group: &wgpu::BindGroup,
    ) {
        let mut rpass = encoder
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

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, frame_bind_group, &[]);
        self.mesh.draw(&mut rpass);
    }
}
