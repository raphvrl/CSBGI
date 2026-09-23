use wesl::include_wesl;

use crate::graphics::{
    GraphicsContext,
    builder::{BindGroupExt, BindGroupLayoutExt, RenderPassExt, RenderPipelineExt},
    pass::voxel::hierarchy::MipHierarchy,
    targets::{DepthTarget, MSAA_SAMPLES, SceneColorTarget},
};
pub struct VoxelDebugPass {
    brick_pipeline: wgpu::RenderPipeline,
    voxel_pipeline: wgpu::RenderPipeline,
    voxel_albedo_pipeline: wgpu::RenderPipeline,
    voxel_normal_pipeline: wgpu::RenderPipeline,
    voxel_radiance_pipeline: wgpu::RenderPipeline,
    mipmap_pipeline: wgpu::RenderPipeline,
    mipmap_bind_group: wgpu::BindGroup,
    empty_bind_group: wgpu::BindGroup,
    mipmap_settings_buffer: wgpu::Buffer,
}

pub struct VoxelDebugPassInput<'a> {
    pub color_view: &'a wgpu::TextureView,
    pub resolve_view: &'a wgpu::TextureView,
    pub depth_view: &'a wgpu::TextureView,
    pub frame_bind_group: &'a wgpu::BindGroup,
    pub voxel_bind_group: &'a wgpu::BindGroup,
    pub radiance_bind_group: &'a wgpu::BindGroup,
    pub brick_count: u32,
    pub occupied_voxel_count: u32,
    pub visible_face_count: u32,
}

#[derive(Clone, Copy)]
struct PipelineLayouts<'a> {
    frame: &'a wgpu::BindGroupLayout,
    voxel: &'a wgpu::BindGroupLayout,
    radiance: &'a wgpu::BindGroupLayout,
}

