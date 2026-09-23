use crate::assets::data::{MeshData, SceneData};
use anyhow::{Result, bail};
use glam::Mat4;

use super::{
    BRICK_SIZE, BrickRange, VoxelGrid, mesh::transformed_mesh_positions, triangle_voxel_range,
    walk::visit_meshes,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct VoxelizationStats {
    pub nodes_visited: usize,
    pub mesh_instances: usize,
    pub triangle_count: usize,
    pub triangles_with_candidates: usize,
    pub candidate_brick_visits: usize,
    pub tight_voxel_tests: usize,
}

impl VoxelizationStats {
    pub fn theoretical_voxel_tests(self) -> usize {
        self.candidate_brick_visits * (BRICK_SIZE as usize).pow(3)
    }
}

pub fn analyze_scene(grid: &VoxelGrid, scene: &SceneData) -> Result<VoxelizationStats> {
    if !grid.is_valid() {
        bail!("voxel analysis requires a finite grid with a positive voxel size");
    }

    let mut stats = VoxelizationStats::default();

    stats.nodes_visited = visit_meshes(scene, |instance| {
        stats.mesh_instances += 1;
        analyze_mesh(grid, instance.mesh, instance.transform, &mut stats)
    })?;

    Ok(stats)
}

fn analyze_mesh(grid: &VoxelGrid, mesh: &MeshData, model: Mat4, stats: &mut VoxelizationStats) -> Result<()> {
    for triangle in transformed_mesh_positions(mesh, model)? {
        stats.triangle_count += 1;

        if let Some(voxel_range) = triangle_voxel_range(grid, triangle) {
            let brick_range = BrickRange::from_voxel_range(voxel_range);
            stats.triangles_with_candidates += 1;
            stats.candidate_brick_visits += brick_range.iter().count();
            stats.tight_voxel_tests += brick_range.voxel_count(voxel_range);
        }
    }

    Ok(())
}
