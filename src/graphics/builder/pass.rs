pub trait RenderPassExt {
    fn render_pass_builder(&mut self) -> RenderPassBuilder<'_>;
}

impl RenderPassExt for wgpu::CommandEncoder {
    fn render_pass_builder(&mut self) -> RenderPassBuilder<'_> {
        RenderPassBuilder::new(self)
    }
}

#[derive(Clone, Copy)]
enum ActiveAttachment {
    Color(usize),
    Depth,
}

pub enum ClearValue {
    Color(wgpu::Color),
    Depth(f32),
    Stencil(u32),
}

impl From<wgpu::Color> for ClearValue {
    fn from(value: wgpu::Color) -> Self {
        Self::Color(value)
    }
}

impl From<f32> for ClearValue {
    fn from(value: f32) -> Self {
        Self::Depth(value)
    }
}

impl From<u32> for ClearValue {
    fn from(value: u32) -> Self {
        Self::Stencil(value)
    }
}

pub struct RenderPassBuilder<'encoder> {
    encoder: &'encoder mut wgpu::CommandEncoder,

    label: Option<String>,
    color_attachments: Vec<Option<wgpu::RenderPassColorAttachment<'encoder>>>,
    depth_stencil_attachment: Option<wgpu::RenderPassDepthStencilAttachment<'encoder>>,
    active_attachment: Option<ActiveAttachment>,
}

impl<'encoder> RenderPassBuilder<'encoder> {
    pub fn new(encoder: &'encoder mut wgpu::CommandEncoder) -> Self {
        Self {
            encoder,

            label: None,
            color_attachments: Vec::new(),
            depth_stencil_attachment: None,
            active_attachment: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn color(mut self, view: &'encoder wgpu::TextureView) -> Self {
        let index = self.color_attachments.len();

        self.color_attachments
            .push(Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: default_operations(),
            }));

        self.active_attachment = Some(ActiveAttachment::Color(index));
        self
    }

    pub fn depth(mut self, view: &'encoder wgpu::TextureView) -> Self {
        self.depth_stencil_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(default_operations()),
            stencil_ops: None,
        });
        self.active_attachment = Some(ActiveAttachment::Depth);
        self
    }

    pub fn load(mut self) -> Self {
        match self.active_attachment {
            Some(ActiveAttachment::Color(index)) => {
                self.color_attachment_mut(index).ops.load = wgpu::LoadOp::Load;
            }
            Some(ActiveAttachment::Depth) => {
                let attachment = self.active_depth_attachment_mut();

                if let Some(ops) = &mut attachment.depth_ops {
                    ops.load = wgpu::LoadOp::Load;
                }

                if let Some(ops) = &mut attachment.stencil_ops {
                    ops.load = wgpu::LoadOp::Load;
                }
            }
            None => panic!("RenderPassBuilder: no active attachment"),
        }

        self
    }

    pub fn clear(mut self, value: impl Into<ClearValue>) -> Self {
        match (self.active_attachment, value.into()) {
            (Some(ActiveAttachment::Color(index)), ClearValue::Color(color)) => {
                self.color_attachment_mut(index).ops.load = wgpu::LoadOp::Clear(color);
            }
            (Some(ActiveAttachment::Depth), ClearValue::Depth(depth)) => {
                self.active_depth_attachment_mut()
                    .depth_ops
                    .get_or_insert(default_operations())
                    .load = wgpu::LoadOp::Clear(depth);
            }
            (Some(ActiveAttachment::Depth), ClearValue::Stencil(stencil)) => {
                self.active_depth_attachment_mut()
                    .stencil_ops
                    .get_or_insert(default_operations())
                    .load = wgpu::LoadOp::Clear(stencil);
            }
            (Some(ActiveAttachment::Color(_)), _)
            | (Some(ActiveAttachment::Depth), ClearValue::Color(_)) => {
                panic!("RenderPassBuilder: invalid clear value for active attachment")
            }
            (None, _) => {
                panic!("RenderPassBuilder: no active attachment")
            }
        }

        self
    }

    pub fn store(mut self) -> Self {
        match self.active_attachment {
            Some(ActiveAttachment::Color(index)) => {
                self.color_attachment_mut(index).ops.store = wgpu::StoreOp::Store;
            }
            Some(ActiveAttachment::Depth) => {
                let attachment = self.active_depth_attachment_mut();

                if let Some(ops) = &mut attachment.depth_ops {
                    ops.store = wgpu::StoreOp::Store;
                }

                if let Some(ops) = &mut attachment.stencil_ops {
                    ops.store = wgpu::StoreOp::Store;
                }
            }
            None => panic!("RenderPassBuilder: no active attachment"),
        }

        self
    }

    pub fn discard(mut self) -> Self {
        match self.active_attachment {
            Some(ActiveAttachment::Color(index)) => {
                self.color_attachment_mut(index).ops.store = wgpu::StoreOp::Discard;
            }
            Some(ActiveAttachment::Depth) => {
                let attachment = self.active_depth_attachment_mut();

                if let Some(ops) = &mut attachment.depth_ops {
                    ops.store = wgpu::StoreOp::Discard;
                }

                if let Some(ops) = &mut attachment.stencil_ops {
                    ops.store = wgpu::StoreOp::Discard;
                }
            }
            None => panic!("RenderPassBuilder: no active attachment"),
        }

        self
    }

    pub fn stencil_load(mut self) -> Self {
        self.active_depth_attachment_mut()
            .stencil_ops
            .get_or_insert(default_operations())
            .load = wgpu::LoadOp::Load;

        self
    }

    pub fn stencil_store(mut self) -> Self {
        self.active_depth_attachment_mut()
            .stencil_ops
            .get_or_insert(default_operations())
            .store = wgpu::StoreOp::Store;

        self
    }

    pub fn stencil_discard(mut self) -> Self {
        self.active_depth_attachment_mut()
            .stencil_ops
            .get_or_insert(default_operations())
            .store = wgpu::StoreOp::Discard;

        self
    }

    pub fn build(self) -> wgpu::RenderPass<'encoder> {
        let Self {
            encoder,
            label,
            color_attachments,
            depth_stencil_attachment,
            active_attachment: _,
        } = self;

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: label.as_deref(),
            color_attachments: &color_attachments,
            depth_stencil_attachment,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        })
    }

    fn color_attachment_mut(
        &mut self,
        index: usize,
    ) -> &mut wgpu::RenderPassColorAttachment<'encoder> {
        self.color_attachments[index]
            .as_mut()
            .expect("RenderPassBuilder: invalid color attachment index")
    }

    fn active_depth_attachment_mut(
        &mut self,
    ) -> &mut wgpu::RenderPassDepthStencilAttachment<'encoder> {
        match self.active_attachment {
            Some(ActiveAttachment::Depth) => {}
            Some(ActiveAttachment::Color(_)) => {
                panic!("RenderPassBuilder: active attachment is not a depth attachment")
            }
            None => panic!("RenderPassBuilder: no active attachment"),
        }

        self.depth_stencil_attachment
            .as_mut()
            .expect("RenderPassBuilder: depth attachment is required")
    }
}

fn default_operations<T>() -> wgpu::Operations<T> {
    wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: wgpu::StoreOp::Store,
    }
}
