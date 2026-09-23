struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) @invariant
    position: vec4<f32>,

    @location(0)
    @interpolate(perspective, sample)
    normal: vec3<f32>,

    @location(1)
    @interpolate(perspective, sample)
    uv: vec2<f32>,

    @location(2)
    @interpolate(perspective, sample)
    light_position: vec4<f32>,

    @location(3)
    @interpolate(perspective, sample)
    tangent: vec4<f32>,

    @location(4)
    @interpolate(perspective, sample)
    world_position: vec3<f32>,
}
