use bytemuck::{Pod, Zeroable};
use wesl::include_wesl;
use wgpu::util::DeviceExt;

use crate::graphics::{
    GraphicsContext,
    builder::{BindGroupExt, BindGroupLayoutExt, ComputePassExt, ComputePipelineExt},
};

const INITIAL_EXPOSURE: f32 = 0.0;

#[derive(Clone, Copy)]
pub struct ExposureSettings {
    pub target_luminance: f32,
    pub min_exposure: f32,
    pub max_exposure: f32,
    pub adaptation_speed: f32,
}

impl Default for ExposureSettings {
    fn default() -> Self {
        Self {
            target_luminance: 0.4,
            min_exposure: -4.0,
            max_exposure: 6.0,
            adaptation_speed: 0.5,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ExposureState {
    exposure: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ExposureParams {
    target_luminance: f32,
    min_exposure: f32,
    max_exposure: f32,
    adaptation_speed: f32,
    delta_time: f32,
    camera_exposure_stops: f32,
    _pad0: [f32; 2],
}

pub struct ExposurePass {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    state_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    settings: ExposureSettings,
}

impl ExposurePass {
    pub fn new(graphics: &GraphicsContext, luminance_view: &wgpu::TextureView) -> Self {
        let device = graphics.device();

        #[rustfmt::skip]
        let bind_group_layout = device
            .bind_group_layout_builder()
            .label("Exposure Bind Group Layout")
            .entry(0)
                .texture()
                .unfilterable()
                .compute()
            .entry(1)
                .storage()
                .compute()
            .entry(2)
                .uniform()
                .compute()
            .build();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Exposure Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("exposure").into()),
        });

        let pipeline = device
            .compute_pipeline_builder()
            .label("Exposure Pipeline")
            .shader(&shader)
            .bind_group_layout(&bind_group_layout)
            .build();

        let state_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Exposure State Buffer"),
            contents: bytemuck::bytes_of(&ExposureState {
                exposure: INITIAL_EXPOSURE,
            }),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Exposure Parameters Buffer"),
            contents: bytemuck::bytes_of(&ExposureParams::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = Self::create_bind_group(
            device,
            &bind_group_layout,
            luminance_view,
            &state_buffer,
            &params_buffer,
        );

        Self {
            pipeline,
            bind_group_layout,
            state_buffer,
            params_buffer,
            bind_group,
            settings: ExposureSettings::default(),
        }
    }

    pub fn update_luminance(
        &mut self,
        graphics: &GraphicsContext,
        luminance_view: &wgpu::TextureView,
    ) {
        self.bind_group = Self::create_bind_group(
            graphics.device(),
            &self.bind_group_layout,
            luminance_view,
            &self.state_buffer,
            &self.params_buffer,
        );
    }

    pub fn update(&self, queue: &wgpu::Queue, delta_time: f32, camera_exposure_stops: f32) {
        let params = ExposureParams {
            target_luminance: self.settings.target_luminance,
            min_exposure: self.settings.min_exposure,
            max_exposure: self.settings.max_exposure,
            adaptation_speed: self.settings.adaptation_speed,
            delta_time,
            camera_exposure_stops,
            _pad0: [0.0; 2],
        };

        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder
            .compute_pass_builder()
            .label("Exposure Compute Pass")
            .build();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.state_buffer
    }

    pub fn settings_mut(&mut self) -> &mut ExposureSettings {
        &mut self.settings
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        luminance_view: &wgpu::TextureView,
        state_buffer: &wgpu::Buffer,
        params_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device
            .bind_group_builder()
            .label("Exposure Bind Group")
            .layout(layout)
            .texture(0, luminance_view)
            .buffer(1, state_buffer)
            .buffer(2, params_buffer)
            .build()
    }
}
