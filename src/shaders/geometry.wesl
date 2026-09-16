struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) light_position: vec4<f32>,
};

struct Frame {
    view_projection: mat4x4<f32>,
    light_view_projection: mat4x4<f32>,
    camera_position: vec3<f32>,
    pad0: f32,
    light_direction: vec3<f32>,
    light_intensity: f32,
    light_color: vec3<f32>,
    ambient: f32,
};

@group(0) @binding(0)
var<uniform> frame: Frame;

@group(0) @binding(1)
var shadow_map: texture_depth_2d;

@group(0) @binding(2)
var shadow_sampler: sampler_comparison;

struct ObjectUniform {
    model: mat4x4<f32>,
};

@group(1) @binding(0)
var<storage, read> objects: array<ObjectUniform>;

struct Material {
    base_color_factor: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: Material;

@group(2) @binding(1)
var base_color_texture: texture_2d<f32>;

@group(2) @binding(2)
var base_color_sampler: sampler;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance: u32) -> VertexOutput {
    let view_projection = frame.view_projection;
    let model = objects[instance].model;

    var out: VertexOutput;
    out.position = view_projection * model * vec4<f32>(in.position, 1.0);
    out.normal = normalize((model * vec4<f32>(in.normal, 0.0)).xyz);
    out.uv = in.uv;
    out.light_position = frame.light_view_projection * model * vec4<f32>(in.position, 1.0);
    return out;
}

fn shadow_visibility(light_position: vec4<f32>) -> f32 {
    if light_position.w <= 0.0 {
        return 1.0;
    }

    let light_ndc = light_position.xyz / light_position.w;
    let shadow_uv = vec2<f32>(
        light_ndc.x * 0.5 + 0.5,
        -light_ndc.y * 0.5 + 0.5,
    );

    if shadow_uv.x < 0.0 || shadow_uv.x > 1.0 ||
        shadow_uv.y < 0.0 || shadow_uv.y > 1.0 ||
        light_ndc.z < 0.0 || light_ndc.z > 1.0 {
        return 1.0;
    }

    return textureSampleCompare(
        shadow_map,
        shadow_sampler,
        shadow_uv,
        light_ndc.z,
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(
        base_color_texture,
        base_color_sampler,
        in.uv,
    );

    let normal = normalize(in.normal);

    let light_direction = normalize(frame.light_direction.xyz);
    let surface_to_light = -light_direction;

    let diffuse_factor = max(dot(normal, surface_to_light), 0.0);
    let diffuse_intensity = frame.light_intensity * diffuse_factor;
    let shadow = shadow_visibility(in.light_position);

    let ambient_color = frame.light_color.rgb * frame.ambient;
    let diffuse_color = frame.light_color.rgb * diffuse_intensity * shadow;

    let lighting = ambient_color + diffuse_color;

    let color = texture_color * material.base_color_factor;

    return vec4<f32>(color.rgb * lighting, color.a);
}
