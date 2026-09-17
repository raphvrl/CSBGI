struct Camera {
    view_projection: mat4x4<f32>,
    inverse_view_projection: mat4x4<f32>,
    camera_position: vec3<f32>,

    pad0: f32,
};

struct Light {
    view_projection: mat4x4<f32>,
    light_direction: vec3<f32>,
    light_intensity: f32,

    light_color: vec3<f32>,
    ambient: f32,
};

struct Frame {
    camera: Camera,
    light: Light,
};
