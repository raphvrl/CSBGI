struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

struct FrameUniforms {
    view_projection: mat4x4<f32>,
    camera_position: vec3<f32>,
    _pad: f32,
};

@group(0) @binding(0)
var<uniform> frame: FrameUniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let view_projection = frame.view_projection;

    var out: VertexOutput;
    out.position = view_projection * vec4<f32>(in.position, 1.0);
    out.normal = in.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    let color = normal * 0.5 + vec3<f32>(0.5);

    return vec4<f32>(color, 1.0);
}
