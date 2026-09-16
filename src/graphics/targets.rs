pub mod depth;
pub mod msaa;
pub mod shadow;

pub use depth::DepthTarget;
pub use msaa::MsaaTarget;
pub use shadow::ShadowTarget;

pub const MSAA_SAMPLES: u32 = 4;
