pub mod color;
pub mod depth;
pub mod luminance;
pub mod shadow;

pub use color::SceneColorTarget;
pub use depth::DepthTarget;
pub use luminance::LuminanceTarget;
pub use shadow::ShadowTarget;

pub const MSAA_SAMPLES: u32 = 4;
