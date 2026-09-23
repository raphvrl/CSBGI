use anyhow::Result;

use crate::{
    camera::CameraRig,
    debug::DebugUi,
    graphics::{
        GraphicsContext,
        bindings::{FrameGpu, ObjectGpu, ToneMappingBinding},
        lighting::DirectionalLight,
        lut::Lut,
        pass::{
            exposure::{ExposurePass, ExposureSettings},
            geometry::{GeometryPass, GeometryPassInput},
            luminance::{LuminancePass, LuminancePassInput},
            lut::{LutPass, LutPassInput},
            shadow::{ShadowPass, ShadowPassInput},
            sky::{SkyPass, SkyPassInput},
            tonemapping::{ToneMappingPass, ToneMappingPassInput},
        },
        targets::{DepthTarget, LuminanceTarget, SceneColorTarget, ShadowTarget},
    },
    scene::Scene,
    voxel::gi::VoxelGi,
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

#[derive(Default)]
struct RendererControls {
    auto_exposure_enabled: bool,
    manual_exposure: f32,
}

impl RendererControls {
    fn new() -> Self {
        Self {
            auto_exposure_enabled: true,
            manual_exposure: 0.0,
        }
    }
}

pub(crate) struct RendererResources {
    scene_color_target: SceneColorTarget,
    luminance_target: LuminanceTarget,
    depth_target: DepthTarget,
    shadow_target: ShadowTarget,
    frame_gpu: FrameGpu,
    object_gpu: ObjectGpu,
    lut: Lut,
}

impl RendererResources {
    pub(crate) fn new(graphics: &GraphicsContext, width: u32, height: u32) -> Self {
        let scene_color_target = SceneColorTarget::new(graphics, width, height);
        let luminance_target = LuminanceTarget::new(graphics, width, height);
        let depth_target = DepthTarget::new(graphics, width, height);
        let shadow_target = ShadowTarget::new(graphics);
        let lut = Lut::new(graphics.device());
        let frame_gpu = FrameGpu::new(graphics, &shadow_target, &depth_target, &lut);
        let object_gpu = ObjectGpu::new(graphics);

        Self {
            scene_color_target,
            luminance_target,
            depth_target,
            shadow_target,
            frame_gpu,
            object_gpu,
            lut,
        }
    }

    pub(crate) fn frame_layout(&self) -> &wgpu::BindGroupLayout {
        self.frame_gpu.layout()
    }
}

pub(crate) struct Renderer {
    scene_color_target: SceneColorTarget,
    luminance_target: LuminanceTarget,
    depth_target: DepthTarget,
    shadow_target: ShadowTarget,
    frame_gpu: FrameGpu,
    object_gpu: ObjectGpu,
    shadow_pass: ShadowPass,
    sky_pass: SkyPass,
    geometry_pass: GeometryPass,
    lut: Lut,
    lut_pass: LutPass,
    lut_key: Option<LutKey>,
    lut_timer: f32,
    luminance_pass: LuminancePass,
    exposure_pass: ExposurePass,
    tonemapping_binding: ToneMappingBinding,
    tonemapping_pass: ToneMappingPass,
    controls: RendererControls,
}

pub(crate) struct RenderFrame<'a> {
    pub(crate) scene: &'a Scene,
    pub(crate) voxel: &'a mut VoxelGi,
    pub(crate) debug: &'a mut DebugUi,
}

impl Renderer {
    pub(crate) fn new(
        graphics: &GraphicsContext,
        resources: RendererResources,
        scene: &Scene,
        voxel: &VoxelGi,
    ) -> Self {
        let RendererResources {
            scene_color_target,
            luminance_target,
            depth_target,
            shadow_target,
            frame_gpu,
            object_gpu,
            lut,
        } = resources;

        let shadow_pass = ShadowPass::new(
            graphics,
            frame_gpu.shadow_layout(),
            object_gpu.layout(),
            scene.material_assets().layout(),
        );
        let sky_pass = SkyPass::new(graphics, frame_gpu.sky_layout());
        let geometry_pass = GeometryPass::new(
            graphics,
            frame_gpu.layout(),
            object_gpu.layout(),
            scene.material_assets().layout(),
            voxel.sampling_layout(),
        );
        let lut_pass = LutPass::new(graphics, frame_gpu.lut_layout());
        let luminance_pass = LuminancePass::new(
            graphics,
            scene_color_target.resolve_view(),
            &luminance_target,
        );
        let exposure_pass = ExposurePass::new(graphics, luminance_target.final_view());
        let tonemapping_binding = ToneMappingBinding::new(
            graphics,
            scene_color_target.resolve_view(),
            exposure_pass.buffer(),
        );
        let tonemapping_pass = ToneMappingPass::new(graphics, &tonemapping_binding);

        Self {
            scene_color_target,
            luminance_target,
            depth_target,
            shadow_target,
            frame_gpu,
            object_gpu,
            shadow_pass,
            sky_pass,
            geometry_pass,
            lut,
            lut_pass,
            lut_key: None,
            lut_timer: 0.0,
            luminance_pass,
            exposure_pass,
            tonemapping_binding,
            tonemapping_pass,
            controls: RendererControls::new(),
        }
    }

