use anyhow::{Result, anyhow, bail};
use glam::{Mat4, Vec2, Vec3};
use rayon::prelude::*;

use crate::assets::data::VertexData;

use super::voxelize::{voxelize_textured_triangles, voxelize_triangles_with_albedos};
use super::{SparseVoxelStorage, VoxelGrid};
use super::{TexturedTriangle, VoxelMaterial};

pub fn voxelize_mesh_with_albedos(
    grid: &VoxelGrid,
    storage: &mut SparseVoxelStorage,
    positions: &[Vec3],
    indices: &[u32],
    model: Mat4,
    albedos: &[u32],
) -> Result<()> {
    validate_triangle_indices(indices)?;

    if albedos.len() != indices.len() / 3 {
        bail!(
            "albedo count {} does not match triangle count {}",
            albedos.len(),
            indices.len() / 3
        );
    }

    let triangles = indices
        .par_chunks_exact(3)
        .enumerate()
        .map(|(triangle_index, triangle_indices)| {
            Ok([
                model.transform_point3(vertex_at(positions, triangle_indices[0], triangle_index, 0)?),
                model.transform_point3(vertex_at(positions, triangle_indices[1], triangle_index, 1)?),
                model.transform_point3(vertex_at(positions, triangle_indices[2], triangle_index, 2)?),
            ])
        })
        .collect::<Result<Vec<_>>>()?;

    voxelize_triangles_with_albedos(grid, storage, &triangles, albedos);

    Ok(())
}

pub fn voxelize_mesh(
    grid: &VoxelGrid,
    storage: &mut SparseVoxelStorage,
    vertices: &[VertexData],
    indices: &[u32],
    model: Mat4,
    material: VoxelMaterial<'_>,
) -> Result<()> {
    validate_triangle_indices(indices)?;

    let triangles = textured_triangles(vertices, indices, model)?;

    voxelize_textured_triangles(grid, storage, &triangles, material);

    Ok(())
}

fn validate_triangle_indices(indices: &[u32]) -> Result<()> {
    if indices.len().is_multiple_of(3) {
        return Ok(());
    }

    bail!("index buffer length {} is not divisible by 3", indices.len())
}

pub(super) fn transformed_mesh_positions(
    mesh: &crate::assets::data::MeshData,
    model: Mat4,
) -> Result<Vec<[Vec3; 3]>> {
    validate_triangle_indices(&mesh.indices)?;

    mesh.indices
        .par_chunks_exact(3)
        .enumerate()
        .map(|(triangle_index, indices)| {
            Ok([
                model.transform_point3(Vec3::from_array(
                    data_at(&mesh.vertices, indices[0], triangle_index, 0)?.position,
                )),
                model.transform_point3(Vec3::from_array(
                    data_at(&mesh.vertices, indices[1], triangle_index, 1)?.position,
                )),
                model.transform_point3(Vec3::from_array(
                    data_at(&mesh.vertices, indices[2], triangle_index, 2)?.position,
                )),
            ])
        })
        .collect()
}

fn textured_triangles(
    vertices: &[VertexData],
    indices: &[u32],
    model: Mat4,
) -> Result<Vec<TexturedTriangle>> {
    indices
        .par_chunks_exact(3)
        .enumerate()
        .map(|(triangle_index, indices)| {
            let vertices = [
                data_at(vertices, indices[0], triangle_index, 0)?,
                data_at(vertices, indices[1], triangle_index, 1)?,
                data_at(vertices, indices[2], triangle_index, 2)?,
            ];

            Ok(TexturedTriangle {
                positions: vertices.map(|vertex| model.transform_point3(Vec3::from_array(vertex.position))),
                uvs: vertices.map(|vertex| Vec2::from_array(vertex.uv)),
            })
        })
        .collect()
}

fn vertex_at(positions: &[Vec3], index: u32, triangle_index: usize, corner: usize) -> Result<Vec3> {
    let index = usize::try_from(index).map_err(|_| {
        anyhow!(
            "triangle {triangle_index}, corner {corner}: \
             index {index} cannot be converted to usize"
        )
    })?;

    positions.get(index).copied().ok_or_else(|| {
        anyhow!(
            "triangle {triangle_index}, corner {corner}: \
             index {index} is outside the vertex buffer"
        )
    })
}

fn data_at(vertices: &[VertexData], index: u32, triangle_index: usize, corner: usize) -> Result<VertexData> {
    let index = usize::try_from(index).map_err(|_| {
        anyhow!("triangle {triangle_index}, corner {corner}: index {index} cannot be converted to usize")
    })?;

    vertices.get(index).copied().ok_or_else(|| {
        anyhow!("triangle {triangle_index}, corner {corner}: index {index} is outside the vertex buffer")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::BrickKey;
    use glam::UVec3;

    fn test_grid() -> VoxelGrid {
        VoxelGrid {
            origin: Vec3::ZERO,
            voxel_size: 1.0,
            dimensions: UVec3::splat(16),
        }
    }

    #[test]
    fn indexed_mesh_is_voxelized() {
        let grid = test_grid();
        let mut storage = SparseVoxelStorage::new();

        let positions = [
            Vec3::new(0.1, 0.1, 0.5),
            Vec3::new(0.9, 0.1, 0.5),
            Vec3::new(0.1, 0.9, 0.5),
        ];

        let indices = [0, 1, 2];

        voxelize_mesh_with_albedos(
            &grid,
            &mut storage,
            &positions,
            &indices,
            Mat4::IDENTITY,
            &[0xffff_ffff],
        )
        .unwrap();

        let brick = storage.brick(BrickKey::new(0, 0, 0)).unwrap();

        assert!(brick.is_occupied(UVec3::new(0, 0, 0)));
    }

    #[test]
    fn model_transform_is_applied() {
        let grid = test_grid();
        let mut storage = SparseVoxelStorage::new();

        let positions = [
            Vec3::new(0.1, 0.1, 0.5),
            Vec3::new(0.9, 0.1, 0.5),
            Vec3::new(0.1, 0.9, 0.5),
        ];

        let indices = [0, 1, 2];

        let model = Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0));

        voxelize_mesh_with_albedos(&grid, &mut storage, &positions, &indices, model, &[0xffff_ffff]).unwrap();

        let brick = storage.brick(BrickKey::new(0, 0, 0)).unwrap();

        assert!(brick.is_occupied(UVec3::new(2, 0, 0)));
    }

    #[test]
    fn invalid_index_count_returns_error() {
        let grid = test_grid();
        let mut storage = SparseVoxelStorage::new();

        let positions = [Vec3::ZERO, Vec3::X, Vec3::Y];

        let indices = [0, 1];

        let result =
            voxelize_mesh_with_albedos(&grid, &mut storage, &positions, &indices, Mat4::IDENTITY, &[]);

        assert!(result.is_err());
    }

    #[test]
    fn out_of_bounds_index_returns_error() {
        let grid = test_grid();
        let mut storage = SparseVoxelStorage::new();

        let positions = [Vec3::ZERO, Vec3::X, Vec3::Y];

        let indices = [0, 1, 5];

        let result =
            voxelize_mesh_with_albedos(&grid, &mut storage, &positions, &indices, Mat4::IDENTITY, &[0]);

        assert!(result.is_err());
    }
}
