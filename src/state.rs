use anyhow::Result;
use winit::{
    event::MouseButton,
    keyboard::KeyCode,
    window::{CursorGrabMode, Window, WindowId},
};

use std::sync::Arc;

use crate::camera::{Camera, CameraController, Projection};
use crate::clock::Clock;
use crate::graphics::{bindings::FrameGpu, context::GraphicsContext, pass::MeshPass};

pub struct State {
    graphics: GraphicsContext,

    camera: Camera,
    camera_ctrl: CameraController,
    clock: Clock,

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
        let camera_ctrl = CameraController::new(5.0, 0.003);
        let clock = Clock::new();

        let frame_gpu = FrameGpu::new(&graphics);

        let mesh_pass = MeshPass::new(&graphics, frame_gpu.layout());

        Ok(Self {
            graphics,

            camera,
            camera_ctrl,
            clock,

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
        self.clock.tick();
        let dt = self.clock.delta_time();

        self.camera_ctrl.update(&mut self.camera, dt);
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

    pub fn on_key(&mut self, key: KeyCode, pressed: bool) {
        if key == KeyCode::Escape && pressed {
            self.release_cursor();
            return;
        }

        self.camera_ctrl.on_key(key, pressed);
    }

    pub fn on_mouse_motion(&mut self, delta: (f64, f64)) {
        self.camera_ctrl.on_mouse_motion(delta);
    }

    pub fn on_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        if button != MouseButton::Right {
            return;
        }

        if pressed {
            self.capture_cursor();
        } else {
            self.release_cursor();
        }
    }

    pub fn on_focus_changed(&mut self, focused: bool) {
        if !focused {
            self.release_cursor();
        }
    }

    fn capture_cursor(&mut self) {
        let result = self
            .window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined));

        if let Err(error) = result {
            tracing::warn!("Unable to grab cursor: {error}");
            return;
        }

        self.window.set_cursor_visible(false);
        self.camera_ctrl.set_active(true);
    }

    fn release_cursor(&mut self) {
        if let Err(error) = self.window.set_cursor_grab(CursorGrabMode::None) {
            tracing::warn!("Unable to release cursor: {error}");
        }

        self.window.set_cursor_visible(true);
        self.camera_ctrl.set_active(false);
    }

    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }
}
