const PQ_N: f32 = 2610.0 / 4096.0 / 4.0;
const PQ_M: f32 = 2523.0 / 4096.0 * 128.0;
const PQ_C1: f32 = 3424.0 / 4096.0;
const PQ_C2: f32 = 2413.0 / 4096.0 * 32.0;
const PQ_C3: f32 = 2392.0 / 4096.0 * 32.0;
const PQ_MAX_NITS: f32 = 10000.0;

const SRGB_TO_REC2020: mat3x3<f32> = mat3x3<f32>(
    vec3<f32>(0.627404, 0.069097, 0.016391),
    vec3<f32>(0.329283, 0.919540, 0.088013),
    vec3<f32>(0.043313, 0.011362, 0.895595),
);

const REC2020_TO_SRGB: mat3x3<f32> = mat3x3<f32>(
    vec3<f32>(1.660491, -0.124551, -0.018151),
    vec3<f32>(-0.587641, 1.132900, -0.100579),
    vec3<f32>(-0.072850, -0.008349, 1.118730),
);

fn srgb_to_rec2020(color: vec3<f32>) -> vec3<f32> {
    return SRGB_TO_REC2020 * color;
}

fn rec2020_to_srgb(color: vec3<f32>) -> vec3<f32> {
    return REC2020_TO_SRGB * color;
}

fn encode(value_nits: f32) -> f32 {
    let normalized = max(value_nits, 0.0) / PQ_MAX_NITS;
    let powered = pow(normalized, PQ_N);

    return pow(
        (PQ_C1 + PQ_C2 * powered) / (1.0 + PQ_C3 * powered),
        PQ_M,
    );
}

fn decode(value: f32) -> f32 {
    let powered = pow(clamp(value, 0.0, 1.0), 1.0 / PQ_M);
    let numerator = max(powered - PQ_C1, 0.0);
    let denominator = max(PQ_C2 - PQ_C3 * powered, 0.000001);

    return pow(numerator / denominator, 1.0 / PQ_N) * PQ_MAX_NITS;
}

fn rec2020_to_ictcp(color: vec3<f32>) -> vec3<f32> {
    let lms_pq = vec3<f32>(
        encode((color.r * 1688.0 + color.g * 2146.0 + color.b * 262.0) / 4096.0),
        encode((color.r * 683.0 + color.g * 2951.0 + color.b * 462.0) / 4096.0),
        encode((color.r * 99.0 + color.g * 309.0 + color.b * 3688.0) / 4096.0),
    );

    return vec3<f32>(
        (2048.0 * lms_pq.x + 2048.0 * lms_pq.y) / 4096.0,
        (6610.0 * lms_pq.x - 13613.0 * lms_pq.y + 7003.0 * lms_pq.z) / 4096.0,
        (17933.0 * lms_pq.x - 17390.0 * lms_pq.y - 543.0 * lms_pq.z) / 4096.0,
    );
}

fn ictcp_to_rec2020(color: vec3<f32>) -> vec3<f32> {
    let linear = vec3<f32>(
        decode(color.x + 0.00860904 * color.y + 0.11103 * color.z),
        decode(color.x - 0.00860904 * color.y - 0.11103 * color.z),
        decode(color.x + 0.560031 * color.y - 0.320627 * color.z),
    );

    return max(vec3<f32>(
        3.43661 * linear.x - 2.50645 * linear.y + 0.0698454 * linear.z,
        -0.79133 * linear.x + 1.9836 * linear.y - 0.192271 * linear.z,
        -0.0259499 * linear.x - 0.0989137 * linear.y + 1.12486 * linear.z,
    ), vec3<f32>(0.0));
}
