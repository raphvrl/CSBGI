pub mod data;
pub mod handles;
pub mod loaders;
pub mod materials;
pub mod meshes;
pub mod scenes;
pub mod textures;

pub use loaders::AssetLoader;
pub use materials::MaterialAssets;
pub use meshes::MeshAssets;
pub use scenes::{SceneAsset, upload_scene};
pub use textures::TextureAssets;
