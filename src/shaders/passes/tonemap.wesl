import package::color::space::{rec2020_to_srgb, srgb_to_rec2020};
import package::color::curve::tonemap;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var scene_texture: texture_2d<f32>;

@group(0) @binding(1)
var scene_sampler: sampler;

struct ExposureState {
    exposure: f32,
};

@group(0) @binding(2)
var<storage, read> exposure_state: ExposureState;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );

    let uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = uvs[vertex_index];

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(scene_texture, scene_sampler, input.uv);
    let exposed_color = scene_color.rgb * exp2(exposure_state.exposure);
    let working_color = max(srgb_to_rec2020(exposed_color), vec3<f32>(0.0));
    let mapped_color = tonemap(working_color);
    let output_color = max(rec2020_to_srgb(mapped_color), vec3<f32>(0.0));

    return vec4<f32>(output_color, scene_color.a);
}
