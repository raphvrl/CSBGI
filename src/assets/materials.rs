use anyhow::{Result, anyhow};

use crate::{
    assets::{
        data::{MaterialData, MaterialIndex},
        handles::MaterialHandle,
        textures::TextureAssets,
    },
    graphics::{
        GraphicsContext,
        bindings::{MaterialGpu, MaterialLayout},
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

        let default = MaterialGpu::new(graphics, &layout, white_texture, [1.0; 4]);

        let mut assets = Self {
            layout,
            materials: vec![default],
            material_handles: Vec::with_capacity(materials.len()),
            default: MaterialHandle::new(0),
        };

        for material in materials {
            let texture_handle = match material.color_image {
                Some(image) => textures.image(image)?,
                None => textures.white(),
            };
            let texture = textures
                .get(texture_handle)
                .ok_or_else(|| anyhow!("resolved texture handle is invalid"))?;

            let gpu_material =
                MaterialGpu::new(graphics, &assets.layout, texture, material.color_factor);

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
