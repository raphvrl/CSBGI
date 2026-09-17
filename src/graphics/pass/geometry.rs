use glam::Mat4;
use wesl::include_wesl;

use crate::{
    assets::{
        MaterialAssets, MeshAssets,
        handles::{MaterialHandle, MeshHandle},
    },
    graphics::{
        GraphicsContext, Vertex,
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
    pub object_bind_group: &'a wgpu::BindGroup,
    pub material_assets: &'a MaterialAssets,
    pub mesh_draws: &'a [MeshDraw],
    pub mesh_assets: &'a MeshAssets,
}

pub struct GeometryPass {
    pipeline: wgpu::RenderPipeline,
}

impl GeometryPass {
    pub fn new(
        graphics: &GraphicsContext,
        frame_layout: &wgpu::BindGroupLayout,
        object_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let device = graphics.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Geometry Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("geometry").into()),
        });

        let pipeline = device
            .render_pipeline_builder()
            .label("Geometry Pipeline")
            .shader(&shader)
            .bind_group_layout(frame_layout)
            .bind_group_layout(object_layout)
            .bind_group_layout(material_layout)
            .vertex_layout(Vertex::layout())
            .primitive(wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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

        Self { pipeline }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: GeometryPassInput<'_>) {
        let mut rpass = encoder
            .render_pass_builder()
            .label("Geometry Render Pass")
            .color(input.color_view)
            .resolve(input.resolve_view)
            .load()
            .depth(input.depth_view)
            .clear(1.0)
            .build();

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, input.frame_bind_group, &[]);
        rpass.set_bind_group(1, input.object_bind_group, &[]);

        for (instance, item) in input.mesh_draws.iter().enumerate() {
            if let (Some(mesh), Some(material)) = (
                input.mesh_assets.get(item.mesh),
                input.material_assets.get(item.material),
            ) {
                rpass.set_bind_group(2, material.bind_group(), &[]);
                mesh.draw(&mut rpass, instance as u32);
            }
        }
    }
}
