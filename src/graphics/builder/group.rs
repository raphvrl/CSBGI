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
    resource: Option<wgpu::BindingResource<'resource>>,
}

pub struct BindGroupBuilder<'device> {
    device: &'device wgpu::Device,

    label: Option<String>,
    layout: Option<&'device wgpu::BindGroupLayout>,
    entries: Vec<PendingEntry<'device>>,
    active_entry: Option<usize>,
}

impl<'device> BindGroupBuilder<'device> {
    pub fn new(device: &'device wgpu::Device) -> Self {
        Self {
            device,

            label: None,
            layout: None,
            entries: Vec::new(),
            active_entry: None,
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

    pub fn entry(mut self, binding: u32) -> Self {
        if self.entries.iter().any(|entry| entry.binding == binding) {
            panic!("BindGroupBuilder: duplicate binding {binding}");
        }

        let index = self.entries.len();

        self.entries.push(PendingEntry {
            binding,
            resource: None,
        });

        self.active_entry = Some(index);
        self
    }

    pub fn resource(mut self, resource: wgpu::BindingResource<'device>) -> Self {
        let entry = self.active_entry_mut();

        if entry.resource.is_some() {
            panic!(
                "BindGroupBuilder: binding {} already has a resource",
                entry.binding
            );
        }

        entry.resource = Some(resource);
        self
    }

    pub fn buffer(self, buffer: &'device wgpu::Buffer) -> Self {
        self.resource(buffer.as_entire_binding())
    }

    pub fn texture(self, view: &'device wgpu::TextureView) -> Self {
        self.resource(wgpu::BindingResource::TextureView(view))
    }

    pub fn sampler(self, sampler: &'device wgpu::Sampler) -> Self {
        self.resource(wgpu::BindingResource::Sampler(sampler))
    }

    pub fn build(self) -> wgpu::BindGroup {
        let Self {
            device,
            label,
            layout,
            entries,
            active_entry: _,
        } = self;

        let layout = layout.expect("BindGroupBuilder: layout is required");

        let entries = entries
            .into_iter()
            .map(|entry| {
                let resource = entry.resource.unwrap_or_else(|| {
                    panic!(
                        "BindGroupBuilder: binding {} has no resource",
                        entry.binding
                    )
                });

                wgpu::BindGroupEntry {
                    binding: entry.binding,
                    resource,
                }
            })
            .collect::<Vec<_>>();

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: label.as_deref(),
            layout,
            entries: &entries,
        })
    }

    fn active_entry_mut(&mut self) -> &mut PendingEntry<'device> {
        let index = self
            .active_entry
            .expect("BindGroupBuilder: no active entry");

        self.entries
            .get_mut(index)
            .expect("BindGroupBuilder: invalid active entry")
    }
}