    pub(crate) fn debug_settings_mut(&mut self) -> (&mut ExposureSettings, &mut bool, &mut f32) {
        let Self {
            exposure_pass,
            controls,
            ..
        } = self;
        (
            exposure_pass.settings_mut(),
            &mut controls.auto_exposure_enabled,
            &mut controls.manual_exposure,
        )
    }

    pub(crate) fn resize(&mut self, graphics: &GraphicsContext, width: u32, height: u32) {
        self.scene_color_target.resize(graphics, width, height);
        self.luminance_target.resize(graphics, width, height);
        self.tonemapping_binding.update_source(
            graphics.device(),
            self.scene_color_target.resolve_view(),
            self.exposure_pass.buffer(),
        );
        self.luminance_pass.update_bindings(
            graphics,
            self.scene_color_target.resolve_view(),
            &self.luminance_target,
        );
        self.exposure_pass
            .update_luminance(graphics, self.luminance_target.final_view());
        self.depth_target.resize(graphics, width, height);
        self.frame_gpu
            .update_depth_target(graphics, &self.depth_target, &self.lut);
    }

    pub(crate) fn update(&mut self, dt: f32) {
        self.lut_timer += dt;
    }

    pub(crate) fn prepare(
        &mut self,
        graphics: &GraphicsContext,
        scene: &Scene,
        camera: &CameraRig,
        dt: f32,
    ) {
        let models = scene.model_matrices();
        self.object_gpu.update(graphics, &models);
        self.exposure_pass
            .set_enabled(self.controls.auto_exposure_enabled);

        if self.controls.auto_exposure_enabled {
            self.exposure_pass
                .update(graphics.queue(), dt, camera.exposure_stops());
        } else {
            self.exposure_pass
                .set_manual_exposure(graphics.queue(), self.controls.manual_exposure);
        }

        let key = LutKey::new(camera.camera().position.y, scene.light());
        let initial = self.lut_key.is_none();
        let changed = self
            .lut_key
            .as_ref()
            .is_none_or(|previous| previous.changed(&key));

        if changed && (initial || self.lut_timer >= LUT_UPDATE_INTERVAL) {
            self.lut.invalidate();
            self.lut_key = Some(key);
            self.lut_timer = 0.0;
        }

        let light_view_projection = scene
            .light()
            .shadow_view_projection(scene.shadow_settings());
        self.frame_gpu.update(
            graphics.queue(),
            camera.camera(),
            scene.light(),
            light_view_projection,
        );
    }

    pub(crate) fn render(
        &mut self,
        graphics: &mut GraphicsContext,
        frame: RenderFrame<'_>,
    ) -> Result<()> {
        let Some(output) = graphics.acquire_frame()? else {
            return Ok(());
        };

        let device = graphics.device();
        let queue = graphics.queue();
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

        if frame.voxel.needs_shadow_pass() {
            self.shadow_pass.record(
                &mut encoder,
                ShadowPassInput {
                    shadow_view: self.shadow_target.view(),
                    frame_bind_group: shadow_frame_bind_group,
                    object_bind_group,
                    material_assets: frame.scene.material_assets(),
                    mesh_draws: frame.scene.draws(),
                    mesh_assets: frame.scene.mesh_assets(),
                },
            );
        }

        frame.voxel.record_lighting(&mut encoder, frame_bind_group);

        self.sky_pass.record(
            &mut encoder,
            SkyPassInput {
                color_view: self.scene_color_target.msaa_view(),
                frame_bind_group: self.frame_gpu.sky_bind_group(),
            },
        );

        if !frame.voxel.uses_solid_debug() {
            self.geometry_pass.record(
                &mut encoder,
                GeometryPassInput {
                    color_view: self.scene_color_target.msaa_view(),
                    resolve_view,
                    depth_view,
                    frame_bind_group,
                    voxel_sampling_bind_group: frame.voxel.sampling_bind_group(),
                    object_bind_group,
                    material_assets: frame.scene.material_assets(),
                    mesh_draws: frame.scene.draws(),
                    mesh_assets: frame.scene.mesh_assets(),
                },
            );
        }

        frame.voxel.record_debug(
            queue,
            &mut encoder,
            self.scene_color_target.msaa_view(),
            resolve_view,
            depth_view,
            frame_bind_group,
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

        frame.debug.render(
            device,
            queue,
            &mut encoder,
            &surface_view,
            [graphics.width(), graphics.height()],
        );

        queue.submit(std::iter::once(encoder.finish()));
        queue.present(output);
        Ok(())
    }
}
