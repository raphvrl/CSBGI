import package::common::frame::{Frame, lux_to_render};
import package::common::voxel::constants::BRICK_SIZE;
import package::common::voxel::types::{
    GpuBrick,
    GpuVoxel,
    GpuVoxelGrid,
    GpuVoxelRadiance,
};

const SHADOW_MAP_SIZE: f32 = 4096.0;
const SHADOW_MAP_SIZE_I: i32 = 4096;
const SHADOW_TEXEL_SIZE: f32 = 1.0 / SHADOW_MAP_SIZE;
const SHADOW_DEPTH_BIAS: f32 = 0.0002;

@group(0) @binding(0)
var<uniform> frame: Frame;

@group(0) @binding(1)
var shadow_map: texture_depth_2d;

@group(1) @binding(0)
var<storage, read> bricks: array<GpuBrick>;

@group(1) @binding(1)
var<uniform> grid: GpuVoxelGrid;

@group(1) @binding(3)
var<storage, read> occupied_voxels: array<GpuVoxel>;

@group(2) @binding(0)
var<storage, read_write> radiance: array<GpuVoxelRadiance>;

fn unpack_albedo(packed: u32) -> vec3<f32> {
    return vec3<f32>(
        f32(packed & 0xffu),
        f32((packed >> 8u) & 0xffu),
        f32((packed >> 16u) & 0xffu),
    ) / 255.0;
}

fn unpack_normal(packed: u32) -> vec3<f32> {
    let normal = vec3<f32>(
        f32(packed & 0xffu),
        f32((packed >> 8u) & 0xffu),
        f32((packed >> 16u) & 0xffu),
    ) / 255.0 * 2.0 - 1.0;

    return normalize(normal);
}

fn shadow_visibility(light_position: vec4<f32>) -> f32 {
    if light_position.w <= 0.0 {
        return 1.0;
    }

    let light_ndc = light_position.xyz / light_position.w;
    let shadow_uv = vec2<f32>(
        light_ndc.x * 0.5 + 0.5,
        -light_ndc.y * 0.5 + 0.5,
    );

    if shadow_uv.x < 0.0 || shadow_uv.x > 1.0
        || shadow_uv.y < 0.0 || shadow_uv.y > 1.0
        || light_ndc.z < 0.0 || light_ndc.z > 1.0
    {
        return 1.0;
    }

    let depth = max(light_ndc.z - SHADOW_DEPTH_BIAS, 0.0);
    let center = clamp(
        vec2<i32>(shadow_uv * SHADOW_MAP_SIZE),
        vec2<i32>(0),
        vec2<i32>(SHADOW_MAP_SIZE_I - 1),
    );
    var result = 0.0;

    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let sample_position = clamp(
                center + vec2<i32>(x, y),
                vec2<i32>(0),
                vec2<i32>(SHADOW_MAP_SIZE_I - 1),
            );
            let sample_depth = textureLoad(shadow_map, sample_position, 0);

            if depth <= sample_depth {
                result += 1.0;
            }
        }
    }

    return result / 9.0;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let occupied_index = global_id.x;

    if occupied_index >= arrayLength(&occupied_voxels) {
        return;
    }

    let voxel = occupied_voxels[occupied_index];
    let brick_index = voxel.brick_index_and_voxel_index.x;
    let voxel_index = voxel.brick_index_and_voxel_index.y;
    let brick = bricks[brick_index];

    let local_voxel = vec3<u32>(
        voxel_index % BRICK_SIZE,
        (voxel_index / BRICK_SIZE) % BRICK_SIZE,
        voxel_index / (BRICK_SIZE * BRICK_SIZE),
    );
    let voxel_size = grid.origin_and_voxel_size.w;
    let brick_size = f32(grid.dimensions_and_brick_size.w) * voxel_size;
    let brick_origin =
        grid.origin_and_voxel_size.xyz + vec3<f32>(brick.key.xyz) * brick_size;
    let voxel_center =
        brick_origin + (vec3<f32>(local_voxel) + vec3<f32>(0.5)) * voxel_size;

    let normal = unpack_normal(brick.normal[voxel_index]);
    let albedo = unpack_albedo(brick.albedo[voxel_index]);
    let direction_to_light = -normalize(frame.light.light_direction);
    let ndotl = max(dot(normal, direction_to_light), 0.0);
    let shadow_position = voxel_center + normal * voxel_size * 0.5;
    let light_position =
        frame.light.view_projection * vec4<f32>(shadow_position, 1.0);
    let visibility = shadow_visibility(light_position);
    let intensity = lux_to_render(frame.light.light_illuminance_lux);
    let direct_light =
        albedo * frame.light.light_color * intensity * ndotl * visibility;

    radiance[occupied_index].value = vec4<f32>(direct_light, 1.0);
}
