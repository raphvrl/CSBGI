use glam::Mat4;

use crate::assets::handles::MeshHandle;

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Transform {
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: glam::Vec3::new(x, y, z),
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }

    pub fn matrix(self) -> glam::Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

pub struct MeshRenderer {
    pub mesh: MeshHandle,
}
