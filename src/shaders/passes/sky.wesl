import package::common::frame::{Frame, lux_to_render};
import package::sky::atmosphere::sun_transmittance;

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,

    @location(0)
    ray_direction: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> frame: Frame;

@group(0) @binding(4)
var sky_lut: texture_2d<f32>;

@group(0) @binding(5)
var sky_sampler: sampler;

const PI: f32 = 3.14159265359;
const SUN_ANGULAR_RADIUS_RAD: f32 = 0.00465;
const UNDER_HORIZON_COLOR: vec3<f32> = vec3<f32>(0.025, 0.035, 0.05);
const HORIZON_BLEND_START: f32 = -0.35;
const HORIZON_BLEND_END: f32 = 0.05;

fn direction_uv(direction: vec3<f32>) -> vec2<f32> {
    let theta = acos(clamp(direction.y, -1.0, 1.0));
    let phi = atan2(direction.z, direction.x);

    return vec2<f32>(phi / (2.0 * PI) + 0.5, theta / PI);
}

fn sun(ray_direction: vec3<f32>) -> vec3<f32> {
    let sun_direction = normalize(-frame.light.light_direction);

    if sun_direction.y <= 0.0 {
        return vec3<f32>(0.0);
    }

    let disk = smoothstep(
        cos(SUN_ANGULAR_RADIUS_RAD),
        cos(SUN_ANGULAR_RADIUS_RAD * 0.8),
        dot(ray_direction, sun_direction),
    );

    let transmittance = sun_transmittance(
        frame.camera.camera_position,
        frame.light.light_direction,
    );

    return disk
        * transmittance
        * frame.light.light_color
        * lux_to_render(frame.light.light_illuminance_lux);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );

    let ndc = positions[vertex_index];
    let near_point = frame.camera.inverse_view_projection * vec4<f32>(ndc, 0.0, 1.0);
    let far_point = frame.camera.inverse_view_projection * vec4<f32>(ndc, 1.0, 1.0);

    var output: VertexOutput;
    output.position = vec4<f32>(ndc, 1.0, 1.0);
    output.ray_direction = far_point.xyz / far_point.w - near_point.xyz / near_point.w;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let ray_direction = normalize(input.ray_direction);
    let sky_color = textureSample(
        sky_lut,
        sky_sampler,
        direction_uv(ray_direction),
    ).rgb;
    let horizon_blend = smoothstep(
        HORIZON_BLEND_START,
        HORIZON_BLEND_END,
        ray_direction.y,
    );
    let background_color = mix(
        UNDER_HORIZON_COLOR,
        sky_color,
        horizon_blend,
    );
    let sun_color = sun(ray_direction) * horizon_blend;

    return vec4<f32>(background_color + sun_color, 1.0);
}
