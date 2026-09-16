mod data;
mod panels;

use egui::{ClippedPrimitive, Context, TexturesDelta, ViewportId};
use egui_winit::State as EguiWinitState;
use winit::{event::WindowEvent, window::Window};

use crate::graphics::{
    GraphicsContext,
    lighting::DirectionalLight,
    pass::egui::{EguiPass, EguiPassInput},
};

pub use data::DebugSnapshot;

pub struct DebugUi {
    context: Context,
    winit: EguiWinitState,
    pass: EguiPass,

    paint_jobs: Vec<ClippedPrimitive>,
    textures_delta: TexturesDelta,
    pixels_per_point: f32,
    visible: bool,

    displayed_snapshot: DebugSnapshot,
    snapshot_timer: f32,
}

impl DebugUi {
    pub fn new(graphics: &GraphicsContext, window: &Window) -> Self {
        let context = Context::default();
        let winit = EguiWinitState::new(
            context.clone(),
            ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        Self {
            context,
            winit,
            pass: EguiPass::new(graphics),
            paint_jobs: Vec::new(),
            textures_delta: TexturesDelta::default(),
            pixels_per_point: window.scale_factor() as f32,
            visible: true,
            displayed_snapshot: DebugSnapshot::default(),
            snapshot_timer: 0.0,
        }
    }

    pub fn on_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.winit.on_window_event(window, event).consumed
    }

    pub fn on_mouse_motion(&mut self, delta: (f64, f64)) {
        self.winit.on_mouse_motion(delta);
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn prepare(
        &mut self,
        window: &Window,
        snapshot: DebugSnapshot,
        light: &mut DirectionalLight,
    ) {
        self.snapshot_timer += snapshot.delta_time;

        if self.snapshot_timer >= 0.1 || self.displayed_snapshot.delta_time == 0.0 {
            self.displayed_snapshot = snapshot;
            self.snapshot_timer = 0.0;
        }

        let input = self.winit.take_egui_input(window);
        let context = self.context.clone();
        let visible = self.visible;
        let displayed_snapshot = self.displayed_snapshot;

        let output = context.run_ui(input, |ui| {
            panels::show(ui, visible, &displayed_snapshot, light);
        });

        self.winit
            .handle_platform_output(window, output.platform_output);

        self.pixels_per_point = output.pixels_per_point;
        self.paint_jobs = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);
        self.textures_delta.clear();
        self.textures_delta = output.textures_delta;
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        size_in_pixels: [u32; 2],
    ) {
        self.pass.record(EguiPassInput {
            device,
            queue,
            encoder,
            color_view,
            size_in_pixels,
            pixels_per_point: self.pixels_per_point,
            paint_jobs: &self.paint_jobs,
            textures_delta: &mut self.textures_delta,
        });
    }
}
