use crate::graphics::GraphicsContext;

pub struct LuminanceTarget {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    views: Vec<wgpu::TextureView>,
    mip_level_count: u32,
}

impl LuminanceTarget {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    pub fn new(graphics: &GraphicsContext, width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0);

        let mip_level_count = 1 + width.max(height).ilog2();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = graphics.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Scene Luminance Target"),
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let views = (0..mip_level_count)
            .map(|level| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        Self {
            texture,
            views,
            mip_level_count,
        }
    }

    pub fn resize(&mut self, graphics: &GraphicsContext, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        *self = Self::new(graphics, width, height);
    }

    pub fn view(&self, level: u32) -> &wgpu::TextureView {
        &self.views[level as usize]
    }

    pub fn mip_level_count(&self) -> u32 {
        self.mip_level_count
    }

    pub fn final_view(&self) -> &wgpu::TextureView {
        self.view(self.mip_level_count - 1)
    }
}
