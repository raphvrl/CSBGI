pub mod depth;
pub mod color;
pub mod shadow;

pub use depth::DepthTarget;
pub use color::SceneColorTarget;
pub use shadow::ShadowTarget;

pub const MSAA_SAMPLES: u32 = 4;
