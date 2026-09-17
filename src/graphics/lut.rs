pub struct Lut {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    dirty: bool,
}

impl Lut {
    pub const WIDTH: u32 = 256;
    pub const HEIGHT: u32 = 128;
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Sky LUT"),
            size: wgpu::Extent3d {
                width: Self::WIDTH,
                height: Self::WIDTH,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sky LUT Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            dirty: true,
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn needs_update(&self) -> bool {
        self.dirty
    }

    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    pub fn mark_updated(&mut self) {
        self.dirty = false;
    }
}
