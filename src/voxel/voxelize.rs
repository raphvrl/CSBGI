use glam::{UVec3, Vec2, Vec3};
use rayon::prelude::*;

use super::{SparseVoxelStorage, VoxelGrid, VoxelMaterial, pack_normal, sample_albedo, triangle_voxel_range};

const SAT_EPSILON: f32 = 1e-6;

#[derive(Clone, Copy, Debug)]
pub struct TexturedTriangle {
    pub positions: [Vec3; 3],
    pub uvs: [Vec2; 3],
}

pub fn triangle_intersects_box(triangle: [Vec3; 3], box_center: Vec3, half_size: f32) -> bool {
    let vertices = triangle.map(|vertex| vertex - box_center);

    let edges = [
        vertices[1] - vertices[0],
        vertices[2] - vertices[1],
        vertices[0] - vertices[2],
    ];

    let box_half_size = Vec3::splat(half_size);

    let box_axes = [Vec3::X, Vec3::Y, Vec3::Z];

    for axis in box_axes {
        if projections_are_separated(&vertices, axis, box_half_size) {
            return false;
        }
    }

    let triangle_normal = edges[0].cross(edges[1]);

    if projections_are_separated(&vertices, triangle_normal, box_half_size) {
        return false;
    }

    for edge in edges {
        for box_axis in box_axes {
            let axis = edge.cross(box_axis);

            if projections_are_separated(&vertices, axis, box_half_size) {
                return false;
            }
        }
    }

    true
}

fn projections_are_separated(triangle: &[Vec3; 3], axis: Vec3, box_half_size: Vec3) -> bool {
    if axis.length_squared() <= SAT_EPSILON {
        return false;
    }

    let triangle_projection = [
        triangle[0].dot(axis),
        triangle[1].dot(axis),
        triangle[2].dot(axis),
    ];

    let triangle_min = triangle_projection.into_iter().fold(f32::INFINITY, f32::min);

    let triangle_max = triangle_projection.into_iter().fold(f32::NEG_INFINITY, f32::max);

    let box_radius =
        box_half_size.x * axis.x.abs() + box_half_size.y * axis.y.abs() + box_half_size.z * axis.z.abs();

    let epsilon = SAT_EPSILON * (1.0 + box_radius);

    triangle_max < -box_radius - epsilon || triangle_min > box_radius + epsilon
}

pub fn voxelize_triangle(
    grid: &VoxelGrid,
    storage: &mut SparseVoxelStorage,
    triangle: [Vec3; 3],
    albedo: u32,
) {
    let normal = triangle_normal(triangle);

    visit_intersecting_voxels(grid, triangle, |address, _center| {
        storage.set_occupied_with_albedo_and_normal(address.brick.into(), address.local, albedo, normal);
    });
}

pub fn voxelize_triangles(
    grid: &VoxelGrid,
    storage: &mut SparseVoxelStorage,
    triangles: &[[Vec3; 3]],
    albedo: u32,
) {
    let items = triangles
        .iter()
        .copied()
        .map(|triangle| (triangle, albedo))
        .collect();
    voxelize_in_parallel(items, storage, |local_storage, (triangle, albedo)| {
        voxelize_triangle(grid, local_storage, triangle, albedo);
    });
}

pub(super) fn voxelize_triangles_with_albedos(
    grid: &VoxelGrid,
    storage: &mut SparseVoxelStorage,
    triangles: &[[Vec3; 3]],
    albedos: &[u32],
) {
    assert_eq!(
        triangles.len(),
        albedos.len(),
        "each triangle must have one albedo"
    );

    let items = triangles.iter().copied().zip(albedos.iter().copied()).collect();
    voxelize_in_parallel(items, storage, |local_storage, (triangle, albedo)| {
        voxelize_triangle(grid, local_storage, triangle, albedo);
    });
}

pub(super) fn voxelize_textured_triangles(
    grid: &VoxelGrid,
    storage: &mut SparseVoxelStorage,
    triangles: &[TexturedTriangle],
    material: VoxelMaterial<'_>,
) {
    voxelize_in_parallel(triangles.to_vec(), storage, |local_storage, triangle| {
        voxelize_textured_triangle(grid, local_storage, triangle, material);
    });
}

const TRIANGLES_PER_CHUNK: usize = 64;

fn voxelize_in_parallel<T, F>(items: Vec<T>, storage: &mut SparseVoxelStorage, visit: F)
where
    T: Send + Sync + Copy,
    F: Fn(&mut SparseVoxelStorage, T) + Send + Sync,
{
    let partial_storages = items
        .par_chunks(TRIANGLES_PER_CHUNK)
        .map(|chunk| {
            let mut local_storage = SparseVoxelStorage::new();
            for &item in chunk {
                visit(&mut local_storage, item);
            }
            local_storage
        })
        .collect::<Vec<_>>();

    for partial_storage in partial_storages {
        storage.merge_from(partial_storage);
    }
}

fn voxelize_textured_triangle(
    grid: &VoxelGrid,
    storage: &mut SparseVoxelStorage,
    triangle: TexturedTriangle,
    material: VoxelMaterial<'_>,
) {
    let normal = triangle_normal(triangle.positions);

    visit_intersecting_voxels(grid, triangle.positions, |address, center| {
        let uv = uv_at_closest_point(&triangle, center);
        storage.set_occupied_with_albedo_and_normal(
            address.brick.into(),
            address.local,
            sample_albedo(material, uv),
            normal,
        );
    });
}

