pub mod depth;
pub mod scene_color;
pub mod shadow;

pub use depth::DepthTarget;
pub use scene_color::SceneColorTarget;
pub use shadow::ShadowTarget;

pub const MSAA_SAMPLES: u32 = 4;
