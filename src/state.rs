use anyhow::Result;
use winit::{
    event::{MouseButton, WindowEvent},
    keyboard::KeyCode,
    window::{CursorGrabMode, Window, WindowId},
};

use std::sync::Arc;

use crate::assets::{AssetLoader, MaterialAssets, MeshAssets, TextureAssets, upload_scene};
use crate::camera::{Camera, CameraController, CameraExposureSettings, Projection};
use crate::clock::Clock;
use crate::debug::{DebugSnapshot, DebugUi};
use crate::ecs::scene::spawn_scene;
use crate::ecs::{MeshRenderer, Transform};
use crate::graphics::{
    bindings::{FrameGpu, ObjectGpu, ToneMappingBinding},
    context::GraphicsContext,
    lighting::{DirectionalLight, ShadowSettings},
    lut::Lut,
    mipmap::MipmapGenerator,
    pass::{
        MeshDraw,
        exposure::ExposurePass,
        geometry::{GeometryPass, GeometryPassInput},
        luminance::{LuminancePass, LuminancePassInput},
        lut::{LutPass, LutPassInput},
        shadow::{ShadowPass, ShadowPassInput},
        sky::{SkyPass, SkyPassInput},
        tonemapping::{ToneMappingPass, ToneMappingPassInput},
    },
    targets::{DepthTarget, LuminanceTarget, SceneColorTarget, ShadowTarget},
};

const LUT_UPDATE_INTERVAL: f32 = 1.0 / 15.0;

struct LutKey {
    camera_height: f32,
    light_direction: glam::Vec3,
    light_color: glam::Vec3,
    light_illuminance_lux: f32,
}

impl LutKey {
    fn new(camera_height: f32, light: &DirectionalLight) -> Self {
        Self {
            camera_height,
            light_direction: light.direction,
            light_color: light.color,
            light_illuminance_lux: light.illuminance_lux,
        }
    }

    fn changed(&self, other: &Self) -> bool {
        const HEIGHT_EPSILON: f32 = 0.01;
        const DIRECTION_EPSILON: f32 = 0.0001;
        const VALUE_EPSILON: f32 = 0.0001;

        (self.camera_height - other.camera_height).abs() > HEIGHT_EPSILON
            || self.light_direction.distance_squared(other.light_direction) > DIRECTION_EPSILON
            || self.light_color.distance_squared(other.light_color) > VALUE_EPSILON
            || (self.light_illuminance_lux - other.light_illuminance_lux).abs() > VALUE_EPSILON
    }
}

pub struct State {
    graphics: GraphicsContext,

    scene_color_target: SceneColorTarget,
    luminance_target: LuminanceTarget,
    depth_target: DepthTarget,
    shadow_target: ShadowTarget,

    camera: Camera,
    camera_exposure: CameraExposureSettings,
    camera_ctrl: CameraController,
    clock: Clock,

    light: DirectionalLight,
    shadow_settings: ShadowSettings,

    frame_gpu: FrameGpu,
    object_gpu: ObjectGpu,

    world: hecs::World,
    mesh_assets: MeshAssets,

    #[allow(dead_code)]
    texture_assets: TextureAssets,
    material_assets: MaterialAssets,

    lut: Lut,
    lut_pass: LutPass,
    lut_key: Option<LutKey>,
    lut_timer: f32,

    mesh_draws: Vec<MeshDraw>,
    shadow_pass: ShadowPass,
    sky_pass: SkyPass,
    geometry_pass: GeometryPass,
    luminance_pass: LuminancePass,
    exposure_pass: ExposurePass,
    tonemapping_binding: ToneMappingBinding,
    tonemapping_pass: ToneMappingPass,
    debug: DebugUi,

    window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let graphics = GraphicsContext::new(&window).await?;

        let width = graphics.width();
        let height = graphics.height();

        let scene_color_target = SceneColorTarget::new(&graphics, width, height);
        let luminance_target = LuminanceTarget::new(&graphics, width, height);
        let depth_target = DepthTarget::new(&graphics, width, height);
        let shadow_target = ShadowTarget::new(&graphics);

        let debug = DebugUi::new(&graphics, window.as_ref());

        let aspect_ratio = graphics.width() as f32 / graphics.height() as f32;

