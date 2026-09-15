pub mod depth;
pub mod msaa;

pub use depth::DepthTarget;
pub use msaa::MsaaTarget;

pub const MSAA_SAMPLES: u32 = 4;
