use glam::UVec3;

use super::{BRICK_SIZE, DEFAULT_NORMAL};

pub const OCCUPANCY_WORDS: usize = 16;
pub const VOXELS_PER_BRICK: usize = BRICK_SIZE as usize * BRICK_SIZE as usize * BRICK_SIZE as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BrickKey {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl BrickKey {
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub fn as_uvec3(self) -> UVec3 {
        UVec3::new(self.x, self.y, self.z)
    }

    pub fn adjacent(self, local: UVec3, delta: [i32; 3]) -> Option<(Self, UVec3)> {
        let mut key = self;
        let mut coordinates = [local.x as i32, local.y as i32, local.z as i32];

        for axis in 0..3 {
            coordinates[axis] += delta[axis];

            if coordinates[axis] < 0 {
                let coordinate = match axis {
                    0 => &mut key.x,
                    1 => &mut key.y,
                    _ => &mut key.z,
                };

                if *coordinate == 0 {
                    return None;
                }

                *coordinate -= 1;
                coordinates[axis] = BRICK_SIZE as i32 - 1;
            } else if coordinates[axis] >= BRICK_SIZE as i32 {
                let coordinate = match axis {
                    0 => &mut key.x,
                    1 => &mut key.y,
                    _ => &mut key.z,
                };

                *coordinate = coordinate.checked_add(1)?;
                coordinates[axis] = 0;
            }
        }

        Some((
            key,
            UVec3::new(
                coordinates[0] as u32,
                coordinates[1] as u32,
                coordinates[2] as u32,
            ),
        ))
    }
}

impl From<UVec3> for BrickKey {
    fn from(value: UVec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Brick {
    pub occupancy: [u32; OCCUPANCY_WORDS],
    pub albedo: [u32; VOXELS_PER_BRICK],
    pub normal: [u32; VOXELS_PER_BRICK],
}

impl Brick {
    pub fn new() -> Self {
        Self {
            occupancy: [0; OCCUPANCY_WORDS],
            albedo: [0; VOXELS_PER_BRICK],
            normal: [DEFAULT_NORMAL; VOXELS_PER_BRICK],
        }
    }

    pub(crate) fn is_valid_local(local: UVec3) -> bool {
        local_index(local).is_some()
    }

    pub fn set_occupied(&mut self, local: UVec3) -> bool {
        let Some(bit_index) = local_index(local) else {
            return false;
        };

        let word_index = bit_index / 32;
        let bit = 1 << (bit_index % 32);
        self.occupancy[word_index] |= bit;
        true
    }

    pub fn set_occupied_with_albedo(&mut self, local: UVec3, albedo: u32) -> bool {
        self.set_occupied_with_albedo_and_normal(local, albedo, DEFAULT_NORMAL)
    }

    pub fn set_occupied_with_albedo_and_normal(&mut self, local: UVec3, albedo: u32, normal: u32) -> bool {
        let Some(bit_index) = local_index(local) else {
            return false;
        };

        let word_index = bit_index / 32;
        let bit = 1 << (bit_index % 32);

        if self.occupancy[word_index] & bit == 0 {
            self.albedo[bit_index] = albedo;
            self.normal[bit_index] = normal;
            self.occupancy[word_index] |= bit;
        }

        true
    }

    pub fn is_occupied(&self, local: UVec3) -> bool {
        let Some(bit_index) = local_index(local) else {
            return false;
        };

        let word_index = bit_index / 32;
        let bit = 1 << (bit_index % 32);
        self.occupancy[word_index] & bit != 0
    }

    pub fn albedo(&self, local: UVec3) -> Option<u32> {
        Some(self.albedo[local_index(local)?])
    }

    pub fn normal(&self, local: UVec3) -> Option<u32> {
        Some(self.normal[local_index(local)?])
    }

    pub(crate) fn occupied_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.occupancy
            .iter()
            .copied()
            .enumerate()
            .flat_map(|(word_index, mut bits)| {
                std::iter::from_fn(move || {
                    if bits == 0 {
                        return None;
                    }

                    let bit_offset = bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    Some(word_index * 32 + bit_offset)
                })
            })
    }

    pub(crate) fn local_from_index(index: usize) -> Option<UVec3> {
        if index >= VOXELS_PER_BRICK {
            return None;
        }

        let brick_size = BRICK_SIZE as usize;
        Some(UVec3::new(
            (index % brick_size) as u32,
            ((index / brick_size) % brick_size) as u32,
            (index / (brick_size * brick_size)) as u32,
        ))
    }
}

impl Default for Brick {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn local_index(local: UVec3) -> Option<usize> {
    if local.x >= BRICK_SIZE || local.y >= BRICK_SIZE || local.z >= BRICK_SIZE {
        return None;
    }

    Some((local.x + local.y * BRICK_SIZE + local.z * BRICK_SIZE * BRICK_SIZE) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_index_round_trips_every_voxel() {
        for index in 0..VOXELS_PER_BRICK {
            assert_eq!(local_index(Brick::local_from_index(index).unwrap()), Some(index));
        }
    }

    #[test]
    fn occupied_indices_are_sorted_and_unique() {
        let mut brick = Brick::new();
        brick.set_occupied(UVec3::new(0, 0, 0));
        brick.set_occupied(UVec3::new(7, 7, 7));

        assert_eq!(brick.occupied_indices().collect::<Vec<_>>(), vec![0, 511]);
    }
}
