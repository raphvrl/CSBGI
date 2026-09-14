use std::num::NonZeroU32;

pub trait BindGroupLayoutExt {
    fn bind_group_layout_builder(&self) -> BindGroupLayoutBuilder<'_>;
}

impl BindGroupLayoutExt for wgpu::Device {
    fn bind_group_layout_builder(&self) -> BindGroupLayoutBuilder<'_> {
        BindGroupLayoutBuilder::new(self)
    }
}

struct LayoutEntry {
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: Option<wgpu::BindingType>,
    count: Option<NonZeroU32>,
}

pub struct BindGroupLayoutBuilder<'device> {
    device: &'device wgpu::Device,

    label: Option<String>,
    entries: Vec<LayoutEntry>,
    active_entry: Option<usize>,
}

impl<'device> BindGroupLayoutBuilder<'device> {
    pub fn new(device: &'device wgpu::Device) -> Self {
        Self {
            device,
            label: None,
            entries: Vec::new(),
            active_entry: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn entry(mut self, binding: u32) -> Self {
        if self.entries.iter().any(|entry| entry.binding == binding) {
            panic!("BindGroupLayoutBuilder: duplicate binding {binding}");
        }

        let index = self.entries.len();

        self.entries.push(LayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::NONE,
            ty: None,
            count: None,
        });

        self.active_entry = Some(index);
        self
    }

    pub fn uniform(mut self) -> Self {
        self.set_type(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        });

        self
    }

    pub fn storage(mut self) -> Self {
        self.set_type(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        });

        self
    }

    pub fn storage_read_only(mut self) -> Self {
        self.set_type(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        });

        self
    }

    pub fn sampler(mut self) -> Self {
        self.set_type(wgpu::BindingType::Sampler(
            wgpu::SamplerBindingType::Filtering,
        ));

        self
    }

    pub fn non_filtering_sampler(mut self) -> Self {
        self.set_type(wgpu::BindingType::Sampler(
            wgpu::SamplerBindingType::NonFiltering,
        ));

        self
    }

    pub fn comparison_sampler(mut self) -> Self {
        self.set_type(wgpu::BindingType::Sampler(
            wgpu::SamplerBindingType::Comparison,
        ));

        self
    }

    pub fn texture(self) -> Self {
        self.texture_with(
            wgpu::TextureSampleType::Float { filterable: true },
            wgpu::TextureViewDimension::D2,
            false,
        )
    }

    pub fn texture_with(
        mut self,
        sample_type: wgpu::TextureSampleType,
        view_dimension: wgpu::TextureViewDimension,
        multisampled: bool,
    ) -> Self {
        self.set_type(wgpu::BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled,
        });

        self
    }

    pub fn visibility(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.active_entry_mut().visibility |= visibility;
        self
    }

    pub fn vertex(self) -> Self {
        self.visibility(wgpu::ShaderStages::VERTEX)
    }

    pub fn fragment(self) -> Self {
        self.visibility(wgpu::ShaderStages::FRAGMENT)
    }

    pub fn compute(self) -> Self {
        self.visibility(wgpu::ShaderStages::COMPUTE)
    }

    pub fn count(mut self, count: NonZeroU32) -> Self {
        self.active_entry_mut().count = Some(count);
        self
    }

    pub fn build(self) -> wgpu::BindGroupLayout {
        let Self {
            device,
            label,
            entries,
            active_entry: _,
        } = self;

        let entries = entries
            .into_iter()
            .map(|entry| {
                let ty = entry.ty.unwrap_or_else(|| {
                    panic!(
                        "BindGroupLayoutBuilder: binding {} has no type",
                        entry.binding
                    )
                });

                if entry.visibility.is_empty() {
                    panic!(
                        "BindGroupLayoutBuilder: binding {} has no visibility",
                        entry.binding
                    );
                }

                wgpu::BindGroupLayoutEntry {
                    binding: entry.binding,
                    visibility: entry.visibility,
                    ty,
                    count: entry.count,
                }
            })
            .collect::<Vec<_>>();

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: label.as_deref(),
            entries: &entries,
        })
    }

    fn set_type(&mut self, ty: wgpu::BindingType) {
        let entry = self.active_entry_mut();

        if entry.ty.is_some() {
            panic!(
                "BindGroupLayoutBuilder: binding {} already has a type",
                entry.binding
            );
        }

        entry.ty = Some(ty);
    }

    fn active_entry_mut(&mut self) -> &mut LayoutEntry {
        let index = self
            .active_entry
            .expect("BindGroupLayoutBuilder: no active entry");

        self.entries
            .get_mut(index)
            .expect("BindGroupLayoutBuilder: invalid active entry")
    }
}
