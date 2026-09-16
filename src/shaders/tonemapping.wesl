struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var scene_texture: texture_2d<f32>;

@group(0) @binding(1)
var scene_sampler: sampler;

fn aces_tonemap(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;

    return clamp(
        (color * (a * color + b)) /
        (color * (c * color + d) + e),
        vec3<f32>(0.0),
        vec3<f32>(1.0),
    );
}

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

    output.position = vec4<f32>(
        positions[vertex_index],
        0.0,
        1.0,
    );

    output.uv = uvs[vertex_index];

    return output;
}

const EXPOSURE: f32 = 1.0;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(
        scene_texture,
        scene_sampler,
        input.uv,
    );

    let exposed_color = scene_color.rgb * exp2(EXPOSURE);

    return vec4<f32>(
        aces_tonemap(exposed_color),
        scene_color.a,
    );
}
