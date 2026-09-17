use glam::{
    Mat4, Vec3,
    camera::rh::{proj::directx, view},
};

pub const REFERENCE_ILLUMINANCE_LUX: f32 = 100_000.0;

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
    pub illuminance_lux: f32,
    pub ambient_illuminance_lux: f32,

    sun_rise: f32,
    sun_yaw: f32,
}

impl DirectionalLight {
    pub fn new(
        direction: Vec3,
        color: Vec3,
        illuminance_lux: f32,
        ambient_illuminance_lux: f32,
    ) -> Self {
        let sun_direction = -direction.normalize_or_zero();
        let horizontal = Vec3::new(sun_direction.x, 0.0, sun_direction.z).length();
        let rise = horizontal
            .atan2(sun_direction.y)
            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

        Self {
            direction,
            color,
            illuminance_lux,
            ambient_illuminance_lux,
            sun_rise: rise,
            sun_yaw: sun_direction.z.atan2(sun_direction.x),
        }
    }

    pub fn sun_angles(&self) -> (f32, f32) {
        (self.sun_rise, self.sun_yaw)
    }

    pub fn set_sun_angles(&mut self, rise: f32, yaw: f32) {
        self.sun_rise = rise.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
        self.sun_yaw = yaw;

        let horizontal = self.sun_rise.sin();
        let sun_direction = Vec3::new(
            horizontal * self.sun_yaw.cos(),
            self.sun_rise.cos(),
            horizontal * self.sun_yaw.sin(),
        );

        self.direction = -sun_direction;
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
        Self::new(
            Vec3::new(-1.0, -1.0, -1.0).normalize(),
            Vec3::ONE,
            REFERENCE_ILLUMINANCE_LUX,
            10_000.0,
        )
    }
}
