use glam::{
    Mat4, Vec3,
    camera::rh::{proj::directx, view},
};

pub struct ShadowSettings {
    pub center: Vec3,
    pub half_extent: f32,
    pub near: f32,
    pub far: f32,
    pub distance: f32,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            half_extent: 35.0,
            near: 0.1,
            far: 120.0,
            distance: 25.0,
        }
    }
}

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

    pub fn shadow_view_projection(&self, settings: &ShadowSettings) -> Mat4 {
        let direction = self.direction.normalize_or_zero();
        let direction = if direction.length_squared() > 0.0 {
            direction
        } else {
            Vec3::NEG_Z
        };

        let up = if direction.dot(Vec3::Y).abs() > 0.99 {
            Vec3::Z
        } else {
            Vec3::Y
        };

        let light_position = settings.center - direction * settings.distance;
        let view = view::look_at_mat4(light_position, settings.center, up);
        let projection = directx::orthographic(
            -settings.half_extent,
            settings.half_extent,
            -settings.half_extent,
            settings.half_extent,
            settings.near,
            settings.far,
        );

        projection * view
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self::new(Vec3::new(-1.0, -1.0, -1.0).normalize(), Vec3::ONE, 1.0, 0.1)
    }
}
