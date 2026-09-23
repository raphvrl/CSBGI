use wesl::include_wesl;

use crate::graphics::{
    GraphicsContext, VoxelGpu,
    builder::{BindGroupExt, BindGroupLayoutExt, ComputePassExt, ComputePipelineExt},
    pass::voxel::hierarchy::{MIP_TEXTURE_FORMAT, MipHierarchy},
};

const WORKGROUP_SIZE: u32 = 64;
const MIP_LEVEL_COUNT: usize = 8;

pub struct MipPass {
    base_pipeline: wgpu::ComputePipeline,
    downsample_pipelines: [wgpu::ComputePipeline; MIP_LEVEL_COUNT - 1],
    voxel_bind_group: wgpu::BindGroup,
    base_bind_group: wgpu::BindGroup,
    downsample_bind_groups: [wgpu::BindGroup; MIP_LEVEL_COUNT - 1],
    level_counts: [u32; MIP_LEVEL_COUNT],
}

impl MipPass {
    pub fn new(
        graphics: &GraphicsContext,
        voxel_layout: &wgpu::BindGroupLayout,
        voxel_gpu: &VoxelGpu,
        hierarchy: &MipHierarchy,
    ) -> Self {
        let device = graphics.device();

        let base_layout = device
            .bind_group_layout_builder()
            .label("Voxel Global Mip Base Layout")
            .entry(0)
            .storage()
            .read_only()
            .compute()
            .entry(1)
            .storage_texture(
                MIP_TEXTURE_FORMAT,
                wgpu::StorageTextureAccess::WriteOnly,
                wgpu::TextureViewDimension::D3,
            )
            .compute()
            .entry(2)
            .uniform()
            .compute()
            .build();
        let downsample_layout = device
            .bind_group_layout_builder()
            .label("Voxel Global Mip Downsample Layout")
            .entry(0)
            .texture_with(
                wgpu::TextureSampleType::Float { filterable: false },
                wgpu::TextureViewDimension::D3,
                false,
            )
            .compute()
            .entry(1)
            .storage_texture(
                MIP_TEXTURE_FORMAT,
                wgpu::StorageTextureAccess::WriteOnly,
                wgpu::TextureViewDimension::D3,
            )
            .compute()
            .entry(2)
            .uniform()
            .compute()
            .build();

        let base_bind_group = device
            .bind_group_builder()
            .label("Voxel Global Mip Base Bind Group")
            .layout(&base_layout)
            .buffer(0, voxel_gpu.radiance_buffer())
            .texture(1, hierarchy.mip_level_view(0))
            .buffer(2, hierarchy.info_buffer())
            .build();
        let downsample_bind_groups = std::array::from_fn(|level| {
            device
                .bind_group_builder()
                .label("Voxel Global Mip Downsample Bind Group")
                .layout(&downsample_layout)
                .texture(0, hierarchy.mip_level_view(level))
                .texture(1, hierarchy.mip_level_view(level + 1))
                .buffer(2, hierarchy.info_buffer())
                .build()
        });

        let base_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Global Mip Base Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("voxel_mipmap_base").into()),
        });
        let mip_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Global Mip Downsample Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("voxel_mipmap").into()),
        });

        let base_pipeline = device
            .compute_pipeline_builder()
            .label("Voxel Global Mip Base Pipeline")
            .shader(&base_shader)
            .bind_group_layout(voxel_layout)
            .bind_group_layout(&base_layout)
            .entry_point("cs_main")
            .build();
        let downsample_pipelines = std::array::from_fn(|level| {
            device
                .compute_pipeline_builder()
                .label(format!("Voxel Global Mip {level} to {} Pipeline", level + 1))
                .shader(&mip_shader)
                .bind_group_layout(&downsample_layout)
                .entry_point(format!("cs_mip_{level}_to_{}", level + 1))
                .build()
        });

        Self {
            base_pipeline,
            downsample_pipelines,
            voxel_bind_group: voxel_gpu.bind_group().clone(),
            base_bind_group,
            downsample_bind_groups,
            level_counts: std::array::from_fn(|level| hierarchy.level_count(level as u32)),
        }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.level_counts[0] == 0 {
            return;
        }

        let mut compute_pass = encoder
            .compute_pass_builder()
            .label("Voxel Global Mipmap Compute Pass")
            .build();

        compute_pass.set_pipeline(&self.base_pipeline);
        compute_pass.set_bind_group(0, &self.voxel_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.base_bind_group, &[]);
        compute_pass.dispatch_workgroups(self.level_counts[0].div_ceil(WORKGROUP_SIZE), 1, 1);

        for (level, pipeline) in self.downsample_pipelines.iter().enumerate() {
            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &self.downsample_bind_groups[level], &[]);
            compute_pass.dispatch_workgroups(self.level_counts[level + 1].div_ceil(WORKGROUP_SIZE), 1, 1);
        }
    }
}
