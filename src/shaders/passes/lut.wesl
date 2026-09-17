import package::common::frame::{Frame, lux_to_render};
import package::sky::atmosphere::calculate;

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,

    @location(0)
    uv: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> frame: Frame;

const PI: f32 = 3.14159265359;

fn direction(uv: vec2<f32>) -> vec3<f32> {
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
    output.uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = calculate(
        direction(input.uv),
        frame.camera.camera_position,
        frame.light.light_direction,
        frame.light.light_color,
        lux_to_render(frame.light.light_illuminance_lux),
    );

    return vec4<f32>(color, 1.0);
}
