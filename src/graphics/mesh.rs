use std::ops::Range;

use wgpu::util::DeviceExt;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};

use super::Frustum;

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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MeshletDebugVertex {
    position: [f32; 3],
    color: [f32; 3],
    uv: [f32; 2],
}

impl MeshletDebugVertex {
    pub const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MeshletDescriptor {
    pub vertex_offset: u32,
    pub triangle_offset: u32,
    pub vertex_count: u32,
    pub triangle_count: u32,
    pub bounds_center: [f32; 3],
    pub bounds_radius: f32,
}

#[derive(Debug, Default)]
pub struct MeshletData {
    pub descriptors: Vec<MeshletDescriptor>,
    pub vertex_indices: Vec<u32>,
    pub triangle_indices: Vec<u8>,
}

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    meshlet_ranges: Vec<Range<u32>>,
    meshlet_bounds: Vec<MeshletBounds>,
    meshlet_debug_vertex_buffer: Option<wgpu::Buffer>,
}

#[derive(Clone, Copy)]
struct MeshletBounds {
    center: Vec3,
    radius: f32,
}

impl Mesh {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        Self::new_with_meshlets(device, vertices, indices, MeshletData::default(), 0)
    }

    pub fn new_with_meshlets(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u32],
        meshlets: MeshletData,
        first_meshlet_id: usize,
    ) -> Self {
        let (meshlet_indices, debug_vertices, meshlet_ranges) =
            expand_meshlets(vertices, &meshlets, first_meshlet_id);
        let meshlet_bounds = meshlets
            .descriptors
            .iter()
            .map(|meshlet| MeshletBounds {
                center: Vec3::from_array(meshlet.bounds_center),
                radius: meshlet.bounds_radius,
            })
            .collect();
        let render_indices = if meshlet_ranges.is_empty() {
            indices
        } else {
            &meshlet_indices
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meshlet Index Buffer"),
            contents: bytemuck::cast_slice(render_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let meshlet_debug_vertex_buffer = (!debug_vertices.is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Meshlet Debug Vertex Buffer"),
                contents: bytemuck::cast_slice(&debug_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });

        let index_count = render_indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            index_count,
            meshlet_ranges,
            meshlet_bounds,
            meshlet_debug_vertex_buffer,
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

        if self.meshlet_ranges.is_empty() {
            rpass.draw_indexed(0..self.index_count, 0, instance..instance + 1);
        } else {
            for range in &self.meshlet_ranges {
                rpass.draw_indexed(range.clone(), 0, instance..instance + 1);
            }
        }
    }

    pub fn draw_culled<'rpass>(
        &'rpass self,
        rpass: &mut wgpu::RenderPass<'rpass>,
        instance: u32,
        model: Mat4,
        frustum: &Frustum,
    ) {
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        if self.meshlet_ranges.is_empty() {
            rpass.draw_indexed(0..self.index_count, 0, instance..instance + 1);
            return;
        }

        let model_scale = max_model_scale(model);
        for (range, bounds) in self.meshlet_ranges.iter().zip(&self.meshlet_bounds) {
            let world_center = model.transform_point3(bounds.center);
            let world_radius = bounds.radius * model_scale;
            if frustum.intersects_sphere(world_center, world_radius) {
                rpass.draw_indexed(range.clone(), 0, instance..instance + 1);
            }
        }
    }

    pub fn draw_meshlets_debug_culled<'rpass>(
        &'rpass self,
        rpass: &mut wgpu::RenderPass<'rpass>,
        instance: u32,
        model: Mat4,
        frustum: &Frustum,
    ) {
        let Some(vertex_buffer) = &self.meshlet_debug_vertex_buffer else {
            return;
        };

        rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
        let model_scale = max_model_scale(model);
        for (range, bounds) in self.meshlet_ranges.iter().zip(&self.meshlet_bounds) {
            let world_center = model.transform_point3(bounds.center);
            let world_radius = bounds.radius * model_scale;
            if frustum.intersects_sphere(world_center, world_radius) {
                rpass.draw(range.clone(), instance..instance + 1);
            }
        }
    }
}

fn max_model_scale(model: Mat4) -> f32 {
    model
        .x_axis
        .truncate()
        .length()
        .max(model.y_axis.truncate().length())
        .max(model.z_axis.truncate().length())
}

fn expand_meshlets(
    vertices: &[Vertex],
    meshlets: &MeshletData,
    first_meshlet_id: usize,
) -> (Vec<u32>, Vec<MeshletDebugVertex>, Vec<Range<u32>>) {
    let mut indices = Vec::new();
    let mut debug_vertices = Vec::new();
    let mut ranges = Vec::with_capacity(meshlets.descriptors.len());

    for (meshlet_index, meshlet) in meshlets.descriptors.iter().enumerate() {
        let start = indices.len() as u32;
        let color = meshlet_color(first_meshlet_id + meshlet_index);

        for triangle_index in 0..meshlet.triangle_count as usize {
            let triangle_start = meshlet.triangle_offset as usize + triangle_index * 3;

            for &local_vertex in &meshlets.triangle_indices[triangle_start..triangle_start + 3] {
                let vertex_index = meshlet.vertex_offset as usize + usize::from(local_vertex);
                let vertex = &vertices[meshlets.vertex_indices[vertex_index] as usize];

                indices.push(meshlets.vertex_indices[vertex_index]);
                debug_vertices.push(MeshletDebugVertex {
                    position: vertex.position.to_array(),
                    color,
                    uv: vertex.uv.to_array(),
                });
            }
        }

        ranges.push(start..indices.len() as u32);
    }

    (indices, debug_vertices, ranges)
}

fn meshlet_color(meshlet_id: usize) -> [f32; 3] {
    let hue = (meshlet_id as f32 * 0.618_034).fract() * 6.0;
    let chroma = 0.72;
    let secondary = chroma * (1.0 - ((hue % 2.0) - 1.0).abs());
    let minimum = 0.18;

    let (red, green, blue) = match hue as u32 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };

    [red + minimum, green + minimum, blue + minimum]
}

fn add_face(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    positions: [Vec3; 4],
    normal: Vec3,
) {
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
    let reference = if normal.y.abs() < 0.999 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let tangent = normal.cross(reference).normalize_or_zero();

    Vec4::new(tangent.x, tangent.y, tangent.z, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_meshlets_with_nonzero_triangle_byte_offset() {
        let vertices = [Vertex {
            position: Vec3::ZERO,
            normal: Vec3::Z,
            uv: Vec2::ZERO,
            tangent: Vec4::X,
        }; 6];
        let meshlets = MeshletData {
            descriptors: vec![
                MeshletDescriptor {
                    vertex_offset: 0,
                    triangle_offset: 0,
                    vertex_count: 3,
                    triangle_count: 1,
                    bounds_center: [0.0; 3],
                    bounds_radius: 0.0,
                },
                MeshletDescriptor {
                    vertex_offset: 3,
                    triangle_offset: 4,
                    vertex_count: 3,
                    triangle_count: 1,
                    bounds_center: [0.0; 3],
                    bounds_radius: 0.0,
                },
            ],
            vertex_indices: vec![0, 1, 2, 3, 4, 5],
            triangle_indices: vec![0, 1, 2, 0, 0, 1, 2, 0],
        };

        let (indices, _, ranges) = expand_meshlets(&vertices, &meshlets, 0);

        assert_eq!(indices, [0, 1, 2, 3, 4, 5]);
        assert_eq!(ranges, [0..3, 3..6]);
    }
}
