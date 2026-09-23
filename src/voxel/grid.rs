use glam::{IVec3, UVec3, Vec3};

pub const BRICK_SIZE: u32 = 8;
pub const VOXEL_SIZE: f32 = 1.0 / BRICK_SIZE as f32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoxelAddress {
    pub voxel: UVec3,
    pub brick: UVec3,
    pub local: UVec3,
}

#[derive(Clone, Copy, Debug)]
pub struct VoxelGrid {
    pub origin: Vec3,
    pub voxel_size: f32,
    pub dimensions: UVec3,
}

impl VoxelGrid {
    pub fn is_valid(&self) -> bool {
        self.origin.is_finite()
            && self.voxel_size.is_finite()
            && self.voxel_size > 0.0
            && self.dimensions.x > 0
            && self.dimensions.y > 0
            && self.dimensions.z > 0
    }

    pub fn brick_dimensions(&self) -> UVec3 {
        UVec3::new(
            ceil_div(self.dimensions.x, BRICK_SIZE),
            ceil_div(self.dimensions.y, BRICK_SIZE),
            ceil_div(self.dimensions.z, BRICK_SIZE),
        )
    }

    pub fn brick_table_index(&self, brick: UVec3) -> Option<usize> {
        let dimensions = self.brick_dimensions();

        if brick.x >= dimensions.x || brick.y >= dimensions.y || brick.z >= dimensions.z {
            return None;
        }

        let index = (brick.x as usize)
            .checked_add((brick.y as usize).checked_mul(dimensions.x as usize)?)?
            .checked_add(
                (brick.z as usize)
                    .checked_mul((dimensions.x as usize).checked_mul(dimensions.y as usize)?)?,
            )?;

        Some(index)
    }

    pub fn address(&self, voxel: IVec3) -> Option<VoxelAddress> {
        let dimensions = self.dimensions.as_ivec3();

        if voxel.x < 0
            || voxel.y < 0
            || voxel.z < 0
            || voxel.x >= dimensions.x
            || voxel.y >= dimensions.y
            || voxel.z >= dimensions.z
        {
            return None;
        }

        let voxel = voxel.as_uvec3();
        let brick_size = UVec3::splat(BRICK_SIZE);

        Some(VoxelAddress {
            voxel,
            brick: voxel / brick_size,
            local: voxel % brick_size,
        })
    }

    pub fn address_uvec(&self, voxel: UVec3) -> VoxelAddress {
        let brick_size = UVec3::splat(BRICK_SIZE);

        VoxelAddress {
            voxel,
            brick: voxel / brick_size,
            local: voxel % brick_size,
        }
    }

    pub fn voxel_center(&self, voxel: UVec3) -> Vec3 {
        self.voxel_to_world(voxel) + Vec3::splat(self.voxel_size * 0.5)
    }

    pub fn voxel_to_world(&self, voxel: UVec3) -> Vec3 {
        self.origin + voxel.as_vec3() * self.voxel_size
    }

    pub fn world_to_voxel(&self, position: Vec3) -> IVec3 {
        let coordinate = (position - self.origin) / self.voxel_size;

        IVec3::new(
            coordinate.x.floor() as i32,
            coordinate.y.floor() as i32,
            coordinate.z.floor() as i32,
        )
    }
}

fn ceil_div(value: u32, divisor: u32) -> u32 {
    value / divisor + u32::from(!value.is_multiple_of(divisor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brick_dimensions_round_up() {
        let grid = VoxelGrid {
            origin: Vec3::ZERO,
            voxel_size: 1.0,
            dimensions: UVec3::new(256, 257, 263),
        };

        assert_eq!(grid.brick_dimensions(), UVec3::new(32, 33, 33));
    }

    #[test]
    fn brick_table_index_is_flattened_in_z_major_order() {
        let grid = VoxelGrid {
            origin: Vec3::ZERO,
            voxel_size: 1.0,
            dimensions: UVec3::splat(256),
        };

        assert_eq!(grid.brick_table_index(UVec3::new(1, 2, 3)), Some(3137));
    }

    #[test]
    fn brick_table_index_rejects_out_of_bounds_bricks() {
        let grid = VoxelGrid {
            origin: Vec3::ZERO,
            voxel_size: 1.0,
            dimensions: UVec3::splat(256),
        };

        assert_eq!(grid.brick_table_index(UVec3::new(32, 0, 0)), None);
    }

    #[test]
    fn invalid_grid_parameters_are_rejected() {
        let grid = VoxelGrid {
            origin: Vec3::ZERO,
            voxel_size: 0.0,
            dimensions: UVec3::ONE,
        };

        assert!(!grid.is_valid());
    }
}
