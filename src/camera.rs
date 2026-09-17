pub mod controller;
pub mod projection;

pub use controller::CameraController;
pub use projection::Projection;

use glam::{Mat4, Quat, Vec3, camera::rh::view};

#[derive(Clone, Copy)]
pub struct CameraExposureSettings {
    pub aperture: f32,
    pub shutter_speed_seconds: f32,
    pub iso: f32,
    pub exposure_compensation: f32,
}

impl Default for CameraExposureSettings {
    fn default() -> Self {
        Self {
            aperture: 16.0,
            shutter_speed_seconds: 1.0 / 100.0,
            iso: 100.0,
            exposure_compensation: 0.0,
        }
    }
}

impl CameraExposureSettings {
    pub fn ev100(self) -> f32 {
        let aperture = self.aperture.max(0.001);
        let shutter_speed = self.shutter_speed_seconds.max(0.000001);
        let iso = self.iso.max(0.001);

        (aperture * aperture / shutter_speed).log2() - (iso / 100.0).log2()
    }

    pub fn exposure_stops(self) -> f32 {
        let reference = Self::default().ev100();

        reference - self.ev100() + self.exposure_compensation
    }
}

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub projection: Projection,
}

impl Camera {
    pub fn new(position: Vec3, rotation: Quat, projection: Projection) -> Self {
        Camera {
            position,
            rotation,
            projection,
        }
    }

    pub fn front(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn view_matrix(&self) -> Mat4 {
        view::look_to_mat4(self.position, self.front(), self.up())
    }

    pub fn projection_matrix(&self) -> Mat4 {
        self.projection.matrix()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if height == 0 {
            return;
        }

        self.projection.resize(width, height);
    }
}
