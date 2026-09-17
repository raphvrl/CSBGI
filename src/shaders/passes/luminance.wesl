struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0)
var scene_texture: texture_2d<f32>;

@group(0) @binding(1)
var scene_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(input.position.xy);
    let color = max(textureLoad(scene_texture, pixel, 0u).rgb, vec3<f32>(0.0));
    let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));

    return vec4<f32>(log2(max(luminance, 0.0001)), 0.0, 0.0, 1.0);
}
