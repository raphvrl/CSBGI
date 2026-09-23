use wesl::include_wesl;

use crate::graphics::{
    GraphicsContext,
    builder::{ComputePassExt, ComputePipelineExt},
};

const WORKGROUP_SIZE: u32 = 64;

pub struct LightingPass {
    pipeline: wgpu::ComputePipeline,
}

pub struct LightingInput<'a> {
    pub frame_bind_group: &'a wgpu::BindGroup,
    pub voxel_bind_group: &'a wgpu::BindGroup,
    pub lighting_bind_group: &'a wgpu::BindGroup,
    pub occupied_voxel_count: u32,
}

impl LightingPass {
    pub fn new(
        graphics: &GraphicsContext,
        frame_layout: &wgpu::BindGroupLayout,
        voxel_layout: &wgpu::BindGroupLayout,
        lighting_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let device = graphics.device();
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Lighting Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("voxel_lighting").into()),
        });

        let pipeline = device
            .compute_pipeline_builder()
            .label("Voxel Lighting Pipeline")
            .shader(&shader)
            .bind_group_layout(frame_layout)
            .bind_group_layout(voxel_layout)
            .bind_group_layout(lighting_layout)
            .entry_point("cs_main")
            .build();

        Self { pipeline }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: LightingInput<'_>) {
        if input.occupied_voxel_count == 0 {
            return;
        }

        let workgroup_count = input.occupied_voxel_count.div_ceil(WORKGROUP_SIZE);
        let mut compute_pass = encoder
            .compute_pass_builder()
            .label("Voxel Lighting Compute Pass")
            .build();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, input.frame_bind_group, &[]);
        compute_pass.set_bind_group(1, input.voxel_bind_group, &[]);
        compute_pass.set_bind_group(2, input.lighting_bind_group, &[]);
        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
    }
}