fn visit_intersecting_voxels(
    grid: &VoxelGrid,
    triangle: [Vec3; 3],
    mut visit: impl FnMut(super::VoxelAddress, Vec3),
) {
    let Some(voxel_range) = triangle_voxel_range(grid, triangle) else {
        return;
    };

    let half_voxel_size = grid.voxel_size * 0.5;
    for global_z in voxel_range.min.z..voxel_range.max_exclusive.z {
        for global_y in voxel_range.min.y..voxel_range.max_exclusive.y {
            for global_x in voxel_range.min.x..voxel_range.max_exclusive.x {
                let global = UVec3::new(global_x, global_y, global_z);
                let center = grid.voxel_center(global);

                if triangle_intersects_box(triangle, center, half_voxel_size) {
                    visit(grid.address_uvec(global), center);
                }
            }
        }
    }
}

fn triangle_normal(triangle: [Vec3; 3]) -> u32 {
    pack_normal((triangle[1] - triangle[0]).cross(triangle[2] - triangle[0]))
}

fn uv_at_closest_point(triangle: &TexturedTriangle, point: Vec3) -> Vec2 {
    let weights = closest_barycentric(triangle.positions, point);

    triangle.uvs[0] * weights[0] + triangle.uvs[1] * weights[1] + triangle.uvs[2] * weights[2]
}

fn closest_barycentric(triangle: [Vec3; 3], point: Vec3) -> [f32; 3] {
    let [a, b, c] = triangle;
    let ab = b - a;
    let ac = c - a;
    let ap = point - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);

    if d1 <= 0.0 && d2 <= 0.0 {
        return [1.0, 0.0, 0.0];
    }

    let bp = point - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);

    if d3 >= 0.0 && d4 <= d3 {
        return [0.0, 1.0, 0.0];
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [1.0 - v, v, 0.0];
    }

    let cp = point - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);

    if d6 >= 0.0 && d5 <= d6 {
        return [0.0, 0.0, 1.0];
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [1.0 - w, 0.0, w];
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [0.0, 1.0 - w, w];
    }

    let denominator = va + vb + vc;
    if denominator.abs() <= SAT_EPSILON {
        return [1.0, 0.0, 0.0];
    }

    let denominator = 1.0 / denominator;
    let v = vb * denominator;
    let w = vc * denominator;

    [1.0 - v - w, v, w]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::BrickKey;

    fn test_grid() -> VoxelGrid {
        VoxelGrid {
            origin: Vec3::ZERO,
            voxel_size: 1.0,
            dimensions: UVec3::splat(16),
        }
    }

    #[test]
    fn triangle_intersects_voxel() {
        let triangle = [
            Vec3::new(0.1, 0.1, 0.5),
            Vec3::new(0.9, 0.1, 0.5),
            Vec3::new(0.1, 0.9, 0.5),
        ];

        let intersects = triangle_intersects_box(triangle, Vec3::new(0.5, 0.5, 0.5), 0.5);

        assert!(intersects);
    }

    #[test]
    fn triangle_does_not_intersect_distant_voxel() {
        let triangle = [
            Vec3::new(0.1, 0.1, 0.1),
            Vec3::new(0.9, 0.1, 0.1),
            Vec3::new(0.1, 0.9, 0.1),
        ];

        let intersects = triangle_intersects_box(triangle, Vec3::new(5.5, 5.5, 5.5), 0.5);

        assert!(!intersects);
    }

    #[test]
    fn voxelize_triangle_marks_occupied_voxel() {
        let grid = test_grid();
        let mut storage = SparseVoxelStorage::new();

        let triangle = [
            Vec3::new(0.1, 0.1, 0.5),
            Vec3::new(0.9, 0.1, 0.5),
            Vec3::new(0.1, 0.9, 0.5),
        ];

        voxelize_triangle(&grid, &mut storage, triangle, 0x1122_3344);

        let brick_key = BrickKey::new(0, 0, 0);
        let brick = storage.brick(brick_key).unwrap();

        assert!(brick.is_occupied(UVec3::new(0, 0, 0)));
        assert_eq!(brick.albedo(UVec3::new(0, 0, 0)), Some(0x1122_3344));
    }

    #[test]
    fn triangle_outside_grid_does_not_allocate_bricks() {
        let grid = test_grid();
        let mut storage = SparseVoxelStorage::new();

        let triangle = [
            Vec3::new(-10.0, 0.0, 0.0),
            Vec3::new(-8.0, 0.0, 0.0),
            Vec3::new(-9.0, 2.0, 0.0),
        ];

        voxelize_triangle(&grid, &mut storage, triangle, 0);

        assert_eq!(storage.brick_count(), 0);
    }

    #[test]
    fn overlapping_triangles_keep_the_first_input_albedo() {
        let grid = test_grid();
        let mut storage = SparseVoxelStorage::new();
        let triangle = [
            Vec3::new(0.1, 0.1, 0.5),
            Vec3::new(0.9, 0.1, 0.5),
            Vec3::new(0.1, 0.9, 0.5),
        ];

        voxelize_triangles_with_albedos(
            &grid,
            &mut storage,
            &[triangle, triangle],
            &[0x1122_3344, 0xaabb_ccdd],
        );

        assert_eq!(
            storage
                .brick(BrickKey::new(0, 0, 0))
                .and_then(|brick| brick.albedo(UVec3::ZERO)),
            Some(0x1122_3344)
        );
    }
}
