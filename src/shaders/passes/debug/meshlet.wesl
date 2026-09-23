import package::common::frame::Frame;
import package::common::alpha::passes_alpha_test;
import package::common::material::Material;
import package::common::object::ObjectUniform;

@group(0) @binding(0)
var<uniform> frame: Frame;

@group(1) @binding(0)
var<storage, read> objects: array<ObjectUniform>;

@group(2) @binding(0)
var<uniform> material: Material;

@group(2) @binding(1)
var base_color_texture: texture_2d<f32>;

@group(2) @binding(4)
var material_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance: u32) -> VertexOutput {
    let model = objects[instance].model;

    var output: VertexOutput;
    output.position = frame.camera.view_projection * model * vec4<f32>(in.position, 1.0);
    output.color = in.color;
    output.uv = in.uv;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(base_color_texture, material_sampler, in.uv);

    let base_color = tex * material.base_color_factor;

    if !passes_alpha_test(base_color.a, material) {
        discard;
    }

    return vec4<f32>(in.color, 1.0);
}
