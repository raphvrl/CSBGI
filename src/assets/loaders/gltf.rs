use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use glam::{Mat4, Quat, Vec3};

use crate::assets::data::{MeshData, NodeData, SceneData, VertexData};

#[derive(Default)]
pub struct GltfLoader;

impl GltfLoader {
    pub fn load(&self, path: &Path) -> Result<SceneData> {
        let (document, buffers, _) =
            gltf::import(path).with_context(|| format!("failed to load {}", path.display()))?;

        let mut scene_data = SceneData::default();

        let mut mesh_map = Vec::new();

        for mesh in document.meshes() {
            let mut primitive_indices = Vec::new();

            for primitive in mesh.primitives() {
                let primitive_index = primitive.index();
                let mesh_data = read_primitive(&primitive, &buffers).with_context(|| {
                    format!(
                        "failed to read mesh {} primitive {}",
                        mesh.index(),
                        primitive_index
                    )
                })?;

                let mesh_index = scene_data.meshes.len();

                scene_data.meshes.push(mesh_data);
                primitive_indices.push(mesh_index);
            }

            mesh_map.push(primitive_indices);
        }

        for node in document.nodes() {
            let (translation, rotation, scale) = node.transform().decomposed();

            let transform = Mat4::from_scale_rotation_translation(
                Vec3::from_array(scale),
                Quat::from_array(rotation),
                Vec3::from_array(translation),
            );

            let meshes = node
                .mesh()
                .map(|mesh| mesh_map[mesh.index()].clone())
                .unwrap_or_default();

            let children = node.children().map(|child| child.index()).collect();

            scene_data.nodes.push(NodeData {
                transform,
                meshes,
                children,
            });
        }

        let scene = document
            .default_scene()
            .or_else(|| document.scenes().next())
            .ok_or_else(|| anyhow!("glTF file does not contain a scene"))?;

        scene_data.roots = scene.nodes().map(|node| node.index()).collect();

        Ok(scene_data)
    }
}

fn read_primitive(
    primitive: &gltf::mesh::Primitive<'_>,
    buffers: &[gltf::buffer::Data],
) -> Result<MeshData> {
    if primitive.mode() != gltf::mesh::Mode::Triangles {
        bail!(
            "unsupported primitive mode {:?}; only triangles are supported",
            primitive.mode()
        );
    }

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions = reader
        .read_positions()
        .ok_or_else(|| anyhow!("primitive has no POSITION attribute"))?
        .collect::<Vec<_>>();

    let normals = reader
        .read_normals()
        .ok_or_else(|| anyhow!("primitive has no NORMAL attribute"))?
        .collect::<Vec<_>>();

    if positions.len() != normals.len() {
        bail!("positions and normals have different lengths");
    }

    let vertices = positions
        .into_iter()
        .zip(normals)
        .map(|(position, normal)| VertexData { position, normal })
        .collect::<Vec<_>>();

    let indices = match reader.read_indices() {
        Some(indices) => indices.into_u32().collect(),
        None => {
            if vertices.len() % 3 != 0 {
                bail!("non-indexed primitive vertex count is not divisible by 3");
            }

            (0..vertices.len() as u32).collect()
        }
    };

    Ok(MeshData { vertices, indices })
}
