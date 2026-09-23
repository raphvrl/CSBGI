use wgpu::util::DeviceExt;

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec4,
}

impl Vertex {
    pub const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x2,
        3 => Float32x4
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl Mesh {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }

    pub fn cube(device: &wgpu::Device, size: f32) -> Self {
        let h = size * 0.5;

        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        add_face(
            &mut vertices,
            &mut indices,
            [
                Vec3::new(-h, -h, h),
                Vec3::new(h, -h, h),
                Vec3::new(h, h, h),
                Vec3::new(-h, h, h),
            ],
            Vec3::Z,
        );

        add_face(
            &mut vertices,
            &mut indices,
            [
                Vec3::new(-h, -h, -h),
                Vec3::new(-h, h, -h),
                Vec3::new(h, h, -h),
                Vec3::new(h, -h, -h),
            ],
            -Vec3::Z,
        );

        add_face(
            &mut vertices,
            &mut indices,
            [
                Vec3::new(h, -h, -h),
                Vec3::new(h, h, -h),
                Vec3::new(h, h, h),
                Vec3::new(h, -h, h),
            ],
            Vec3::X,
        );

        add_face(
            &mut vertices,
            &mut indices,
            [
                Vec3::new(-h, -h, -h),
                Vec3::new(-h, -h, h),
                Vec3::new(-h, h, h),
                Vec3::new(-h, h, -h),
            ],
            -Vec3::X,
        );

        add_face(
            &mut vertices,
            &mut indices,
            [
                Vec3::new(-h, h, -h),
                Vec3::new(-h, h, h),
                Vec3::new(h, h, h),
                Vec3::new(h, h, -h),
            ],
            Vec3::Y,
        );

        add_face(
            &mut vertices,
            &mut indices,
            [
                Vec3::new(-h, -h, -h),
                Vec3::new(h, -h, -h),
                Vec3::new(h, -h, h),
                Vec3::new(-h, -h, h),
            ],
            -Vec3::Y,
        );

        Self::new(device, &vertices, &indices)
    }

    pub fn draw<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>, instance: u32) {
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..self.index_count, 0, instance..instance + 1);
    }
}

fn add_face(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, positions: [Vec3; 4], normal: Vec3) {
    let start = vertices.len() as u32;

    vertices.extend(positions.into_iter().map(|position| Vertex {
        position,
        normal,
        uv: Vec2::ZERO,
        tangent: tangent_from_normal(normal),
    }));

    indices.extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
}

fn tangent_from_normal(normal: Vec3) -> Vec4 {
    let reference = if normal.y.abs() < 0.999 { Vec3::Y } else { Vec3::X };
    let tangent = normal.cross(reference).normalize_or_zero();

    Vec4::new(tangent.x, tangent.y, tangent.z, 1.0)
}
