use crate::graphics::GraphicsContext;

use super::MSAA_SAMPLES;

pub struct SceneColorTarget {
    #[allow(dead_code)]
    msaa_texture: wgpu::Texture,
    msaa_view: wgpu::TextureView,

    #[allow(dead_code)]
    resolve_texture: wgpu::Texture,
    resolve_view: wgpu::TextureView,
}

impl SceneColorTarget {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    pub fn new(graphics: &GraphicsContext, width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0);

        let device = graphics.device();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Scene Color MSAA Target"),
            size,
            mip_level_count: 1,
            sample_count: MSAA_SAMPLES,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let resolve_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Scene Color HDR Target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let resolve_view = resolve_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            msaa_texture,
            msaa_view,
            resolve_texture,
            resolve_view,
        }
    }

    pub fn resize(&mut self, graphics: &GraphicsContext, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        *self = Self::new(graphics, width, height);
    }

    pub fn msaa_view(&self) -> &wgpu::TextureView {
        &self.msaa_view
    }

    pub fn resolve_view(&self) -> &wgpu::TextureView {
        &self.resolve_view
    }
}
