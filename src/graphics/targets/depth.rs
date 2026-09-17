use crate::graphics::GraphicsContext;

use super::MSAA_SAMPLES;

pub struct DepthTarget {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl DepthTarget {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(graphics: &GraphicsContext, width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0);

        let device = graphics.device();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Scene Depth Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: MSAA_SAMPLES,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }

    pub fn resize(&mut self, graphics: &GraphicsContext, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        *self = Self::new(graphics, width, height);
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
