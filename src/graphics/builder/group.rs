pub trait BindGroupExt {
    fn bind_group_builder(&self) -> BindGroupBuilder<'_>;
}

impl BindGroupExt for wgpu::Device {
    fn bind_group_builder(&self) -> BindGroupBuilder<'_> {
        BindGroupBuilder::new(self)
    }
}

struct PendingEntry<'resource> {
    binding: u32,
    resource: wgpu::BindingResource<'resource>,
}

pub struct BindGroupBuilder<'device> {
    device: &'device wgpu::Device,

    label: Option<String>,
    layout: Option<&'device wgpu::BindGroupLayout>,
    entries: Vec<PendingEntry<'device>>,
}

impl<'device> BindGroupBuilder<'device> {
    pub fn new(device: &'device wgpu::Device) -> Self {
        Self {
            device,

            label: None,
            layout: None,
            entries: Vec::new(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn layout(mut self, layout: &'device wgpu::BindGroupLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn resource(mut self, binding: u32, resource: wgpu::BindingResource<'device>) -> Self {
        self.insert(binding, resource);
        self
    }

    pub fn buffer(self, binding: u32, buffer: &'device wgpu::Buffer) -> Self {
        self.resource(binding, buffer.as_entire_binding())
    }

    pub fn texture(self, binding: u32, view: &'device wgpu::TextureView) -> Self {
        self.resource(binding, wgpu::BindingResource::TextureView(view))
    }

    pub fn sampler(self, binding: u32, sampler: &'device wgpu::Sampler) -> Self {
        self.resource(binding, wgpu::BindingResource::Sampler(sampler))
    }

    pub fn build(self) -> wgpu::BindGroup {
        let Self {
            device,
            label,
            layout,
            entries,
        } = self;

        let layout = layout.expect("BindGroupBuilder: layout is required");

        let entries = entries
            .into_iter()
            .map(|entry| wgpu::BindGroupEntry {
                binding: entry.binding,
                resource: entry.resource,
            })
            .collect::<Vec<_>>();

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: label.as_deref(),
            layout,
            entries: &entries,
        })
    }

    fn insert(&mut self, binding: u32, resource: wgpu::BindingResource<'device>) {
        if self.entries.iter().any(|entry| entry.binding == binding) {
            panic!("BindGroupBuilder: duplicate binding {binding}");
        }

        self.entries.push(PendingEntry { binding, resource });
    }
}
