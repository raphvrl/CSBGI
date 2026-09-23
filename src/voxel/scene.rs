use anyhow::{Result, anyhow, bail};

use crate::assets::data::{MeshData, SceneData};

use super::{
    DEFAULT_ALBEDO, SparseVoxelStorage, VoxelGrid, VoxelMaterial, mesh::voxelize_mesh, walk::visit_meshes,
};

pub fn voxelize_scene(grid: &VoxelGrid, storage: &mut SparseVoxelStorage, scene: &SceneData) -> Result<()> {
    if !grid.is_valid() {
        bail!("voxelization requires a finite grid with a positive voxel size");
    }

    visit_meshes(scene, |instance| {
        let material = mesh_material(scene, instance.mesh)?;
        voxelize_mesh(
            grid,
            storage,
            &instance.mesh.vertices,
            &instance.mesh.indices,
            instance.transform,
            material,
        )
    })
    .map(|_| ())
}

fn mesh_material<'a>(scene: &'a SceneData, mesh: &MeshData) -> Result<VoxelMaterial<'a>> {
    let material = match mesh.material {
        Some(index) => scene
            .materials
            .get(index)
            .ok_or_else(|| anyhow!("mesh references missing material {index}"))?,
        None => {
            return Ok(VoxelMaterial {
                color_factor: DEFAULT_ALBEDO,
                color_image: None,
            });
        }
    };

    let color_image = material
        .color_image
        .map(|index| {
            scene
                .images
                .get(index)
                .ok_or_else(|| anyhow!("material references missing image {index}"))
        })
        .transpose()?;

    Ok(VoxelMaterial {
        color_factor: material.color_factor,
        color_image,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use glam::{Mat4, UVec3, Vec2, Vec3};

    use crate::assets::data::{ImageData, MaterialData, NodeData, VertexData};

    fn test_grid() -> VoxelGrid {
        VoxelGrid {
            origin: Vec3::ZERO,
            voxel_size: 1.0,
            dimensions: UVec3::splat(16),
        }
    }

    fn test_mesh() -> MeshData {
        MeshData {
            vertices: vec![
                VertexData {
                    position: [0.1, 0.1, 0.5],
                    normal: [0.0, 0.0, 1.0],
                    uv: [0.0, 0.0],
                    tangent: [1.0, 0.0, 0.0, 1.0],
                },
                VertexData {
                    position: [0.9, 0.1, 0.5],
                    normal: [0.0, 0.0, 1.0],
                    uv: [1.0, 0.0],
                    tangent: [1.0, 0.0, 0.0, 1.0],
                },
                VertexData {
                    position: [0.1, 0.9, 0.5],
                    normal: [0.0, 0.0, 1.0],
                    uv: [0.0, 1.0],
                    tangent: [1.0, 0.0, 0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2],
            material: None,
        }
    }

    #[test]
    fn scene_mesh_is_voxelized() {
        let scene = SceneData {
            meshes: vec![test_mesh()],
            nodes: vec![NodeData {
                transform: Mat4::IDENTITY,
                meshes: vec![0],
                children: Vec::new(),
            }],
            roots: vec![0],
            ..Default::default()
        };

        let mut storage = SparseVoxelStorage::new();
        voxelize_scene(&test_grid(), &mut storage, &scene).unwrap();

        assert!(
            storage
                .brick(crate::voxel::BrickKey::new(0, 0, 0))
                .unwrap()
                .is_occupied(UVec3::new(0, 0, 0))
        );
    }

    #[test]
    fn mesh_material_color_is_voxelized() {
        let mut mesh = test_mesh();
        mesh.material = Some(0);

        let scene = SceneData {
            materials: vec![MaterialData {
                color_factor: [1.0, 0.0, 0.0, 1.0],
                metallic_factor: 0.0,
                roughness_factor: 1.0,
                occlusion_factor: 1.0,
                color_image: None,
                normal_image: None,
                orm_image: None,
                normal_scale: 1.0,
            }],
            meshes: vec![mesh],
            nodes: vec![NodeData {
                transform: Mat4::IDENTITY,
                meshes: vec![0],
                children: Vec::new(),
            }],
            roots: vec![0],
            ..Default::default()
        };

        let mut storage = SparseVoxelStorage::new();
        voxelize_scene(&test_grid(), &mut storage, &scene).unwrap();

        assert_eq!(
            storage
                .brick(crate::voxel::BrickKey::new(0, 0, 0))
                .unwrap()
                .albedo(UVec3::new(0, 0, 0)),
            Some(0xff00_00ff)
        );
    }

    #[test]
    fn textured_material_color_is_sampled() {
        let mut mesh = test_mesh();
        mesh.material = Some(0);

        let scene = SceneData {
            images: vec![ImageData {
                width: 1,
                height: 1,
                rgba8: vec![0, 255, 0, 255],
            }],
            materials: vec![MaterialData {
                color_factor: [1.0, 1.0, 1.0, 1.0],
                metallic_factor: 0.0,
                roughness_factor: 1.0,
                occlusion_factor: 1.0,
                color_image: Some(0),
                normal_image: None,
                orm_image: None,
                normal_scale: 1.0,
            }],
            ..Default::default()
        };

        let material = mesh_material(&scene, &mesh).unwrap();
        assert_eq!(
            crate::voxel::sample_albedo(material, Vec2::splat(0.5)),
            0xff00_ff00
        );
    }

    #[test]
    fn node_transforms_are_composed() {
        let scene = SceneData {
            meshes: vec![test_mesh()],
            nodes: vec![
                NodeData {
                    transform: Mat4::from_translation(Vec3::X),
                    meshes: Vec::new(),
                    children: vec![1],
                },
                NodeData {
                    transform: Mat4::from_translation(Vec3::X),
                    meshes: vec![0],
                    children: Vec::new(),
                },
            ],
            roots: vec![0],
            ..Default::default()
        };

        let mut storage = SparseVoxelStorage::new();
        voxelize_scene(&test_grid(), &mut storage, &scene).unwrap();

        assert!(
            storage
                .brick(crate::voxel::BrickKey::new(0, 0, 0))
                .unwrap()
                .is_occupied(UVec3::new(2, 0, 0))
        );
    }

    #[test]
    fn invalid_root_node_returns_error() {
        let scene = SceneData {
            roots: vec![10],
            ..Default::default()
        };

        let mut storage = SparseVoxelStorage::new();
        assert!(voxelize_scene(&test_grid(), &mut storage, &scene).is_err());
    }

    #[test]
    fn missing_material_returns_error() {
        let mut mesh = test_mesh();
        mesh.material = Some(0);

        let scene = SceneData {
            meshes: vec![mesh],
            nodes: vec![NodeData {
                transform: Mat4::IDENTITY,
                meshes: vec![0],
                children: Vec::new(),
            }],
            roots: vec![0],
            ..Default::default()
        };

        let mut storage = SparseVoxelStorage::new();
        assert!(voxelize_scene(&test_grid(), &mut storage, &scene).is_err());
    }
}
