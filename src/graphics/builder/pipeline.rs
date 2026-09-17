pub trait RenderPipelineExt {
    fn render_pipeline_builder(&self) -> RenderPipelineBuilder<'_>;
}

impl RenderPipelineExt for wgpu::Device {
    fn render_pipeline_builder(&self) -> RenderPipelineBuilder<'_> {
        RenderPipelineBuilder::new(self)
    }
}

pub struct RenderPipelineBuilder<'device> {
    device: &'device wgpu::Device,

    label: Option<String>,
    shader: Option<&'device wgpu::ShaderModule>,
    layout: Option<&'device wgpu::PipelineLayout>,
    bind_group_layouts: Vec<Option<&'device wgpu::BindGroupLayout>>,

    vertex_layouts: Vec<Option<wgpu::VertexBufferLayout<'device>>>,

    vertex_entry_point: String,
    fragment_entry_point: Option<String>,

    color_format: Option<wgpu::TextureFormat>,
    blend: Option<wgpu::BlendState>,
    write_mask: wgpu::ColorWrites,

    primitive: wgpu::PrimitiveState,
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
}

impl<'device> RenderPipelineBuilder<'device> {
    pub fn new(device: &'device wgpu::Device) -> Self {
        Self {
            device,

            label: None,
            shader: None,
            layout: None,
            bind_group_layouts: Vec::new(),

            vertex_layouts: Vec::new(),
            vertex_entry_point: "vs_main".to_string(),
            fragment_entry_point: Some("fs_main".to_string()),

            color_format: None,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,

            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn shader(mut self, shader: &'device wgpu::ShaderModule) -> Self {
        self.shader = Some(shader);
        self
    }

    pub fn layout(mut self, layout: &'device wgpu::PipelineLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn bind_group_layout(mut self, layout: &'device wgpu::BindGroupLayout) -> Self {
        self.bind_group_layouts.push(Some(layout));
        self
    }

    pub fn vertex_layout(mut self, layout: wgpu::VertexBufferLayout<'device>) -> Self {
        self.vertex_layouts.push(Some(layout));
        self
    }

    pub fn vertex_entry_point(mut self, entry_point: impl Into<String>) -> Self {
        self.vertex_entry_point = entry_point.into();
        self
    }

    pub fn fragment_entry_point(mut self, entry_point: impl Into<String>) -> Self {
        self.fragment_entry_point = Some(entry_point.into());
        self
    }

    pub fn without_fragment(mut self) -> Self {
        self.fragment_entry_point = None;
        self
    }

    pub fn color_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.color_format = Some(format);
        self
    }

    pub fn blend(mut self, blend: wgpu::BlendState) -> Self {
        self.blend = Some(blend);
        self
    }

    pub fn write_mask(mut self, write_mask: wgpu::ColorWrites) -> Self {
        self.write_mask = write_mask;
        self
    }

    pub fn primitive(mut self, primitive: wgpu::PrimitiveState) -> Self {
        self.primitive = primitive;
        self
    }

    pub fn depth_stencil(mut self, depth_stencil: wgpu::DepthStencilState) -> Self {
        self.depth_stencil = Some(depth_stencil);
        self
    }

    pub fn multisample(mut self, multisample: wgpu::MultisampleState) -> Self {
        self.multisample = multisample;
        self
    }

    pub fn build(mut self) -> wgpu::RenderPipeline {
        let shader = self
            .shader
            .expect("RenderPipelineBuilder: shader is required");

        let generated_layout = if self.layout.is_none() {
            Some(
                self.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: self.label.as_deref(),
                        bind_group_layouts: &self.bind_group_layouts,
                        immediate_size: 0,
                    }),
            )
        } else {
            None
        };

        let layout = self.layout.or(generated_layout.as_ref());

        let targets = self.color_format.map(|format| {
            [Some(wgpu::ColorTargetState {
                format,
                blend: self.blend,
                write_mask: self.write_mask,
            })]
        });

        let fragment = self.fragment_entry_point.as_deref().map(|entry_point| {
            let targets = targets
                .as_ref()
                .map_or(&[][..], |targets| targets.as_slice());

            wgpu::FragmentState {
                module: shader,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                targets,
            }
        });

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: self.label.as_deref(),
                layout,
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some(self.vertex_entry_point.as_str()),
                    compilation_options: Default::default(),
                    buffers: &self.vertex_layouts,
                },
                fragment,
                primitive: self.primitive,
                depth_stencil: self.depth_stencil.take(),
                multisample: self.multisample,
                multiview_mask: None,
                cache: None,
            })
    }
}
