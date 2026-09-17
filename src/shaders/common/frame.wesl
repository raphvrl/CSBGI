struct Camera {
    view_projection: mat4x4<f32>,
    inverse_view_projection: mat4x4<f32>,
    camera_position: vec3<f32>,

    pad0: f32,
};

struct Light {
    view_projection: mat4x4<f32>,
    light_direction: vec3<f32>,
    light_illuminance_lux: f32,

    light_color: vec3<f32>,
    ambient_illuminance_lux: f32,
};

struct Frame {
    camera: Camera,
    light: Light,
};

const REFERENCE_ILLUMINANCE_LUX: f32 = 100000.0;

fn lux_to_render(illuminance_lux: f32) -> f32 {
    return illuminance_lux / REFERENCE_ILLUMINANCE_LUX;
}
