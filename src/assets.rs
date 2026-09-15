pub mod data;
pub mod handles;
pub mod loaders;
pub mod meshes;
pub mod scenes;

pub use loaders::AssetLoader;
pub use meshes::MeshAssets;
pub use scenes::{SceneAsset, upload_scene};
