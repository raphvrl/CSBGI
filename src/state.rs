use std::{sync::Arc, time::Instant};

use anyhow::Result;
use winit::{
    event::{MouseButton, WindowEvent},
    keyboard::KeyCode,
    window::{CursorGrabMode, Window, WindowId},
};

use crate::{
    camera::CameraRig,
    clock::Clock,
    debug::{DebugInput, DebugSnapshot, DebugUi, ProfileStage},
    graphics::{
        GraphicsContext,
        renderer::{RenderFrame, Renderer, RendererResources},
    },
    scene::Scene,
    voxel::gi::VoxelGi,
};

pub struct State {
    graphics: GraphicsContext,
    camera: CameraRig,
    scene: Scene,
    voxel: VoxelGi,
    renderer: Renderer,
    clock: Clock,
    debug: DebugUi,
    window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let startup = Instant::now();
        let graphics = GraphicsContext::new(&window).await?;
        tracing::info!(
            stage = "graphics",
            elapsed_ms = startup.elapsed().as_millis()
        );

        let width = graphics.width();
        let height = graphics.height();
        let renderer_resources = RendererResources::new(&graphics, width, height);
        let scene_data = Scene::load_data(&startup)?;

        let camera = CameraRig::new(width, height);
        let voxel = VoxelGi::new(
            &graphics,
            renderer_resources.frame_layout(),
            &scene_data,
            &startup,
        )?;
        let scene = Scene::new(&graphics, scene_data, &startup)?;
        let renderer = Renderer::new(&graphics, renderer_resources, &scene, &voxel);
        let clock = Clock::new();
        let debug = DebugUi::new(&graphics, window.as_ref());

        Ok(Self {
            graphics,
            camera,
            scene,
            voxel,
            renderer,
            clock,
            debug,
            window,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.graphics.resize(width, height);
        self.renderer.resize(&self.graphics, width, height);
        self.camera.resize(width, height);
    }

    pub fn update(&mut self) {
        self.clock.tick();
        let dt = self.clock.delta_time();
        self.debug.begin_profile_frame(self.clock.frame_time());

        let started = Instant::now();
        self.camera.update(dt);
        self.debug
            .record_cpu_time(ProfileStage::CameraUpdate, started);

        let started = Instant::now();
        self.renderer.update(dt);
        self.debug
            .record_cpu_time(ProfileStage::RendererUpdate, started);
    }

    pub fn prepare(&mut self) {
        let previous_light = self.scene.light_signature();
        let started = Instant::now();
        self.scene.prepare_draws();
        self.debug
            .record_cpu_time(ProfileStage::ScenePreparation, started);

        let (exposure, auto_exposure_enabled, manual_exposure) = self.renderer.debug_settings_mut();
        let (
            voxel_debug_enabled,
            occupied_voxel_debug_enabled,
            voxel_debug_mode,
            mip_debug_enabled,
            mip_level,
            voxel_trace_settings,
        ) = self.voxel.debug_settings_mut();

        self.debug.prepare(
            self.window.as_ref(),
            DebugSnapshot {
                delta_time: self.clock.frame_time(),
                mesh_count: self.scene.mesh_count(),
                draw_count: self.scene.mesh_count(),
                camera_position: self.camera.camera().position,
            },
            DebugInput {
                light: self.scene.light_mut(),
                exposure,
                camera_exposure: self.camera.exposure_settings_mut(),
                voxel_debug_enabled,
                occupied_voxel_debug_enabled,
                voxel_debug_mode,
                mip_debug_enabled,
                mip_level,
                voxel_trace_settings,
                auto_exposure_enabled,
                manual_exposure,
            },
        );

        self.voxel.upload_trace_settings(self.graphics.queue());
        self.voxel
            .mark_radiance_dirty(previous_light != self.scene.light_signature());
        let started = Instant::now();
        self.renderer.prepare(
            &self.graphics,
            &self.scene,
            &self.camera,
            self.clock.delta_time(),
        );
        self.debug
            .record_cpu_time(ProfileStage::RendererPreparation, started);
    }

    pub fn render(&mut self) -> Result<()> {
        self.window.request_redraw();
        self.renderer.render(
            &mut self.graphics,
            RenderFrame {
                scene: &self.scene,
                voxel: &mut self.voxel,
                debug: &mut self.debug,
            },
        )
    }

    pub fn on_key(&mut self, key: KeyCode, pressed: bool) {
        if key == KeyCode::F1 && pressed {
            self.debug.toggle_visibility();
            return;
        }

        if key == KeyCode::Escape && pressed {
            self.release_cursor();
            return;
        }

        self.camera.on_key(key, pressed);
    }

    pub fn on_mouse_motion(&mut self, delta: (f64, f64)) {
        self.debug.on_mouse_motion(delta);
        self.camera.on_mouse_motion(delta);
    }

    pub fn on_window_event(&mut self, event: &WindowEvent) -> bool {
        self.debug.on_window_event(self.window.as_ref(), event)
    }

    pub fn on_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        if button != MouseButton::Right || !pressed {
            return;
        }

        self.capture_cursor();
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
        self.camera.set_active(true);
    }

    fn release_cursor(&mut self) {
        if let Err(error) = self.window.set_cursor_grab(CursorGrabMode::None) {
            tracing::warn!("Unable to release cursor: {error}");
        }

        self.window.set_cursor_visible(true);
        self.camera.set_active(false);
    }

    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }
}
