use anyhow::Result;
use winit::window::{Window, WindowId};

use std::sync::Arc;

use crate::graphics::{context::GraphicsContext, pass::MeshPass};

pub struct State {
    graphics: GraphicsContext,

    triangle_pass: MeshPass,

    window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let graphics = GraphicsContext::new(&window).await?;
        let triangle_pass = MeshPass::new(&graphics);

        Ok(Self {
            window,
            graphics,
            triangle_pass,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.graphics.resize(width, height);
    }

    pub fn update(&mut self) {
        // tmp: update logic here
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

        self.triangle_pass.record(&mut encoder, &view);

        queue.submit(std::iter::once(encoder.finish()));
        queue.present(output);

        Ok(())
    }

    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }
}
