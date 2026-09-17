struct ExposureState {
    exposure: f32,
};

struct ExposureParams {
    target_luminance: f32,
    min_exposure: f32,
    max_exposure: f32,
    adaptation_speed: f32,
    delta_time: f32,
    camera_exposure_stops: f32,
};

@group(0) @binding(0)
var luminance_texture: texture_2d<f32>;

@group(0) @binding(1)
var<storage, read_write> exposure_state: ExposureState;

@group(0) @binding(2)
var<uniform> params: ExposureParams;

@compute @workgroup_size(1)
fn cs_main() {
    let log_luminance = textureLoad(luminance_texture, vec2<i32>(0, 0), 0).r;
    let average_luminance = exp2(log_luminance);
    let target_exposure = clamp(
        log2(params.target_luminance / max(average_luminance, 0.0001)),
        params.min_exposure,
        params.max_exposure,
    ) + params.camera_exposure_stops;

    let adaptation = 1.0 - exp(-params.adaptation_speed * params.delta_time);
    exposure_state.exposure +=
        (target_exposure - exposure_state.exposure) * adaptation;
}
