mod gltf;

pub use gltf::GltfLoader;

use std::{ffi::OsStr, path::PathBuf};

use anyhow::{Result, anyhow, bail};

use super::data::SceneData;

#[derive(Default)]
pub struct AssetLoader {
    gltf: GltfLoader,
}

impl AssetLoader {
    pub fn load(&self, path: impl Into<PathBuf>) -> Result<SceneData> {
        let path = path.into();

        let extension = path
            .extension()
            .and_then(OsStr::to_str)
            .ok_or_else(|| anyhow!("asset without file extension: {}", path.display()))?
            .to_ascii_lowercase();

        match extension.as_str() {
            "gltf" | "glb" => self.gltf.load(&path),
            _ => bail!("unsupported asset format: .{extension}"),
        }
    }
}
