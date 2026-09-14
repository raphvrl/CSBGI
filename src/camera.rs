pub mod controller;
pub mod projection;

pub use controller::CameraController;
pub use projection::Projection;

use glam::{Mat4, Quat, Vec3, camera::rh::view};

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
