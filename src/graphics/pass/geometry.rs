use glam::Mat4;
use wesl::include_wesl;

use crate::{
    assets::{
        MaterialAssets, MeshAssets,
        handles::{MaterialHandle, MeshHandle},
    },
    graphics::{
        Frustum, GraphicsContext, MeshletDebugVertex, Vertex,
        builder::{RenderPassExt, RenderPipelineExt},
        targets::{DepthTarget, MSAA_SAMPLES, SceneColorTarget},
    },
};

pub struct MeshDraw {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    pub model: Mat4,
}

pub struct GeometryPassInput<'a> {
    pub color_view: &'a wgpu::TextureView,
    pub resolve_view: &'a wgpu::TextureView,
    pub depth_view: &'a wgpu::TextureView,
    pub frame_bind_group: &'a wgpu::BindGroup,
    pub voxel_sampling_bind_group: &'a wgpu::BindGroup,
    pub object_bind_group: &'a wgpu::BindGroup,
    pub material_assets: &'a MaterialAssets,
    pub mesh_draws: &'a [MeshDraw],
    pub mesh_assets: &'a MeshAssets,
    pub meshlet_debug_enabled: bool,
    pub frustum: &'a Frustum,
}

pub struct GeometryPass {
    single_sided_pipeline: wgpu::RenderPipeline,
    double_sided_pipeline: wgpu::RenderPipeline,
    meshlet_debug_pipeline: wgpu::RenderPipeline,
}

impl GeometryPass {
    pub fn new(
        graphics: &GraphicsContext,
        frame_layout: &wgpu::BindGroupLayout,
        object_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        voxel_sampling_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let device = graphics.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Geometry Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("geometry").into()),
        });

        let meshlet_debug_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Meshlet Debug Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("meshlet_debug").into()),
        });

        let pipeline = |label, cull_mode| {
            device
                .render_pipeline_builder()
                .label(label)
                .shader(&shader)
                .bind_group_layout(frame_layout)
                .bind_group_layout(object_layout)
                .bind_group_layout(material_layout)
                .bind_group_layout(voxel_sampling_layout)
                .vertex_layout(Vertex::layout())
                .primitive(wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode,
                    ..Default::default()
                })
                .color_format(SceneColorTarget::FORMAT)
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
        };

        let meshlet_debug_pipeline = device
            .render_pipeline_builder()
            .label("Meshlet Debug Pipeline")
            .shader(&meshlet_debug_shader)
            .bind_group_layout(frame_layout)
            .bind_group_layout(object_layout)
            .bind_group_layout(material_layout)
            .vertex_layout(MeshletDebugVertex::layout())
            .primitive(wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            })
            .color_format(SceneColorTarget::FORMAT)
            .depth_stencil(wgpu::DepthStencilState {
                format: DepthTarget::FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            })
            .multisample(wgpu::MultisampleState {
                count: MSAA_SAMPLES,
                mask: !0,
                alpha_to_coverage_enabled: false,
            })
            .build();

        Self {
            single_sided_pipeline: pipeline("Geometry Pipeline", Some(wgpu::Face::Back)),
            double_sided_pipeline: pipeline("Double-Sided Geometry Pipeline", None),
            meshlet_debug_pipeline,
        }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: GeometryPassInput<'_>) {
        let mut pass_builder = encoder
            .render_pass_builder()
            .label("Geometry Render Pass")
            .color(input.color_view)
            .resolve(input.resolve_view)
            .load()
            .depth(input.depth_view);

        if input.meshlet_debug_enabled {
            pass_builder = pass_builder.clear(1.0);
        } else {
            pass_builder = pass_builder.load();
        }

        let mut rpass = pass_builder.build();

        rpass.set_bind_group(0, input.frame_bind_group, &[]);
        rpass.set_bind_group(1, input.object_bind_group, &[]);

        for (instance, item) in input.mesh_draws.iter().enumerate() {
            if let (Some(mesh), Some(material)) = (
                input.mesh_assets.get(item.mesh),
                input.material_assets.get(item.material),
            ) {
                if input.meshlet_debug_enabled {
                    rpass.set_pipeline(&self.meshlet_debug_pipeline);
                    rpass.set_bind_group(2, material.bind_group(), &[]);
                    mesh.draw_meshlets_debug_culled(
                        &mut rpass,
                        instance as u32,
                        item.model,
                        input.frustum,
                    );
                    continue;
                }

                rpass.set_bind_group(3, input.voxel_sampling_bind_group, &[]);
                let pipeline = if material.double_sided() {
                    &self.double_sided_pipeline
                } else {
                    &self.single_sided_pipeline
                };
                rpass.set_pipeline(pipeline);
                rpass.set_bind_group(2, material.bind_group(), &[]);
                mesh.draw_culled(&mut rpass, instance as u32, item.model, input.frustum);
            }
        }
    }
}
