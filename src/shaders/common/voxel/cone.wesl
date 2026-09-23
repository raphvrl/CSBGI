import package::common::voxel::geometry_sampler::{
    grid,
    mip_info,
    sample_voxel_lod,
    trace_settings,
    voxel_lod_for_diameter,
};

const TAU: f32 = 6.28318530718;

const VCT_MAX_CONES: u32 = 6u;
const VCT_STEP_FACTOR: f32 = 0.5;
const VCT_START_DISTANCE_VOXELS: f32 = 2.0;
const VCT_MIN_DIAMETER_VOXELS: f32 = 4.0;
const VCT_MIN_TRANSMITTANCE: f32 = 0.02;
const VCT_NORMAL_OFFSET_VOXELS: f32 = 1.5;
const VCT_RING_TILT: f32 = TAU / 8.0;
const VCT_CENTER_WEIGHT: f32 = 0.5;
const VCT_POLE_THRESHOLD: f32 = 0.99;

fn trace_cone(
    origin: vec3<f32>,
    direction: vec3<f32>,
) -> vec3<f32> {
    let voxel_size = grid.origin_and_voxel_size.w;
    let base_cell_size = mip_info.levels[0u].cell_size_and_padding.x;
    let diameter_per_distance = 2.0 * tan(trace_settings.cone_angle * 0.5);
    let min_diameter = voxel_size * VCT_MIN_DIAMETER_VOXELS;
    var distance = voxel_size * VCT_START_DISTANCE_VOXELS;
    var transmittance = 1.0;
    var result = vec3<f32>(0.0);

    while distance < trace_settings.max_distance && transmittance >= VCT_MIN_TRANSMITTANCE {
        let diameter = max(distance * diameter_per_distance, min_diameter);
        let lod = voxel_lod_for_diameter(diameter);
        let sample = sample_voxel_lod(origin + direction * distance, lod);
        let step_length = diameter * VCT_STEP_FACTOR;

        let cell_size = base_cell_size * exp2(lod);
        let opacity = 1.0 - pow(1.0 - sample.opacity, step_length / cell_size);

        result += transmittance * sample.radiance * opacity;
        transmittance *= 1.0 - opacity;

        distance += step_length;
    }

    return result;
}

fn trace_diffuse(
    world_position: vec3<f32>,
    normal_input: vec3<f32>,
) -> vec3<f32> {
    let normal = normalize(normal_input);
    let reference = select(
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(1.0, 0.0, 0.0),
        abs(normal.y) > VCT_POLE_THRESHOLD,
    );

    let tangent = normalize(cross(reference, normal));
    let bitangent = normalize(cross(normal, tangent));
    let voxel_size = grid.origin_and_voxel_size.w;
    let origin = world_position + normal * voxel_size * VCT_NORMAL_OFFSET_VOXELS;

    let center = trace_cone(origin, normal);

    let ring_count = clamp(
        max(trace_settings.cone_count, 1u) - 1u,
        1u,
        VCT_MAX_CONES - 1u,
    );
    let angle_step = TAU / f32(ring_count);
    let ring_axis = normal * cos(VCT_RING_TILT);
    let ring_radius = sin(VCT_RING_TILT);
    let ring_weight = (1.0 - VCT_CENTER_WEIGHT) / f32(ring_count);

    var indirect = center * VCT_CENTER_WEIGHT;

    for (var ring_index = 0u; ring_index < ring_count; ring_index++) {
        let angle = f32(ring_index) * angle_step;
        let direction = ring_axis
            + (tangent * cos(angle) + bitangent * sin(angle)) * ring_radius;

        indirect += trace_cone(origin, direction) * ring_weight;
    }

    return indirect * trace_settings.intensity;
}
