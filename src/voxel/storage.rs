use std::collections::BTreeMap;

use glam::UVec3;

use super::{BrickKey, DEFAULT_NORMAL, brick::Brick};

#[derive(Default)]
pub struct SparseVoxelStorage {
    bricks: BTreeMap<BrickKey, Brick>,
}

impl SparseVoxelStorage {
    pub fn new() -> Self {
        Self::default()
    }

    fn ensure_brick(&mut self, key: BrickKey) -> &mut Brick {
        self.bricks.entry(key).or_default()
    }

    pub fn brick(&self, key: BrickKey) -> Option<&Brick> {
        self.bricks.get(&key)
    }

    pub fn set_occupied(&mut self, key: BrickKey, local: UVec3) -> bool {
        if !Brick::is_valid_local(local) {
            return false;
        }

        self.ensure_brick(key).set_occupied(local)
    }

    pub fn set_occupied_with_albedo(&mut self, key: BrickKey, local: UVec3, albedo: u32) -> bool {
        self.set_occupied_with_albedo_and_normal(key, local, albedo, DEFAULT_NORMAL)
    }

    pub fn set_occupied_with_albedo_and_normal(
        &mut self,
        key: BrickKey,
        local: UVec3,
        albedo: u32,
        normal: u32,
    ) -> bool {
        if !Brick::is_valid_local(local) {
            return false;
        }

        self.ensure_brick(key)
            .set_occupied_with_albedo_and_normal(local, albedo, normal)
    }

    pub fn brick_count(&self) -> usize {
        self.bricks.len()
    }

    pub fn iter_bricks(&self) -> impl Iterator<Item = (BrickKey, &Brick)> {
        self.bricks.iter().map(|(&key, brick)| (key, brick))
    }

    pub(crate) fn merge_from(&mut self, other: Self) {
        for (key, brick) in other.iter_bricks() {
            let target = self.ensure_brick(key);

            for (word_index, source_word) in brick.occupancy.iter().copied().enumerate() {
                let target_word = &mut target.occupancy[word_index];
                let new_bits = source_word & !*target_word;
                *target_word |= source_word;

                let mut bits = new_bits;
                while bits != 0 {
                    let bit_offset = bits.trailing_zeros() as usize;
                    let voxel_index = word_index * 32 + bit_offset;
                    target.albedo[voxel_index] = brick.albedo[voxel_index];
                    target.normal[voxel_index] = brick.normal[voxel_index];
                    bits &= bits - 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::brick::{OCCUPANCY_WORDS, VOXELS_PER_BRICK};

    #[test]
    fn new_brick_is_empty() {
        let brick = Brick::new();

        assert_eq!(brick.occupancy, [0; OCCUPANCY_WORDS]);
        assert_eq!(brick.albedo, [0; VOXELS_PER_BRICK]);
    }

    #[test]
    fn voxel_can_be_marked_as_occupied() {
        let mut brick = Brick::new();

        assert!(brick.set_occupied(UVec3::new(3, 2, 1)));
        assert!(brick.is_occupied(UVec3::new(3, 2, 1)));
        assert!(!brick.is_occupied(UVec3::new(3, 2, 2)));
    }

    #[test]
    fn occupied_voxel_keeps_its_first_albedo() {
        let mut brick = Brick::new();
        let local = UVec3::new(3, 2, 1);

        assert!(brick.set_occupied_with_albedo(local, 0x1122_3344));
        assert!(brick.set_occupied_with_albedo(local, 0xaabb_ccdd));

        assert_eq!(brick.albedo(local), Some(0x1122_3344));
    }

    #[test]
    fn invalid_local_coordinate_is_rejected() {
        let mut brick = Brick::new();

        assert!(!brick.set_occupied(UVec3::new(8, 0, 0)));
        assert!(!brick.is_occupied(UVec3::new(8, 0, 0)));
    }

    #[test]
    fn same_brick_is_allocated_only_once() {
        let mut storage = SparseVoxelStorage::new();
        let key = BrickKey::new(2, 3, 4);

        storage.ensure_brick(key);
        storage.ensure_brick(key);

        assert_eq!(storage.brick_count(), 1);
    }

    #[test]
    fn different_bricks_get_different_indices() {
        let mut storage = SparseVoxelStorage::new();

        storage.ensure_brick(BrickKey::new(0, 0, 0));
        storage.ensure_brick(BrickKey::new(1, 0, 0));

        assert_eq!(storage.brick_count(), 2);
    }

    #[test]
    fn setting_a_voxel_allocates_its_brick() {
        let mut storage = SparseVoxelStorage::new();
        let key = BrickKey::new(1, 2, 3);

        assert_eq!(storage.brick_count(), 0);

        assert!(storage.set_occupied(key, UVec3::new(4, 5, 6)));

        assert_eq!(storage.brick_count(), 1);
        assert!(storage.brick(key).unwrap().is_occupied(UVec3::new(4, 5, 6)));
    }

    #[test]
    fn invalid_local_coordinate_does_not_allocate_a_brick() {
        let mut storage = SparseVoxelStorage::new();

        assert!(!storage.set_occupied(BrickKey::new(1, 2, 3), UVec3::new(8, 0, 0)));
        assert_eq!(storage.brick_count(), 0);
    }
}
