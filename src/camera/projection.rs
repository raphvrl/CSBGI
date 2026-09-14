use glam::{Mat4, camera::rh::proj::directx};

pub enum Projection {
    Perspective {
        fov_y: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    },
}

impl Projection {
    pub fn perspective(fov_y: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Projection::Perspective {
            fov_y,
            aspect_ratio,
            near,
            far,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        let aspect_ratio = width as f32 / height as f32;

        match self {
            Self::Perspective {
                aspect_ratio: current,
                ..
            } => {
                *current = aspect_ratio;
            }
        }
    }

    pub fn matrix(&self) -> Mat4 {
        match self {
            Projection::Perspective {
                fov_y,
                aspect_ratio,
                near,
                far,
            } => directx::perspective(*fov_y, *aspect_ratio, *near, *far),
        }
    }
}
