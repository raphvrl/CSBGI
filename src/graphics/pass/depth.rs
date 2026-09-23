use wesl::include_wesl;

use crate::{
    assets::{MaterialAssets, MeshAssets},
    graphics::{
        Frustum, GraphicsContext, Vertex,
        builder::{RenderPassExt, RenderPipelineExt},
        targets::{DepthTarget, MSAA_SAMPLES, SceneColorTarget},
    },
};

use super::geometry::MeshDraw;

pub struct DepthPrepassInput<'a> {
    pub color_view: &'a wgpu::TextureView,
    pub depth_view: &'a wgpu::TextureView,
    pub frame_bind_group: &'a wgpu::BindGroup,
    pub object_bind_group: &'a wgpu::BindGroup,
    pub material_assets: &'a MaterialAssets,
    pub mesh_draws: &'a [MeshDraw],
    pub mesh_assets: &'a MeshAssets,
    pub frustum: &'a Frustum,
}

pub struct DepthPrepass {
    single_sided_pipeline: wgpu::RenderPipeline,
    double_sided_pipeline: wgpu::RenderPipeline,
}

impl DepthPrepass {
    pub fn new(
        graphics: &GraphicsContext,
        frame_layout: &wgpu::BindGroupLayout,
        object_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let device = graphics.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Depth Prepass Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("depth").into()),
        });

        let pipeline = |label, cull_mode| {
            device
                .render_pipeline_builder()
                .label(label)
                .shader(&shader)
                .bind_group_layout(frame_layout)
                .bind_group_layout(object_layout)
                .bind_group_layout(material_layout)
                .vertex_layout(Vertex::layout())
                .primitive(wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode,
                    ..Default::default()
                })
                .color_format(SceneColorTarget::FORMAT)
                .write_mask(wgpu::ColorWrites::empty())
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
                .build()
        };

        Self {
            single_sided_pipeline: pipeline("Depth Prepass Pipeline", Some(wgpu::Face::Back)),
            double_sided_pipeline: pipeline("Double-Sided Depth Prepass Pipeline", None),
        }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: DepthPrepassInput<'_>) {
        let mut render_pass = encoder
            .render_pass_builder()
            .label("Depth Prepass Render Pass")
            .color(input.color_view)
            .load()
            .depth(input.depth_view)
            .clear(1.0)
            .build();

        render_pass.set_bind_group(0, input.frame_bind_group, &[]);
        render_pass.set_bind_group(1, input.object_bind_group, &[]);

        for (instance, draw) in input.mesh_draws.iter().enumerate() {
            if let (Some(mesh), Some(material)) = (
                input.mesh_assets.get(draw.mesh),
                input.material_assets.get(draw.material),
            ) {
                let pipeline = if material.double_sided() {
                    &self.double_sided_pipeline
                } else {
                    &self.single_sided_pipeline
                };
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(2, material.bind_group(), &[]);
                mesh.draw_culled(&mut render_pass, instance as u32, draw.model, input.frustum);
            }
        }
    }
}