        let projection = Projection::perspective(60.0_f32.to_radians(), aspect_ratio, 0.1, 100.0);

        let position = glam::Vec3::new(0.0, 0.0, 2.0);
        let rotation = glam::Quat::IDENTITY;

        let camera = Camera::new(position, rotation, projection);
        let camera_exposure = CameraExposureSettings::default();
        let camera_ctrl = CameraController::new(5.0, 0.003);
        let clock = Clock::new();

        let light = DirectionalLight::default();

        let lut = Lut::new(graphics.device());

        let frame_gpu = FrameGpu::new(&graphics, &shadow_target, &depth_target, &lut);

        let mut world = hecs::World::new();
        let mut mesh_assets = MeshAssets::default();
        let asset_loader = AssetLoader::default();
        let scene_data = asset_loader.load("assets/models/Sponza/glTF/Sponza.gltf")?;

        let mipmap_generator = MipmapGenerator::new(graphics.device());

        let texture_assets = TextureAssets::from_images(
            &graphics,
            &mipmap_generator,
            &scene_data.images,
            &scene_data.materials,
        )?;

        let lut_pass = LutPass::new(&graphics, frame_gpu.lut_layout());

        let material_assets =
            MaterialAssets::from_data(&graphics, &scene_data.materials, &texture_assets)?;

        let object_gpu = ObjectGpu::new(&graphics);

        let shadow_pass = ShadowPass::new(
            &graphics,
            frame_gpu.shadow_layout(),
            object_gpu.layout(),
            material_assets.layout(),
        );

        let sky_pass = SkyPass::new(&graphics, frame_gpu.sky_layout());

        let geometry_pass = GeometryPass::new(
            &graphics,
            frame_gpu.layout(),
            object_gpu.layout(),
            material_assets.layout(),
        );

        let luminance_pass = LuminancePass::new(
            &graphics,
            scene_color_target.resolve_view(),
            &luminance_target,
        );

        let exposure_pass = ExposurePass::new(&graphics, luminance_target.final_view());

        let tonemapping_binding = ToneMappingBinding::new(
            &graphics,
            scene_color_target.resolve_view(),
            exposure_pass.buffer(),
        );

        let tonemapping_pass = ToneMappingPass::new(&graphics, &tonemapping_binding);

        let scene_asset = upload_scene(
            graphics.device(),
            &mut mesh_assets,
            &material_assets,
            scene_data,
        )?;

        spawn_scene(&mut world, &scene_asset);

