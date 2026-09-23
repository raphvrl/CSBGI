import package::common::frame::Frame;
import package::common::material::Material;
import package::common::alpha::passes_alpha_test;
import package::common::object::ObjectUniform;
import package::common::vertex::VertexInput;

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,

    @location(0)
    uv: vec2<f32>,
};

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

@vertex
fn vs_main(input: VertexInput, @builtin(instance_index) instance: u32) -> VertexOutput {
    let model = objects[instance].model;

    var output: VertexOutput;
    output.position = frame.light.view_projection * model * vec4<f32>(input.position, 1.0);
    output.uv = input.uv;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) {
    let base_color = textureSample(
        base_color_texture,
        material_sampler,
        input.uv,
    ) * material.base_color_factor;

    if !passes_alpha_test(base_color.a, material) {
        discard;
    }
}
