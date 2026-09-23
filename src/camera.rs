pub mod controller;
pub mod projection;

pub use controller::CameraController;
pub use projection::Projection;

use glam::{Mat4, Quat, Vec3, camera::rh::view};
use winit::keyboard::KeyCode;

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

pub struct CameraRig {
    camera: Camera,
    exposure: CameraExposureSettings,
    controller: CameraController,
}

impl CameraRig {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        let aspect_ratio = width as f32 / height as f32;
        let projection = Projection::perspective(60.0_f32.to_radians(), aspect_ratio, 0.1, 100.0);
        let camera = Camera::new(Vec3::new(0.0, 0.0, 2.0), Quat::IDENTITY, projection);

        Self {
            camera,
            exposure: CameraExposureSettings::default(),
            controller: CameraController::new(5.0, 0.003),
        }
    }

    pub(crate) fn update(&mut self, dt: f32) {
        self.controller.update(&mut self.camera, dt);
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    pub(crate) fn on_key(&mut self, key: KeyCode, pressed: bool) {
        self.controller.on_key(key, pressed);
    }

    pub(crate) fn on_mouse_motion(&mut self, delta: (f64, f64)) {
        self.controller.on_mouse_motion(delta);
    }

    pub(crate) fn set_active(&mut self, active: bool) {
        self.controller.set_active(active);
    }

    pub(crate) fn camera(&self) -> &Camera {
        &self.camera
    }

    pub(crate) fn exposure_settings_mut(&mut self) -> &mut CameraExposureSettings {
        &mut self.exposure
    }

    pub(crate) fn exposure_stops(&self) -> f32 {
        self.exposure.exposure_stops()
    }
}
