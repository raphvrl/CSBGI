use wesl::include_wesl;

use crate::{
    assets::{MaterialAssets, MeshAssets},
    graphics::{
        GraphicsContext, Vertex,
        builder::{RenderPassExt, RenderPipelineExt},
        targets::ShadowTarget,
    },
};

use super::geometry::MeshDraw;

const SHADOW_DEPTH_BIAS: i32 = 2;
const SHADOW_SLOPE_SCALE: f32 = 4.0;

pub struct ShadowPassInput<'a> {
    pub shadow_view: &'a wgpu::TextureView,
    pub frame_bind_group: &'a wgpu::BindGroup,
    pub object_bind_group: &'a wgpu::BindGroup,
    pub material_assets: &'a MaterialAssets,
    pub mesh_draws: &'a [MeshDraw],
    pub mesh_assets: &'a MeshAssets,
}

pub struct ShadowPass {
    pipeline: wgpu::RenderPipeline,
}

impl ShadowPass {
    pub fn new(
        graphics: &GraphicsContext,
        frame_layout: &wgpu::BindGroupLayout,
        object_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let device = graphics.device();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Shader"),
            source: wgpu::ShaderSource::Wgsl(include_wesl!("shadow").into()),
        });

        let pipeline = device
            .render_pipeline_builder()
            .label("Shadow Pipeline")
            .shader(&shader)
            .bind_group_layout(frame_layout)
            .bind_group_layout(object_layout)
            .bind_group_layout(material_layout)
            .vertex_layout(Vertex::layout())
            .primitive(wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                // Interior and thin wall faces must also be able to cast shadows.
                cull_mode: None,
                ..Default::default()
            })
            .depth_stencil(wgpu::DepthStencilState {
                format: ShadowTarget::FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    constant: SHADOW_DEPTH_BIAS,
                    slope_scale: SHADOW_SLOPE_SCALE,
                    clamp: 0.0,
                },
            })
            .build();

        Self { pipeline }
    }

    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, input: ShadowPassInput<'_>) {
        let mut rpass = encoder
            .render_pass_builder()
            .label("Shadow Render Pass")
            .depth(input.shadow_view)
            .clear(1.0)
            .build();

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, input.frame_bind_group, &[]);
        rpass.set_bind_group(1, input.object_bind_group, &[]);

        for (instance, draw) in input.mesh_draws.iter().enumerate() {
            if let (Some(mesh), Some(material)) = (
                input.mesh_assets.get(draw.mesh),
                input.material_assets.get(draw.material),
            ) {
                rpass.set_bind_group(2, material.bind_group(), &[]);
                mesh.draw(&mut rpass, instance as u32);
            }
        }
    }
}
