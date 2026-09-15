use anyhow::{Result, anyhow};

use crate::{
    assets::{
        data::{ImageData, ImageIndex},
        handles::TextureHandle,
    },
    graphics::{GraphicsContext, Texture, mipmap::MipmapGenerator},
};

pub struct TextureAssets {
    textures: Vec<Texture>,
    image_handles: Vec<TextureHandle>,
    white: TextureHandle,
}

impl TextureAssets {
    pub fn from_images(
        graphics: &GraphicsContext,
        mipmap_generator: &MipmapGenerator,
        images: &[ImageData],
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
        )?;
        mipmap_generator.generate(graphics.device(), &mut encoder, &white_texture);

        let mut assets = Self {
            textures: vec![white_texture],
            image_handles: Vec::with_capacity(images.len()),
            white: TextureHandle::new(0),
        };

        for (index, image) in images.iter().enumerate() {
            let texture = Texture::from_rgba8(
                graphics.device(),
                graphics.queue(),
                &format!("Asset Texture {index}"),
                image.width,
                image.height,
                &image.rgba8,
            )?;
            mipmap_generator.generate(graphics.device(), &mut encoder, &texture);

            let handle = assets.add(texture);
            assets.image_handles.push(handle);
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

    pub fn image(&self, index: ImageIndex) -> Result<TextureHandle> {
        self.image_handles
            .get(index)
            .copied()
            .ok_or_else(|| anyhow!("material references missing image {index}"))
    }

    fn add(&mut self, texture: Texture) -> TextureHandle {
        let handle = TextureHandle::new(self.textures.len());
        self.textures.push(texture);
        handle
    }
}
