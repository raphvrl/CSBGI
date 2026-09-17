import package::color::space::{ictcp_to_rec2020, rec2020_to_ictcp};

const REFERENCE_LUMINANCE: f32 = 100.0;
const DISPLAY_TARGET_NITS: f32 = 250.0;
const FRAMEBUFFER_TARGET: f32 = DISPLAY_TARGET_NITS / REFERENCE_LUMINANCE;
const SDR_CORRECTION: f32 = 1.0 / FRAMEBUFFER_TARGET;

const GRAY_POINT: f32 = 0.538;
const LINEAR_SECTION: f32 = 0.444;
const TOE_STRENGTH: f32 = 1.280;
const BLEND_RATIO: f32 = 0.6;
const FADE_START: f32 = 0.98;
const FADE_END: f32 = 1.16;
const TARGET_ICTCP_LUMINANCE: f32 = 0.602559155;
const CURVE_KA: f32 = 2.9633333333;
const CURVE_KB: f32 = -3.3733512381;
const CURVE_KC: f32 = -0.5395683453;

fn curve_value(x: f32) -> f32 {
    if x < 0.0 {
        return 0.0;
    }

    let linear_weight = smoothstep(0.0, GRAY_POINT, x);
    let toe_weight = 1.0 - linear_weight;
    let toe = GRAY_POINT * pow(x / GRAY_POINT, TOE_STRENGTH);

    if x < LINEAR_SECTION * FRAMEBUFFER_TARGET {
        return toe_weight * toe + linear_weight * x;
    }

    return CURVE_KA + CURVE_KB * exp(x * CURVE_KC);
}

fn curve_rgb(color: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(curve_value(color.r), curve_value(color.g), curve_value(color.b));
}

fn tonemap(color: vec3<f32>) -> vec3<f32> {
    let original_ictcp = rec2020_to_ictcp(color * REFERENCE_LUMINANCE);
    let curve_rgb = curve_rgb(color);
    let curve_ictcp = rec2020_to_ictcp(curve_rgb * REFERENCE_LUMINANCE);

    let chroma_fade = 1.0 - smoothstep(
        FADE_START,
        FADE_END,
        original_ictcp.x / TARGET_ICTCP_LUMINANCE,
    );

    let scaled_rgb = ictcp_to_rec2020(vec3<f32>(
        curve_ictcp.x,
        original_ictcp.y * chroma_fade,
        original_ictcp.z * chroma_fade,
    )) / REFERENCE_LUMINANCE;

    let blended = mix(curve_rgb, scaled_rgb, BLEND_RATIO);

    return SDR_CORRECTION * min(blended, vec3<f32>(FRAMEBUFFER_TARGET));
}
