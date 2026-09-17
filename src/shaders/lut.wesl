import package::atmosphere::calculate_sky;

struct Frame {
    view_projection: mat4x4<f32>,
    inverse_view_projection: mat4x4<f32>,
    light_view_projection: mat4x4<f32>,

    camera_position: vec3<f32>,

    _pad0: f32,

    light_direction: vec3<f32>,
    light_intensity: f32,

    light_color: vec3<f32>,
    ambient: f32,
};

@group(0) @binding(0)
var<uniform> frame: Frame;

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,

    @location(0)
    uv: vec2<f32>,
};

const PI: f32 = 3.14159265359;

fn direction_from_uv(uv: vec2<f32>) -> vec3<f32> {
    let theta = uv.y * PI;
    let phi = (uv.x - 0.5) * 2.0 * PI;

    let sin_theta = sin(theta);

    return normalize(vec3<f32>(
        sin_theta * cos(phi),
        cos(theta),
        sin_theta * sin(phi),
    ));
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );

    let ndc = positions[vertex_index];

    var output: VertexOutput;
    output.position = vec4<f32>(ndc, 1.0, 1.0);

    output.uv = vec2<f32>(
        ndc.x * 0.5 + 0.5,
        0.5 - ndc.y * 0.5,
    );

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let ray_direction = direction_from_uv(input.uv);

    let color = calculate_sky(
        ray_direction,
        frame.camera_position,
        frame.light_direction,
        frame.light_color,
        frame.light_intensity,
    );

    return vec4<f32>(color, 1.0);
}
