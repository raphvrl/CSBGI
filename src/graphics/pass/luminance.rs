use wesl::include_wesl;

use crate::graphics::{
    GraphicsContext,
    builder::{BindGroupExt, BindGroupLayoutExt, RenderPassExt, RenderPipelineExt},
    targets::LuminanceTarget,
};

pub struct LuminancePass {
    luminance_pipeline: wgpu::RenderPipeline,
    reduce_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    scene_bind_group: wgpu::BindGroup,
    reduce_bind_groups: Vec<wgpu::BindGroup>,
}

pub struct LuminancePassInput<'a> {
    pub scene_view: &'a wgpu::TextureView,
    pub target: &'a LuminanceTarget,
}

impl LuminancePass {
    pub fn new(graphics: &GraphicsContext, scene_view: &wgpu::TextureView, target: &LuminanceTarget) -> Self {
        let device = graphics.device();

        #[rustfmt::skip]
        let bind_group_layout = device
            .bind_group_layout_builder()
            .label("Luminance Bind Group Layout")
            .entry(0)
                .texture()
                .fragment()
            .entry(1)
                .sampler()
                .fragment()
            .build();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Luminance Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let luminance_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Luminance Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("luminance").into()),
        });

        let reduce_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Luminance Reduce Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("reduce").into()),
        });

        let luminance_pipeline = device
            .render_pipeline_builder()
            .label("Luminance Pipeline")
            .shader(&luminance_shader)
            .bind_group_layout(&bind_group_layout)
            .color_format(LuminanceTarget::FORMAT)
            .build();

        let reduce_pipeline = device
            .render_pipeline_builder()
            .label("Luminance Reduce Pipeline")
            .shader(&reduce_shader)
            .bind_group_layout(&bind_group_layout)
            .color_format(LuminanceTarget::FORMAT)
            .build();

        let scene_bind_group = Self::create_bind_group(
            device,
            &bind_group_layout,
            scene_view,
            &sampler,
            "Luminance Scene Bind Group",
        );

        let mut pass = Self {
            luminance_pipeline,
            reduce_pipeline,
            bind_group_layout,
            sampler,
            scene_bind_group,
            reduce_bind_groups: Vec::new(),
        };

        pass.rebuild_reduce_bind_groups(device, target);
        pass
    }

    pub fn update_bindings(
        &mut self,
        graphics: &GraphicsContext,
        scene_view: &wgpu::TextureView,
        target: &LuminanceTarget,
    ) {
        let device = graphics.device();

        self.scene_bind_group = Self::create_bind_group(
            device,
            &self.bind_group_layout,
            scene_view,
            &self.sampler,
            "Luminance Scene Bind Group",
        );
        self.rebuild_reduce_bind_groups(device, target);
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: LuminancePassInput<'_>) {
        {
            let mut render_pass = encoder
                .render_pass_builder()
                .label("Luminance Render Pass")
                .color(input.target.view(0))
                .clear(wgpu::Color::BLACK)
                .store()
                .build();

            render_pass.set_pipeline(&self.luminance_pipeline);
            render_pass.set_bind_group(0, &self.scene_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        for level in 0..input.target.mip_level_count() - 1 {
            let mut render_pass = encoder
                .render_pass_builder()
                .label("Luminance Reduce Render Pass")
                .color(input.target.view(level + 1))
                .clear(wgpu::Color::BLACK)
                .store()
                .build();

            render_pass.set_pipeline(&self.reduce_pipeline);
            render_pass.set_bind_group(0, &self.reduce_bind_groups[level as usize], &[]);
            render_pass.draw(0..3, 0..1);
        }
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        source_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        label: &'static str,
    ) -> wgpu::BindGroup {
        device
            .bind_group_builder()
            .label(label)
            .layout(layout)
            .texture(0, source_view)
            .sampler(1, sampler)
            .build()
    }

    fn rebuild_reduce_bind_groups(&mut self, device: &wgpu::Device, target: &LuminanceTarget) {
        self.reduce_bind_groups = (0..target.mip_level_count() - 1)
            .map(|level| {
                Self::create_bind_group(
                    device,
                    &self.bind_group_layout,
                    target.view(level),
                    &self.sampler,
                    "Luminance Reduce Bind Group",
                )
            })
            .collect();
    }
}
