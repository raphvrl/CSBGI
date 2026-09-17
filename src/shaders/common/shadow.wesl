const MAP_SIZE: f32 = 4096.0;
const TEXEL_SIZE: f32 = 1.0 / MAP_SIZE;
const RECEIVER_DEPTH_BIAS: f32 = 0.0002;

fn sample(
    shadow_map: texture_depth_2d,
    shadow_sampler: sampler_comparison,
    uv: vec2<f32>,
    depth: f32,
    offset: vec2<f32>,
) -> f32 {
    return textureSampleCompare(
        shadow_map,
        shadow_sampler,
        uv + offset * TEXEL_SIZE,
        depth,
    );
}

fn visibility(
    shadow_map: texture_depth_2d,
    shadow_sampler: sampler_comparison,
    light_position: vec4<f32>,
) -> f32 {
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

    let depth = max(light_ndc.z - RECEIVER_DEPTH_BIAS, 0.0);
    var result = 0.0;

    result += sample(shadow_map, shadow_sampler, shadow_uv, depth, vec2<f32>(-1.0, -1.0));
    result += sample(shadow_map, shadow_sampler, shadow_uv, depth, vec2<f32>(0.0, -1.0));
    result += sample(shadow_map, shadow_sampler, shadow_uv, depth, vec2<f32>(1.0, -1.0));
    result += sample(shadow_map, shadow_sampler, shadow_uv, depth, vec2<f32>(-1.0, 0.0));
    result += sample(shadow_map, shadow_sampler, shadow_uv, depth, vec2<f32>(0.0, 0.0));
    result += sample(shadow_map, shadow_sampler, shadow_uv, depth, vec2<f32>(1.0, 0.0));
    result += sample(shadow_map, shadow_sampler, shadow_uv, depth, vec2<f32>(-1.0, 1.0));
    result += sample(shadow_map, shadow_sampler, shadow_uv, depth, vec2<f32>(0.0, 1.0));
    result += sample(shadow_map, shadow_sampler, shadow_uv, depth, vec2<f32>(1.0, 1.0));

    return result / 9.0;
}
