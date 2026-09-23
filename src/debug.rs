mod data;
mod history;
mod panels;

use std::time::Instant;

use egui::{ClippedPrimitive, Context, TexturesDelta, ViewportId};
use egui_winit::State as EguiWinitState;
use winit::{event::WindowEvent, window::Window};

use crate::camera::CameraExposureSettings;
use crate::graphics::{
    GraphicsContext,
    bindings::GpuVoxelTraceSettings,
    lighting::DirectionalLight,
    pass::egui::{EguiPass, EguiPassInput},
    pass::exposure::ExposureSettings,
};

pub use data::{DebugSnapshot, ProfileStage, VoxelDebugMode};
use history::ProfilerHistory;

pub struct DebugInput<'a> {
    pub light: &'a mut DirectionalLight,
    pub exposure: &'a mut ExposureSettings,
    pub camera_exposure: &'a mut CameraExposureSettings,
    pub voxel_debug_enabled: &'a mut bool,
    pub occupied_voxel_debug_enabled: &'a mut bool,
    pub voxel_debug_mode: &'a mut VoxelDebugMode,
    pub mip_debug_enabled: &'a mut bool,
    pub mip_level: &'a mut u32,
    pub voxel_trace_settings: &'a mut GpuVoxelTraceSettings,
    pub auto_exposure_enabled: &'a mut bool,
    pub manual_exposure: &'a mut f32,
    pub meshlet_debug_enabled: &'a mut bool,
}

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
    profiler_history: ProfilerHistory,
    profiler_visible: bool,
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
            profiler_history: ProfilerHistory::default(),
            profiler_visible: false,
        }
    }

    pub fn begin_profile_frame(&mut self, frame_time: f32) {
        self.profiler_history.begin_frame(frame_time);
    }

    pub fn record_cpu_time(&mut self, stage: ProfileStage, started: Instant) {
        self.profiler_history
            .record_cpu(stage, started.elapsed().as_secs_f32() * 1000.0);
    }

    pub fn record_gpu_times(&mut self, values_ms: [f32; ProfileStage::COUNT]) {
        self.profiler_history.set_gpu_sample(values_ms);
    }

    pub fn gpu_timestamps_unavailable(&mut self) {
        self.profiler_history.set_gpu_unavailable();
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

    pub fn prepare(&mut self, window: &Window, snapshot: DebugSnapshot, mut input: DebugInput<'_>) {
        self.snapshot_timer += snapshot.delta_time;

        if self.snapshot_timer >= 0.1 || self.displayed_snapshot.delta_time == 0.0 {
            self.displayed_snapshot = snapshot;
            self.snapshot_timer = 0.0;
        }

        let egui_input = self.winit.take_egui_input(window);
        let context = self.context.clone();
        let visible = self.visible;
        let displayed_snapshot = self.displayed_snapshot;

        let ui_started = Instant::now();
        let profiler = &self.profiler_history;
        let profiler_visible = &mut self.profiler_visible;
        let output = context.run_ui(egui_input, |ui| {
            panels::show(
                ui,
                visible,
                &displayed_snapshot,
                &mut input,
                profiler,
                profiler_visible,
            );
        });
        self.profiler_history.record_cpu(
            ProfileStage::DebugUi,
            ui_started.elapsed().as_secs_f32() * 1000.0,
        );

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
