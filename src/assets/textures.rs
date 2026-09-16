use anyhow::{Result, anyhow};
use std::collections::HashSet;

use crate::{
    assets::{
        data::{ImageData, ImageIndex, MaterialData},
        handles::TextureHandle,
    },
    graphics::{GraphicsContext, Texture, mipmap::MipmapGenerator},
};

pub struct TextureAssets {
    textures: Vec<Texture>,
    color_handles: Vec<TextureHandle>,
    normal_handles: Vec<Option<TextureHandle>>,
    white: TextureHandle,
    flat_normal: TextureHandle,
}

impl TextureAssets {
    pub fn from_images(
        graphics: &GraphicsContext,
        mipmap_generator: &MipmapGenerator,
        images: &[ImageData],
        materials: &[MaterialData],
    ) -> Result<Self> {
        let mut encoder =
            graphics
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Texture Upload Encoder"),
                });

        let white_texture = Texture::from_rgba8(
            graphics.device(),
            graphics.queue(),
            "Default White Texture",
            1,
            1,
            &[255, 255, 255, 255],
            wgpu::TextureFormat::Rgba8UnormSrgb,
        )?;
        mipmap_generator.generate(graphics.device(), &mut encoder, &white_texture);

        let flat_normal_texture = Texture::from_rgba8(
            graphics.device(),
            graphics.queue(),
            "Default Flat Normal Texture",
            1,
            1,
            &[128, 128, 255, 255],
            wgpu::TextureFormat::Rgba8Unorm,
        )?;
        mipmap_generator.generate(graphics.device(), &mut encoder, &flat_normal_texture);

        let normal_images = materials
            .iter()
            .filter_map(|material| material.normal_image)
            .collect::<HashSet<_>>();

        let mut assets = Self {
            textures: vec![white_texture, flat_normal_texture],
            color_handles: Vec::with_capacity(images.len()),
            normal_handles: vec![None; images.len()],
            white: TextureHandle::new(0),
            flat_normal: TextureHandle::new(1),
        };

        for (index, image) in images.iter().enumerate() {
            let color_texture = Texture::from_rgba8(
                graphics.device(),
                graphics.queue(),
                &format!("Asset Color Texture {index}"),
                image.width,
                image.height,
                &image.rgba8,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            )?;
            mipmap_generator.generate(graphics.device(), &mut encoder, &color_texture);

            let color_handle = assets.add(color_texture);
            assets.color_handles.push(color_handle);

            if normal_images.contains(&index) {
                let normal_texture = Texture::from_rgba8(
                    graphics.device(),
                    graphics.queue(),
                    &format!("Asset Normal Texture {index}"),
                    image.width,
                    image.height,
                    &image.rgba8,
                    wgpu::TextureFormat::Rgba8Unorm,
                )?;
                mipmap_generator.generate(graphics.device(), &mut encoder, &normal_texture);

                assets.normal_handles[index] = Some(assets.add(normal_texture));
            }
        }

        graphics.queue().submit(Some(encoder.finish()));

        Ok(assets)
    }

    pub fn get(&self, handle: TextureHandle) -> Option<&Texture> {
        self.textures.get(handle.index())
    }

    pub fn white(&self) -> TextureHandle {
        self.white
    }

    pub fn flat_normal(&self) -> TextureHandle {
        self.flat_normal
    }

    pub fn color(&self, index: ImageIndex) -> Result<TextureHandle> {
        self.color_handles
            .get(index)
            .copied()
            .ok_or_else(|| anyhow!("material references missing image {index}"))
    }

    pub fn normal(&self, index: ImageIndex) -> Result<TextureHandle> {
        self.normal_handles
            .get(index)
            .and_then(|handle| *handle)
            .ok_or_else(|| anyhow!("material references missing normal image {index}"))
    }

    fn add(&mut self, texture: Texture) -> TextureHandle {
        let handle = TextureHandle::new(self.textures.len());
        self.textures.push(texture);
        handle
    }
}