impl VoxelDebugPass {
    pub fn new(
        graphics: &GraphicsContext,
        frame_layout: &wgpu::BindGroupLayout,
        voxel_layout: &wgpu::BindGroupLayout,
        radiance_layout: &wgpu::BindGroupLayout,
        hierarchy: &MipHierarchy,
    ) -> Self {
        let device = graphics.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Debug Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("debug_voxel").into()),
        });

        let brick_pipeline = Self::create_pipeline(
            device,
            &shader,
            PipelineLayouts {
                frame: frame_layout,
                voxel: voxel_layout,
                radiance: radiance_layout,
            },
            "vs_main",
            "Voxel Debug Pipeline",
            wgpu::PrimitiveTopology::LineList,
        );
        let voxel_pipeline = Self::create_pipeline(
            device,
            &shader,
            PipelineLayouts {
                frame: frame_layout,
                voxel: voxel_layout,
                radiance: radiance_layout,
            },
            "vs_voxel",
            "Occupied Voxel Debug Pipeline",
            wgpu::PrimitiveTopology::LineList,
        );
        let voxel_albedo_pipeline = Self::create_pipeline(
            device,
            &shader,
            PipelineLayouts {
                frame: frame_layout,
                voxel: voxel_layout,
                radiance: radiance_layout,
            },
            "vs_voxel_albedo",
            "Occupied Voxel Albedo Debug Pipeline",
            wgpu::PrimitiveTopology::TriangleList,
        );
        let voxel_normal_pipeline = Self::create_pipeline(
            device,
            &shader,
            PipelineLayouts {
                frame: frame_layout,
                voxel: voxel_layout,
                radiance: radiance_layout,
            },
            "vs_voxel_normal",
            "Occupied Voxel Normal Debug Pipeline",
            wgpu::PrimitiveTopology::TriangleList,
        );
        let voxel_radiance_pipeline = Self::create_pipeline(
            device,
            &shader,
            PipelineLayouts {
                frame: frame_layout,
                voxel: voxel_layout,
                radiance: radiance_layout,
            },
            "vs_voxel_radiance",
            "Occupied Voxel Radiance Debug Pipeline",
            wgpu::PrimitiveTopology::TriangleList,
        );

        let mipmap_layout = device
            .bind_group_layout_builder()
            .label("Voxel Mipmap Debug Bind Group Layout")
            .entry(0)
            .texture()
            .view_dimension(wgpu::TextureViewDimension::D3)
            .vertex()
            .entry(1)
            .sampler()
            .vertex()
            .entry(2)
            .uniform()
            .vertex()
            .entry(3)
            .uniform()
            .vertex()
            .build();

        let mipmap_settings_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Voxel Mipmap Debug Settings Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mipmap_bind_group = device
            .bind_group_builder()
            .label("Voxel Mipmap Debug Bind Group")
            .layout(&mipmap_layout)
            .texture(0, hierarchy.mip_view())
            .sampler(1, hierarchy.mip_sampler())
            .buffer(2, hierarchy.info_buffer())
            .buffer(3, &mipmap_settings_buffer)
            .build();
        let empty_layout = device
            .bind_group_layout_builder()
            .label("Voxel Mipmap Debug Empty Layout")
            .build();
        let empty_bind_group = device
            .bind_group_builder()
            .label("Voxel Mipmap Debug Empty Bind Group")
            .layout(&empty_layout)
            .build();
        let mipmap_pipeline = Self::create_mipmap_pipeline(
            device,
            &shader,
            frame_layout,
            voxel_layout,
            &empty_layout,
            &mipmap_layout,
        );

        Self {
            brick_pipeline,
            voxel_pipeline,
            voxel_albedo_pipeline,
            voxel_normal_pipeline,
            voxel_radiance_pipeline,
            mipmap_pipeline,
            mipmap_bind_group,
            empty_bind_group,
            mipmap_settings_buffer,
        }
    }

    fn create_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        layouts: PipelineLayouts<'_>,
        vertex_entry_point: &str,
        label: &str,
        topology: wgpu::PrimitiveTopology,
    ) -> wgpu::RenderPipeline {
        device
            .render_pipeline_builder()
            .label(label)
            .shader(shader)
            .bind_group_layout(layouts.frame)
            .bind_group_layout(layouts.voxel)
            .bind_group_layout(layouts.radiance)
            .vertex_entry_point(vertex_entry_point)
            .primitive(wgpu::PrimitiveState {
                topology,
                cull_mode: if topology == wgpu::PrimitiveTopology::TriangleList {
                    Some(wgpu::Face::Back)
                } else {
                    None
                },
                ..Default::default()
            })
            .color_format(SceneColorTarget::FORMAT)
            .blend(if topology == wgpu::PrimitiveTopology::TriangleList {
                wgpu::BlendState::REPLACE
            } else {
                wgpu::BlendState::ALPHA_BLENDING
            })
            .depth_stencil(wgpu::DepthStencilState {
                format: DepthTarget::FORMAT,
                depth_write_enabled: Some(topology == wgpu::PrimitiveTopology::TriangleList),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            })
            .multisample(wgpu::MultisampleState {
                count: MSAA_SAMPLES,
                mask: !0,
                alpha_to_coverage_enabled: false,
            })
            .build()
    }

    fn create_mipmap_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        frame_layout: &wgpu::BindGroupLayout,
        voxel_layout: &wgpu::BindGroupLayout,
        empty_layout: &wgpu::BindGroupLayout,
        mipmap_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        device
            .render_pipeline_builder()
            .label("Voxel Mipmap Debug Pipeline")
            .shader(shader)
            .bind_group_layout(frame_layout)
            .bind_group_layout(voxel_layout)
            .bind_group_layout(empty_layout)
            .bind_group_layout(mipmap_layout)
            .vertex_entry_point("vs_voxel_mipmap")
            .primitive(wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                cull_mode: None,
                ..Default::default()
            })
            .color_format(SceneColorTarget::FORMAT)
            .blend(wgpu::BlendState::ALPHA_BLENDING)
            .depth_stencil(wgpu::DepthStencilState {
                format: DepthTarget::FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            })
            .multisample(wgpu::MultisampleState {
                count: MSAA_SAMPLES,
                mask: !0,
                alpha_to_coverage_enabled: false,
            })
            .build()
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: VoxelDebugPassInput<'_>) {
        if input.brick_count == 0 {
            return;
        }

        let mut render_pass = encoder
            .render_pass_builder()
            .label("Voxel Debug Render Pass")
            .color(input.color_view)
            .resolve(input.resolve_view)
            .load()
            .depth(input.depth_view)
            .load()
            .build();

        render_pass.set_pipeline(&self.brick_pipeline);
        render_pass.set_bind_group(0, input.frame_bind_group, &[]);
        render_pass.set_bind_group(1, input.voxel_bind_group, &[]);
        render_pass.set_bind_group(2, input.radiance_bind_group, &[]);
        render_pass.draw(0..24, 0..input.brick_count);
    }

    pub fn record_occupied_voxels(&self, encoder: &mut wgpu::CommandEncoder, input: VoxelDebugPassInput<'_>) {
        if input.occupied_voxel_count == 0 {
            return;
        }

        let mut render_pass = encoder
            .render_pass_builder()
            .label("Occupied Voxel Debug Render Pass")
            .color(input.color_view)
            .resolve(input.resolve_view)
            .load()
            .depth(input.depth_view)
            .load()
            .build();

        render_pass.set_pipeline(&self.voxel_pipeline);
        render_pass.set_bind_group(0, input.frame_bind_group, &[]);
        render_pass.set_bind_group(1, input.voxel_bind_group, &[]);
        render_pass.set_bind_group(2, input.radiance_bind_group, &[]);
        render_pass.draw(0..24, 0..input.occupied_voxel_count);
    }

    pub fn record_occupied_voxel_faces(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        input: VoxelDebugPassInput<'_>,
    ) {
        if input.visible_face_count == 0 {
            return;
        }

        let mut render_pass = encoder
            .render_pass_builder()
            .label("Occupied Voxel Albedo Render Pass")
            .color(input.color_view)
            .resolve(input.resolve_view)
            .load()
            .depth(input.depth_view)
            .clear(1.0)
            .build();

        render_pass.set_pipeline(&self.voxel_albedo_pipeline);
        render_pass.set_bind_group(0, input.frame_bind_group, &[]);
        render_pass.set_bind_group(1, input.voxel_bind_group, &[]);
        render_pass.set_bind_group(2, input.radiance_bind_group, &[]);
        render_pass.draw(0..6, 0..input.visible_face_count);
    }

    pub fn record_occupied_voxel_normals(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        input: VoxelDebugPassInput<'_>,
    ) {
        if input.visible_face_count == 0 {
            return;
        }

        let mut render_pass = encoder
            .render_pass_builder()
            .label("Occupied Voxel Normal Render Pass")
            .color(input.color_view)
            .resolve(input.resolve_view)
            .load()
            .depth(input.depth_view)
            .clear(1.0)
            .build();

        render_pass.set_pipeline(&self.voxel_normal_pipeline);
        render_pass.set_bind_group(0, input.frame_bind_group, &[]);
        render_pass.set_bind_group(1, input.voxel_bind_group, &[]);
        render_pass.set_bind_group(2, input.radiance_bind_group, &[]);
        render_pass.draw(0..6, 0..input.visible_face_count);
    }

    pub fn record_occupied_voxel_radiance(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        input: VoxelDebugPassInput<'_>,
    ) {
        if input.visible_face_count == 0 {
            return;
        }

        let mut render_pass = encoder
            .render_pass_builder()
            .label("Occupied Voxel Radiance Render Pass")
            .color(input.color_view)
            .resolve(input.resolve_view)
            .load()
            .depth(input.depth_view)
            .clear(1.0)
            .build();

        render_pass.set_pipeline(&self.voxel_radiance_pipeline);
        render_pass.set_bind_group(0, input.frame_bind_group, &[]);
        render_pass.set_bind_group(1, input.voxel_bind_group, &[]);
        render_pass.set_bind_group(2, input.radiance_bind_group, &[]);
        render_pass.draw(0..6, 0..input.visible_face_count);
    }

    pub fn update_level(&self, queue: &wgpu::Queue, level: u32) {
        queue.write_buffer(
            &self.mipmap_settings_buffer,
            0,
            bytemuck::cast_slice(&[level, 0, 0, 0, 0, 0, 0, 0]),
        );
    }

    pub fn record_mips(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        input: VoxelDebugPassInput<'_>,
        instance_count: u32,
    ) {
        if instance_count == 0 {
            return;
        }

        let mut render_pass = encoder
            .render_pass_builder()
            .label("Voxel Mipmap Debug Render Pass")
            .color(input.color_view)
            .resolve(input.resolve_view)
            .load()
            .depth(input.depth_view)
            .clear(1.0)
            .build();

        render_pass.set_pipeline(&self.mipmap_pipeline);
        render_pass.set_bind_group(0, input.frame_bind_group, &[]);
        render_pass.set_bind_group(1, input.voxel_bind_group, &[]);
        render_pass.set_bind_group(2, &self.empty_bind_group, &[]);
        render_pass.set_bind_group(3, &self.mipmap_bind_group, &[]);
        render_pass.draw(0..24, 0..instance_count);
    }
}
