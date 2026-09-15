struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct Frame {
    view_projection: mat4x4<f32>,
    camera_position: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> frame: Frame;

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
    out.normal = in.normal;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(
        base_color_texture,
        base_color_sampler,
        in.uv,
    );

    let color = texture_color * material.base_color_factor;

    return color;
}
