use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use crate::graphics::{
    GraphicsContext,
    builder::{BindGroupExt, BindGroupLayoutExt},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ObjectUniform {
    model: Mat4,
}

pub struct ObjectGpu {
    layout: wgpu::BindGroupLayout,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: usize,
}

impl ObjectGpu {
    pub fn new(graphics: &GraphicsContext) -> Self {
        let device = graphics.device();

        #[rustfmt::skip]
        let layout = device
            .bind_group_layout_builder()
            .label("Object Bind Group Layout")
            .entry(0)
                .storage()
                .read_only()
                .vertex()
            .build();

        let capacity = 1;
        let buffer = create_buffer(device, capacity);
        let bind_group = create_bind_group(device, &layout, &buffer);

        Self {
            layout,
            buffer,
            bind_group,
            capacity,
        }
    }

    pub fn update(&mut self, graphics: &GraphicsContext, models: &[Mat4]) {
        self.ensure_capacity(graphics, models.len());

        if models.is_empty() {
            return;
        }

        let objects = models
            .iter()
            .copied()
            .map(|model| ObjectUniform { model })
            .collect::<Vec<_>>();

        graphics
            .queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&objects));
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn ensure_capacity(&mut self, graphics: &GraphicsContext, required: usize) {
        if required <= self.capacity {
            return;
        }

        self.capacity = required.next_power_of_two();
        self.buffer = create_buffer(graphics.device(), self.capacity);
        self.bind_group = create_bind_group(graphics.device(), &self.layout, &self.buffer);
    }
}

fn create_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Object Storage Buffer"),
        size: (capacity * std::mem::size_of::<ObjectUniform>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device
        .bind_group_builder()
        .label("Object Bind Group")
        .layout(layout)
        .buffer(0, buffer)
        .build()
}
