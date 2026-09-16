use glam::Vec3;

pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub ambient: f32,
}

impl DirectionalLight {
    pub fn new(direction: Vec3, color: Vec3, intensity: f32, ambient: f32) -> Self {
        Self {
            direction,
            color,
            intensity,
            ambient,
        }
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self::new(Vec3::new(-1.0, -1.0, -1.0).normalize(), Vec3::ONE, 1.0, 0.1)
    }
}
