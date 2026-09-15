use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use glam::{Mat4, Quat, Vec3};
use rayon::prelude::*;

use crate::assets::data::{ImageData, MaterialData, MeshData, NodeData, SceneData, VertexData};

#[derive(Default)]
pub struct GltfLoader;

impl GltfLoader {
    pub fn load(&self, path: &Path) -> Result<SceneData> {
        let (document, buffers, images) =
            gltf::import(path).with_context(|| format!("failed to load {}", path.display()))?;

        let images = read_images(images)?;
        let materials = read_materials(&document);
        let (meshes, mesh_map) = read_meshes(&document, &buffers)?;
        let (nodes, roots) = read_scene(&document, &mesh_map)?;

        Ok(SceneData {
            images,
            materials,
            meshes,
            nodes,
            roots,
        })
    }
}

fn read_images(images: Vec<gltf::image::Data>) -> Result<Vec<ImageData>> {
    images
        .into_par_iter()
        .enumerate()
        .map(|(index, image)| {
            convert_image(image).with_context(|| format!("failed to convert image {index}"))
        })
        .collect()
}

fn convert_image(image: gltf::image::Data) -> Result<ImageData> {
    let gltf::image::Data {
        pixels,
        format,
        width,
        height,
    } = image;

    let data = match format {
        gltf::image::Format::R8 => {
            validate_pixel_data(&pixels, width, height, 1)?;
            pixels
                .as_chunks::<1>()
                .0
                .iter()
                .flat_map(|pixel| [pixel[0], 255, 255, 255])
                .collect()
        }
        gltf::image::Format::R8G8 => {
            validate_pixel_data(&pixels, width, height, 2)?;
            pixels
                .as_chunks::<2>()
                .0
                .iter()
                .flat_map(|pixel| [pixel[0], pixel[1], 0, 255])
                .collect()
        }
        gltf::image::Format::R8G8B8 => {
            validate_pixel_data(&pixels, width, height, 3)?;
            pixels
                .as_chunks::<3>()
                .0
                .iter()
                .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
                .collect()
        }
        gltf::image::Format::R8G8B8A8 => {
            validate_pixel_data(&pixels, width, height, 4)?;
            pixels
        }
        _ => bail!("unsupported image format {format:?}"),
    };

    Ok(ImageData {
        width,
        height,
        rgba8: data,
    })
}

fn validate_pixel_data(data: &[u8], width: u32, height: u32, channels: usize) -> Result<()> {
    let pixel_count = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| anyhow!("image dimensions overflow"))?;
    let expected_len = pixel_count
        .checked_mul(channels)
        .ok_or_else(|| anyhow!("image size overflow"))?;

    if data.len() != expected_len {
        bail!("image has {} bytes, expected {}", data.len(), expected_len);
    }

    Ok(())
}

fn read_materials(document: &gltf::Document) -> Vec<MaterialData> {
    document
        .materials()
        .map(|material| {
            let pbr = material.pbr_metallic_roughness();
            let color_image = pbr
                .base_color_texture()
                .map(|info| info.texture().source().index());

            MaterialData {
                color_factor: pbr.base_color_factor(),
                color_image,
            }
        })
        .collect()
}

fn read_meshes(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> Result<(Vec<MeshData>, Vec<Vec<usize>>)> {
    let mut meshes = Vec::new();
    let mut mesh_map = Vec::new();

    for mesh in document.meshes() {
        let mut primitive_indices = Vec::new();

        for primitive in mesh.primitives() {
            let primitive_index = primitive.index();
            let material = primitive.material().index();
            let mesh_data = read_primitive(&primitive, buffers, material).with_context(|| {
                format!(
                    "failed to read mesh {} primitive {}",
                    mesh.index(),
                    primitive_index
                )
            })?;

            primitive_indices.push(meshes.len());
            meshes.push(mesh_data);
        }

        mesh_map.push(primitive_indices);
    }

    Ok((meshes, mesh_map))
}

fn read_scene(
    document: &gltf::Document,
    mesh_map: &[Vec<usize>],
) -> Result<(Vec<NodeData>, Vec<usize>)> {
    let nodes = document
        .nodes()
        .map(|node| {
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

            NodeData {
                transform,
                meshes,
                children,
            }
        })
        .collect();

    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next())
        .ok_or_else(|| anyhow!("glTF file does not contain a scene"))?;

    let roots = scene.nodes().map(|node| node.index()).collect();

    Ok((nodes, roots))
}

fn read_primitive(
    primitive: &gltf::mesh::Primitive<'_>,
    buffers: &[gltf::buffer::Data],
    material: Option<usize>,
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

    let uvs = reader
        .read_tex_coords(0)
        .ok_or_else(|| anyhow!("primitive has no TEXCOORD_0 attribute"))?
        .into_f32()
        .collect::<Vec<_>>();

    if positions.len() != normals.len() {
        bail!("positions and normals have different lengths");
    }

    if positions.len() != uvs.len() {
        bail!("positions and uvs have different lengths");
    }

    let vertices = positions
        .into_iter()
        .zip(normals)
        .zip(uvs)
        .map(|((position, normal), uv)| VertexData {
            position,
            normal,
            uv,
        })
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

    Ok(MeshData {
        vertices,
        indices,
        material,
    })
}
