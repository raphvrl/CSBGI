use std::time::Instant;

use anyhow::Result;
use glam::Mat4;

use crate::{
    assets::{AssetLoader, MaterialAssets, MeshAssets, TextureAssets, data::SceneData, upload_scene},
    ecs::{MeshRenderer, Transform, scene::spawn_scene},
    graphics::{
        GraphicsContext,
        lighting::{DirectionalLight, ShadowSettings},
        mipmap::MipmapGenerator,
        pass::MeshDraw,
    },
};

pub struct Scene {
    world: hecs::World,
    mesh_assets: MeshAssets,
    #[allow(dead_code)]
    texture_assets: TextureAssets,
    material_assets: MaterialAssets,
    mesh_draws: Vec<MeshDraw>,
    light: DirectionalLight,
    shadow_settings: ShadowSettings,
}

impl Scene {
    pub(crate) fn load_data(startup: &Instant) -> Result<SceneData> {
        let scene_data = AssetLoader::default().load("assets/models/Sponza/glTF/Sponza.gltf")?;
        tracing::info!(stage = "scene_load", elapsed_ms = startup.elapsed().as_millis());

        Ok(scene_data)
    }

    pub(crate) fn new(graphics: &GraphicsContext, scene_data: SceneData, startup: &Instant) -> Result<Self> {
        let mut mesh_assets = MeshAssets::default();
        let mipmap_generator = MipmapGenerator::new(graphics.device());
        let texture_assets = TextureAssets::from_images(
            graphics,
            &mipmap_generator,
            &scene_data.images,
            &scene_data.materials,
        )?;

        tracing::info!(
            stage = "texture_upload",
            elapsed_ms = startup.elapsed().as_millis()
        );

        let material_assets = MaterialAssets::from_data(graphics, &scene_data.materials, &texture_assets)?;
        let scene_asset = upload_scene(graphics.device(), &mut mesh_assets, &material_assets, scene_data)?;

        tracing::info!(stage = "scene_upload", elapsed_ms = startup.elapsed().as_millis());

        let mut world = hecs::World::new();
        spawn_scene(&mut world, &scene_asset);

        Ok(Self {
            world,
            mesh_assets,
            texture_assets,
            material_assets,
            mesh_draws: Vec::new(),
            light: DirectionalLight::default(),
            shadow_settings: ShadowSettings::default(),
        })
    }

    pub(crate) fn prepare_draws(&mut self) {
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
    }

    pub(crate) fn model_matrices(&self) -> Vec<Mat4> {
        self.mesh_draws.iter().map(|draw| draw.model).collect()
    }

    pub(crate) fn mesh_count(&self) -> usize {
        self.mesh_draws.len()
    }

    pub(crate) fn draws(&self) -> &[MeshDraw] {
        &self.mesh_draws
    }

    pub(crate) fn mesh_assets(&self) -> &MeshAssets {
        &self.mesh_assets
    }

    pub(crate) fn material_assets(&self) -> &MaterialAssets {
        &self.material_assets
    }

    pub(crate) fn light(&self) -> &DirectionalLight {
        &self.light
    }

    pub(crate) fn light_mut(&mut self) -> &mut DirectionalLight {
        &mut self.light
    }

    pub(crate) fn shadow_settings(&self) -> &ShadowSettings {
        &self.shadow_settings
    }

    pub(crate) fn light_signature(&self) -> (glam::Vec3, glam::Vec3, f32, f32) {
        (
            self.light.direction,
            self.light.color,
            self.light.illuminance_lux,
            self.light.ambient_illuminance_lux,
        )
    }
}
