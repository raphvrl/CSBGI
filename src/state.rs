use anyhow::Result;
use winit::window::{Window, WindowId};

use std::sync::Arc;

use crate::camera::{Camera, Projection};
use crate::graphics::{bindings::FrameGpu, context::GraphicsContext, pass::MeshPass};

pub struct State {
    graphics: GraphicsContext,

    camera: Camera,

    frame_gpu: FrameGpu,

    mesh_pass: MeshPass,

    window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let graphics = GraphicsContext::new(&window).await?;

        let aspect_ratio = graphics.width() as f32 / graphics.height() as f32;

        let projection = Projection::perspective(60.0_f32.to_radians(), aspect_ratio, 0.1, 100.0);

        let position = glam::Vec3::new(0.0, 0.0, 2.0);
        let rotation = glam::Quat::IDENTITY;

        let camera = Camera::new(position, rotation, projection);

        let frame_gpu = FrameGpu::new(&graphics);

        let mesh_pass = MeshPass::new(&graphics, frame_gpu.layout());

        Ok(Self {
            graphics,

            camera,

            frame_gpu,

            mesh_pass,

            window,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.graphics.resize(width, height);
        self.camera.resize(width, height);
    }

    pub fn update(&mut self) {
        self.frame_gpu.update(self.graphics.queue(), &self.camera);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        let Some(output) = self.graphics.acquire_frame()? else {
            return Ok(());
        };

        let device = self.graphics.device();
        let queue = self.graphics.queue();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        self.mesh_pass
            .record(&mut encoder, &view, self.frame_gpu.bind_group());

        queue.submit(std::iter::once(encoder.finish()));
        queue.present(output);

        Ok(())
    }

    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }
}
