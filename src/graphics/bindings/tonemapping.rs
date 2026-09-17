use crate::graphics::{
    GraphicsContext,
    builder::{BindGroupExt, BindGroupLayoutExt},
};

pub struct ToneMappingBinding {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
}

impl ToneMappingBinding {
    pub fn new(
        graphics: &GraphicsContext,
        source_view: &wgpu::TextureView,
        exposure_buffer: &wgpu::Buffer,
    ) -> Self {
        let device = graphics.device();

        #[rustfmt::skip]
        let layout = device
            .bind_group_layout_builder()
            .label("Tone Mapping Bind Group Layout")
            .entry(0)
                .texture()
                .fragment()
            .entry(1)
                .sampler()
                .fragment()
            .entry(2)
                .storage_read_only()
                .fragment()
            .build();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Tone Mapping Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group =
            Self::create_bind_group(device, &layout, source_view, &sampler, exposure_buffer);

        Self {
            layout,
            sampler,
            bind_group,
        }
    }

    pub fn update_source(
        &mut self,
        device: &wgpu::Device,
        source_view: &wgpu::TextureView,
        exposure_buffer: &wgpu::Buffer,
    ) {
        self.bind_group = Self::create_bind_group(
            device,
            &self.layout,
            source_view,
            &self.sampler,
            exposure_buffer,
        );
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        source_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        exposure_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device
            .bind_group_builder()
            .label("Tone Mapping Bind Group")
            .layout(layout)
            .entry(0)
            .texture(source_view)
            .entry(1)
            .sampler(sampler)
            .entry(2)
            .buffer(exposure_buffer)
            .build()
    }
}
