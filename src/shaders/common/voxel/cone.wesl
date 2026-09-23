import package::common::voxel::geometry_sampler::{
    grid,
    mip_info,
    sample_voxel_lod,
    trace_settings,
    voxel_lod_for_diameter,
};
import package::common::voxel::constants::MIP_LEVELS;

const VCT_MAX_CONES: u32 = 6u;
const VCT_MAX_STEPS: u32 = 8u;

fn trace_cone(
    origin: vec3<f32>,
    direction: vec3<f32>,
) -> vec3<f32> {
    let voxel_size = grid.origin_and_voxel_size.w;
    var distance = voxel_size * 2.0;
    var transmittance = 1.0;
    var result = vec3<f32>(0.0);

    for (var step = 0u; step < VCT_MAX_STEPS; step++) {
        if distance >= trace_settings.max_distance
            || step >= trace_settings.max_steps
            || transmittance < 0.02 {
            break;
        }

        let diameter = max(
            distance * 2.0 * tan(trace_settings.cone_angle * 0.5),
            voxel_size * 4.0,
        );
        let lod = voxel_lod_for_diameter(diameter);
        let sample = sample_voxel_lod(origin + direction * distance, lod);

        result += transmittance * sample.radiance * sample.opacity;
        transmittance *= 1.0 - sample.opacity;

        let lower_level = u32(floor(lod));
        let upper_level = min(lower_level + 1u, MIP_LEVELS - 1u);

        let mip_step = mix(
            mip_info.levels[lower_level].cell_size_and_padding.x,
            mip_info.levels[upper_level].cell_size_and_padding.x,
            fract(lod),
        );

        distance += max(mip_step, voxel_size);
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
        abs(normal.y) > 0.99,
    );

    let tangent = normalize(cross(reference, normal));
    let bitangent = normalize(cross(normal, tangent));
    let origin = world_position + normal * grid.origin_and_voxel_size.w * 1.5;
    let side = 0.70710678;

    let center = trace_cone(origin, normal);

    let ring_directions = array<vec2<f32>, 5>(
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.30901699, 0.95105654),
        vec2<f32>(-0.80901699, 0.58778524),
        vec2<f32>(-0.80901699, -0.58778524),
        vec2<f32>(0.30901699, -0.95105654),
    );

    let ring_count = clamp(
        max(trace_settings.cone_count, 1u) - 1u,
        1u,
        VCT_MAX_CONES - 1u,
    );

    var indirect = center * 0.5;

    for (var ring_index = 0u; ring_index < VCT_MAX_CONES - 1u; ring_index++) {
        if ring_index >= ring_count {
            break;
        }

        let ring_direction = ring_directions[ring_index];
        let direction = normalize(
            normal * side + (tangent * ring_direction.x + bitangent * ring_direction.y) * side,
        );

        indirect += trace_cone(origin, direction) * (0.5 / f32(ring_count));
    }

    return indirect * trace_settings.intensity;
}
