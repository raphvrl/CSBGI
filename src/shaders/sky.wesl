import package::atmosphere::sun_transmittance;

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

@group(0) @binding(3)
var scene_depth: texture_depth_multisampled_2d;

@group(0) @binding(4)
var sky_lut: texture_2d<f32>;

@group(0) @binding(5)
var sky_sampler: sampler;

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,

    @location(0)
    ray_direction: vec3<f32>,
};

const PI: f32 = 3.14159265359;
const SUN_ANGULAR_RADIUS_RAD: f32 = 0.00465;

fn uv_from_direction(direction: vec3<f32>) -> vec2<f32> {
    let theta = acos(clamp(direction.y, -1.0, 1.0));
    let phi = atan2(direction.z, direction.x);

    return vec2<f32>(
        phi / (2.0 * PI) + 0.5,
        theta / PI,
    );
}

fn calculate_sun(ray_direction: vec3<f32>) -> vec3<f32> {
    let sun_direction = normalize(-frame.light_direction);

    if sun_direction.y <= 0.0 {
        return vec3<f32>(0.0);
    }

    let disk = smoothstep(
        cos(SUN_ANGULAR_RADIUS_RAD),
        cos(SUN_ANGULAR_RADIUS_RAD * 0.8),
        dot(ray_direction, sun_direction),
    );

    let transmittance = sun_transmittance(
        frame.camera_position,
        frame.light_direction,
    );

    return disk
        * transmittance
        * frame.light_color
        * frame.light_intensity;
}

fn sky_coverage(pixel: vec2<i32>) -> f32 {
    let depth0 = textureLoad(scene_depth, pixel, 0u);
    let depth1 = textureLoad(scene_depth, pixel, 1u);
    let depth2 = textureLoad(scene_depth, pixel, 2u);
    let depth3 = textureLoad(scene_depth, pixel, 3u);

    let clear_samples = select(0.0, 1.0, depth0 >= 1.0)
        + select(0.0, 1.0, depth1 >= 1.0)
        + select(0.0, 1.0, depth2 >= 1.0)
        + select(0.0, 1.0, depth3 >= 1.0);

    return clear_samples * 0.25;
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );

    let ndc = positions[vertex_index];

    let near_point = frame.inverse_view_projection * vec4<f32>(ndc, 0.0, 1.0);
    let far_point = frame.inverse_view_projection * vec4<f32>(ndc, 1.0, 1.0);

    var output: VertexOutput;
    output.position = vec4<f32>(ndc, 1.0, 1.0);
    output.ray_direction = far_point.xyz / far_point.w - near_point.xyz / near_point.w;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let coverage = sky_coverage(vec2<i32>(input.position.xy));

    if coverage <= 0.0 {
        discard;
    }

    let ray_direction = normalize(input.ray_direction);
    let uv = uv_from_direction(ray_direction);
    let sky_color = textureSample(sky_lut, sky_sampler, uv).rgb;
    let sun_color = calculate_sun(ray_direction);

    return vec4<f32>(sky_color + sun_color, coverage);
}
