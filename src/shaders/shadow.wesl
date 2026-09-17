struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

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

struct ObjectUniform {
    model: mat4x4<f32>,
};

@group(1) @binding(0)
var<storage, read> objects: array<ObjectUniform>;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance: u32) -> VertexOutput {
    let model = objects[instance].model;

    var out: VertexOutput;
    out.position = frame.light_view_projection * model * vec4<f32>(in.position, 1.0);
    return out;
}
