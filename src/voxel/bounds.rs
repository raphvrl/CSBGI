use glam::{IVec3, UVec3, Vec3};

use super::{BRICK_SIZE, VoxelGrid};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrickRange {
    pub min: UVec3,
    pub max_exclusive: UVec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoxelRange {
    pub min: UVec3,
    pub max_exclusive: UVec3,
}

impl BrickRange {
    pub fn from_voxel_range(range: VoxelRange) -> Self {
        let brick_size = BRICK_SIZE as i32;

        Self {
            min: UVec3::new(
                range.min.x / BRICK_SIZE,
                range.min.y / BRICK_SIZE,
                range.min.z / BRICK_SIZE,
            ),
            max_exclusive: UVec3::new(
                ceil_div(range.max_exclusive.x as i32, brick_size),
                ceil_div(range.max_exclusive.y as i32, brick_size),
                ceil_div(range.max_exclusive.z as i32, brick_size),
            ),
        }
    }

    pub fn is_empty(self) -> bool {
        self.min.x >= self.max_exclusive.x
            || self.min.y >= self.max_exclusive.y
            || self.min.z >= self.max_exclusive.z
    }

    pub fn iter(self) -> impl Iterator<Item = UVec3> {
        let min = self.min;
        let max = self.max_exclusive;

        (min.z..max.z).flat_map(move |z| {
            (min.y..max.y).flat_map(move |y| (min.x..max.x).map(move |x| UVec3::new(x, y, z)))
        })
    }

    pub fn voxel_count(self, voxel_range: VoxelRange) -> usize {
        self.iter()
            .map(|brick| voxel_count_in_brick(brick, voxel_range))
            .sum()
    }
}

pub fn triangle_brick_range(grid: &VoxelGrid, triangle: [Vec3; 3]) -> Option<BrickRange> {
    triangle_voxel_range(grid, triangle).map(BrickRange::from_voxel_range)
}

pub fn triangle_voxel_range(grid: &VoxelGrid, triangle: [Vec3; 3]) -> Option<VoxelRange> {
    let min_world = triangle[0].min(triangle[1]).min(triangle[2]);
    let max_world = triangle[0].max(triangle[1]).max(triangle[2]);

    let min_voxel_unclamped = grid.world_to_voxel(min_world);
    let max_voxel_unclamped = grid.world_to_voxel(max_world) + IVec3::ONE;

    let grid_dimensions = grid.dimensions.as_ivec3();

    let min_voxel = clamp_ivec3(min_voxel_unclamped, IVec3::ZERO, grid_dimensions);

    let max_voxel_exclusive = clamp_ivec3(max_voxel_unclamped, IVec3::ZERO, grid_dimensions);

    if min_voxel.x >= max_voxel_exclusive.x
        || min_voxel.y >= max_voxel_exclusive.y
        || min_voxel.z >= max_voxel_exclusive.z
    {
        return None;
    }

    Some(VoxelRange {
        min: min_voxel.as_uvec3(),
        max_exclusive: max_voxel_exclusive.as_uvec3(),
    })
}

fn voxel_count_in_brick(brick: UVec3, voxel_range: VoxelRange) -> usize {
    let brick_min = brick * BRICK_SIZE;
    let brick_max = brick_min + UVec3::splat(BRICK_SIZE);

    let min = UVec3::new(
        voxel_range.min.x.max(brick_min.x),
        voxel_range.min.y.max(brick_min.y),
        voxel_range.min.z.max(brick_min.z),
    );

    let max = UVec3::new(
        voxel_range.max_exclusive.x.min(brick_max.x),
        voxel_range.max_exclusive.y.min(brick_max.y),
        voxel_range.max_exclusive.z.min(brick_max.z),
    );

    let size = max - min;
    (size.x * size.y * size.z) as usize
}

fn clamp_ivec3(value: IVec3, min: IVec3, max: IVec3) -> IVec3 {
    IVec3::new(
        value.x.clamp(min.x, max.x),
        value.y.clamp(min.y, max.y),
        value.z.clamp(min.z, max.z),
    )
}

fn ceil_div(value: i32, divisor: i32) -> u32 {
    debug_assert!(value >= 0);
    debug_assert!(divisor > 0);

    ((value + divisor - 1) / divisor) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_grid() -> VoxelGrid {
        VoxelGrid {
            origin: Vec3::ZERO,
            voxel_size: 1.0,
            dimensions: UVec3::splat(16),
        }
    }

    #[test]
    fn triangle_inside_one_brick() {
        let grid = test_grid();

        let triangle = [
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 2.0, 3.0),
            Vec3::new(2.0, 5.0, 3.0),
        ];

        let range = triangle_brick_range(&grid, triangle).unwrap();

        assert_eq!(range.min, UVec3::ZERO);
        assert_eq!(range.max_exclusive, UVec3::ONE);
        assert_eq!(range.iter().count(), 1);
    }

    #[test]
    fn triangle_voxel_range_is_tight() {
        let grid = test_grid();

        let triangle = [
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 2.0, 3.0),
            Vec3::new(2.0, 5.0, 3.0),
        ];

        let range = triangle_voxel_range(&grid, triangle).unwrap();

        assert_eq!(range.min, UVec3::new(1, 2, 3));
        assert_eq!(range.max_exclusive, UVec3::new(5, 6, 4));
    }

    #[test]
    fn triangle_crossing_brick_boundary() {
        let grid = test_grid();

        let triangle = [
            Vec3::new(7.0, 1.0, 1.0),
            Vec3::new(9.0, 1.0, 1.0),
            Vec3::new(8.0, 3.0, 1.0),
        ];

        let range = triangle_brick_range(&grid, triangle).unwrap();

        assert_eq!(range.min, UVec3::new(0, 0, 0));
        assert_eq!(range.max_exclusive, UVec3::new(2, 1, 1));

        let bricks = range.iter().collect::<Vec<_>>();

        assert_eq!(bricks, vec![UVec3::new(0, 0, 0), UVec3::new(1, 0, 0),]);
    }

    #[test]
    fn triangle_outside_grid() {
        let grid = test_grid();

        let triangle = [
            Vec3::new(-10.0, 0.0, 0.0),
            Vec3::new(-8.0, 0.0, 0.0),
            Vec3::new(-9.0, 2.0, 0.0),
        ];

        assert_eq!(triangle_brick_range(&grid, triangle), None);
    }

    #[test]
    fn triangle_partially_inside_grid() {
        let grid = test_grid();

        let triangle = [
            Vec3::new(-2.0, 1.0, 1.0),
            Vec3::new(2.0, 1.0, 1.0),
            Vec3::new(0.0, 4.0, 1.0),
        ];

        let range = triangle_brick_range(&grid, triangle).unwrap();

        assert_eq!(range.min, UVec3::new(0, 0, 0));
        assert_eq!(range.max_exclusive, UVec3::new(1, 1, 1));
    }
}
