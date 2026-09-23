use anyhow::{Result, anyhow};

use crate::{
    assets::{
        data::{MaterialData, MaterialIndex},
        handles::MaterialHandle,
        textures::TextureAssets,
    },
    graphics::{
        GraphicsContext,
        bindings::{MaterialGpu, MaterialLayout, MaterialParameters},
    },
};

pub struct MaterialAssets {
    layout: MaterialLayout,
    materials: Vec<MaterialGpu>,
    material_handles: Vec<MaterialHandle>,
    default: MaterialHandle,
}

impl MaterialAssets {
    pub fn from_data(
        graphics: &GraphicsContext,
        materials: &[MaterialData],
        textures: &TextureAssets,
    ) -> Result<Self> {
        let layout = MaterialLayout::new(graphics);
        let white_texture = textures
            .get(textures.white())
            .ok_or_else(|| anyhow!("default white texture is missing"))?;
        let flat_normal_texture = textures
            .get(textures.flat_normal())
            .ok_or_else(|| anyhow!("default flat normal texture is missing"))?;
        let default_orm_texture = textures
            .get(textures.default_orm())
            .ok_or_else(|| anyhow!("default ORM texture is missing"))?;

        let default = MaterialGpu::new(
            graphics,
            &layout,
            white_texture,
            flat_normal_texture,
            default_orm_texture,
            MaterialParameters {
                base_color_factor: [1.0; 4],
                normal_scale: 1.0,
                metallic_factor: 0.0,
                roughness_factor: 1.0,
                occlusion_factor: 1.0,
            },
        );

        let mut assets = Self {
            layout,
            materials: vec![default],
            material_handles: Vec::with_capacity(materials.len()),
            default: MaterialHandle::new(0),
        };

        for material in materials {
            let color_handle = match material.color_image {
                Some(image) => textures.color(image)?,
                None => textures.white(),
            };
            let normal_handle = match material.normal_image {
                Some(image) => textures.normal(image)?,
                None => textures.flat_normal(),
            };
            let orm_handle = match material.orm_image {
                Some(image) => textures.orm(image)?,
                None => textures.default_orm(),
            };
            let color_texture = textures
                .get(color_handle)
                .ok_or_else(|| anyhow!("resolved texture handle is invalid"))?;
            let normal_texture = textures
                .get(normal_handle)
                .ok_or_else(|| anyhow!("resolved normal texture handle is invalid"))?;
            let orm_texture = textures
                .get(orm_handle)
                .ok_or_else(|| anyhow!("resolved ORM texture handle is invalid"))?;

            let gpu_material = MaterialGpu::new(
                graphics,
                &assets.layout,
                color_texture,
                normal_texture,
                orm_texture,
                MaterialParameters {
                    base_color_factor: material.color_factor,
                    normal_scale: material.normal_scale,
                    metallic_factor: material.metallic_factor,
                    roughness_factor: material.roughness_factor,
                    occlusion_factor: material.occlusion_factor,
                },
            );

            let handle = MaterialHandle::new(assets.materials.len());
            assets.materials.push(gpu_material);
            assets.material_handles.push(handle);
        }

        Ok(assets)
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        self.layout.layout()
    }

    pub fn get(&self, handle: MaterialHandle) -> Option<&MaterialGpu> {
        self.materials.get(handle.index())
    }

    pub fn resolve(&self, material: Option<MaterialIndex>) -> Result<MaterialHandle> {
        match material {
            Some(index) => self
                .material_handles
                .get(index)
                .copied()
                .ok_or_else(|| anyhow!("mesh references missing material {index}")),
            None => Ok(self.default),
        }
    }
}