        Ok(Self {
            graphics,

            scene_color_target,
            luminance_target,
            depth_target,
            shadow_target,

            camera,
            camera_exposure,
            camera_ctrl,
            clock,

            light,
            shadow_settings: ShadowSettings::default(),

            shadow_pass,
            frame_gpu,
            object_gpu,

            world,
            mesh_assets,

            texture_assets,
            material_assets,

            lut,
            lut_pass,
            lut_key: None,
            lut_timer: 0.0,

            mesh_draws: Vec::new(),
            sky_pass,
            tonemapping_binding,
            geometry_pass,
            luminance_pass,
            exposure_pass,
            tonemapping_pass,
            debug,

            window,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.graphics.resize(width, height);
        self.scene_color_target
            .resize(&self.graphics, width, height);
        self.luminance_target.resize(&self.graphics, width, height);
        self.tonemapping_binding.update_source(
            self.graphics.device(),
            self.scene_color_target.resolve_view(),
            self.exposure_pass.buffer(),
        );
        self.luminance_pass.update_bindings(
            &self.graphics,
            self.scene_color_target.resolve_view(),
            &self.luminance_target,
        );
        self.exposure_pass
            .update_luminance(&self.graphics, self.luminance_target.final_view());
        self.depth_target.resize(&self.graphics, width, height);
        self.frame_gpu
            .update_depth_target(&self.graphics, &self.depth_target, &self.lut);
        self.camera.resize(width, height);
    }

    pub fn update(&mut self) {
        self.clock.tick();
        let dt = self.clock.delta_time();

        self.camera_ctrl.update(&mut self.camera, dt);
        self.lut_timer += dt;
    }

    pub fn prepare(&mut self) {
        self.mesh_draws = self
            .world
            .query::<(&Transform, &MeshRenderer)>()
            .iter()
            .map(|(transform, renderer)| MeshDraw {
                mesh: renderer.mesh,
                material: renderer.material,
                model: transform.matrix(),
            })
            .collect();

        let models = self
            .mesh_draws
            .iter()
            .map(|draw| draw.model)
            .collect::<Vec<_>>();

        self.object_gpu.update(&self.graphics, &models);

        self.debug.prepare(
            self.window.as_ref(),
            DebugSnapshot {
                delta_time: self.clock.delta_time(),
                mesh_count: self.mesh_draws.len(),
                draw_count: self.mesh_draws.len(),
                camera_position: self.camera.position,
            },
            &mut self.light,
            self.exposure_pass.settings_mut(),
            &mut self.camera_exposure,
        );

        self.exposure_pass.update(
            self.graphics.queue(),
            self.clock.delta_time(),
            self.camera_exposure.exposure_stops(),
        );

        let key = LutKey::new(self.camera.position.y, &self.light);

        let initial = self.lut_key.is_none();
        let changed = match self.lut_key.as_ref() {
            Some(previous) => previous.changed(&key),
            None => true,
        };

        if changed && (initial || self.lut_timer >= LUT_UPDATE_INTERVAL) {
            self.lut.invalidate();
            self.lut_key = Some(key);
            self.lut_timer = 0.0;
        }

        let light_view_projection = self.light.shadow_view_projection(&self.shadow_settings);

        self.frame_gpu.update(
            self.graphics.queue(),
            &self.camera,
            &self.light,
            light_view_projection,
        );
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        let Some(output) = self.graphics.acquire_frame()? else {
            return Ok(());
        };

        let device = self.graphics.device();
        let queue = self.graphics.queue();

        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let depth_view = self.depth_target.view();
        let resolve_view = self.scene_color_target.resolve_view();

        let frame_bind_group = self.frame_gpu.bind_group();
        let shadow_frame_bind_group = self.frame_gpu.shadow_bind_group();
        let object_bind_group = self.object_gpu.bind_group();

        self.shadow_pass.record(
            &mut encoder,
            ShadowPassInput {
                shadow_view: self.shadow_target.view(),
                frame_bind_group: shadow_frame_bind_group,
                object_bind_group,
                material_assets: &self.material_assets,
                mesh_draws: &self.mesh_draws,
                mesh_assets: &self.mesh_assets,
            },
        );

        self.sky_pass.record(
            &mut encoder,
            SkyPassInput {
                color_view: self.scene_color_target.msaa_view(),
                frame_bind_group: self.frame_gpu.sky_bind_group(),
            },
        );

        self.geometry_pass.record(
            &mut encoder,
            GeometryPassInput {
                color_view: self.scene_color_target.msaa_view(),
                resolve_view,
                depth_view,
                frame_bind_group,
                object_bind_group,
                material_assets: &self.material_assets,
                mesh_draws: &self.mesh_draws,
                mesh_assets: &self.mesh_assets,
            },
        );

        if self.lut.needs_update() {
            self.lut_pass.record(
                &mut encoder,
                LutPassInput {
                    lut_view: self.lut.view(),
                    frame_bind_group: self.frame_gpu.lut_bind_group(),
                },
            );

            self.lut.mark_updated();
        }

        self.luminance_pass.record(
            &mut encoder,
            LuminancePassInput {
                scene_view: resolve_view,
                target: &self.luminance_target,
            },
        );

        self.exposure_pass.record(&mut encoder);

        self.tonemapping_pass.record(
            &mut encoder,
            ToneMappingPassInput {
                binding: &self.tonemapping_binding,
                target_view: &surface_view,
            },
        );

        self.debug.render(
            device,
            queue,
            &mut encoder,
            &surface_view,
            [self.graphics.width(), self.graphics.height()],
        );

        queue.submit(std::iter::once(encoder.finish()));
        queue.present(output);

        Ok(())
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

        self.camera_ctrl.on_key(key, pressed);
    }

    pub fn on_mouse_motion(&mut self, delta: (f64, f64)) {
        self.debug.on_mouse_motion(delta);
        self.camera_ctrl.on_mouse_motion(delta);
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
