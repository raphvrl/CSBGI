use glam::{Quat, Vec2, Vec3};
use winit::keyboard::KeyCode;

use super::Camera;

pub struct CameraController {
    speed: f32,
    sensitivity: f32,

    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,

    yaw: f32,
    pitch: f32,
    mouse_delta: Vec2,
    active: bool,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,

            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false,

            yaw: 0.0,
            pitch: 0.0,
            mouse_delta: Vec2::ZERO,
            active: false,
        }
    }

    pub fn on_key(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::KeyW => self.forward = pressed,
            KeyCode::KeyS => self.backward = pressed,
            KeyCode::KeyA => self.left = pressed,
            KeyCode::KeyD => self.right = pressed,
            KeyCode::Space => self.up = pressed,

            KeyCode::ControlLeft | KeyCode::ControlRight => {
                self.down = pressed;
            }

            _ => {}
        }
    }

    pub fn on_mouse_motion(&mut self, delta: (f64, f64)) {
        if !self.active {
            return;
        }

        let delta = Vec2::new(delta.0 as f32, delta.1 as f32);
        self.mouse_delta += delta;
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;

        if !active {
            self.reset_input();
        }
    }

    pub fn update(&mut self, camera: &mut Camera, dt: f32) {
        if !self.active {
            return;
        }

        self.update_rotation(camera);
        self.update_position(camera, dt);

        self.mouse_delta = Vec2::ZERO;
    }

    fn update_rotation(&mut self, camera: &mut Camera) {
        self.yaw -= self.mouse_delta.x * self.sensitivity;
        self.pitch -= self.mouse_delta.y * self.sensitivity;

        let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01;

        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);

        camera.rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
    }

    fn update_position(&self, camera: &mut Camera, dt: f32) {
        let front = camera.front();
        let right = camera.right();

        let horizontal_front = Vec3::new(front.x, 0.0, front.z).normalize_or_zero();
        let horizontal_right = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        let mut direction = Vec3::ZERO;

        if self.forward {
            direction += horizontal_front;
        }

        if self.backward {
            direction -= horizontal_front;
        }

        if self.right {
            direction += horizontal_right;
        }

        if self.left {
            direction -= horizontal_right;
        }

        if self.up {
            direction += Vec3::Y;
        }

        if self.down {
            direction -= Vec3::Y;
        }

        if direction != Vec3::ZERO {
            camera.position += direction.normalize() * self.speed * dt;
        }
    }

    fn reset_input(&mut self) {
        self.forward = false;
        self.backward = false;
        self.left = false;
        self.right = false;
        self.up = false;
        self.down = false;
        self.mouse_delta = Vec2::ZERO;
    }
}
