import package::common::voxel::mip::GpuVoxelMipInfo;
import package::common::voxel::types::{GpuBrick, GpuVoxelGrid, GpuVoxelRadiance};
import package::common::voxel::constants::{BRICK_SIZE, EMPTY_INDEX, MIP_BASE_VOXELS};

@group(0) @binding(0)
var<storage, read> bricks: array<GpuBrick>;

@group(0) @binding(1)
var<uniform> grid: GpuVoxelGrid;

@group(0) @binding(2)
var<storage, read> brick_lookup: array<u32>;

@group(1) @binding(0)
var<storage, read> base_radiance: array<GpuVoxelRadiance>;

@group(1) @binding(1)
var output_mip: texture_storage_3d<rgba16float, write>;

@group(1) @binding(2)
var<uniform> mip_info: GpuVoxelMipInfo;

fn empty_sample() -> vec4<f32> {
    return vec4<f32>(0.0);
}

fn lookup_brick(key: vec3<u32>) -> u32 {
    let dimensions = grid.brick_dimensions_and_padding.xyz;
    if any(key >= dimensions) {
        return EMPTY_INDEX;
    }

    let index = key.x
        + key.y * dimensions.x
        + key.z * dimensions.x * dimensions.y;
    return brick_lookup[index];
}

fn sample_voxel(voxel: vec3<u32>) -> vec4<f32> {
    if any(voxel >= grid.dimensions_and_brick_size.xyz) {
        return empty_sample();
    }

    let brick_index = lookup_brick(voxel / BRICK_SIZE);
    if brick_index == EMPTY_INDEX {
        return empty_sample();
    }

    let local = voxel % BRICK_SIZE;
    let local_index = local.x
        + local.y * BRICK_SIZE
        + local.z * BRICK_SIZE * BRICK_SIZE;
    let brick = bricks[brick_index];
    let word_index = local_index / 32u;
    let bit_offset = local_index % 32u;
    if (brick.occupancy[word_index] & (1u << bit_offset)) == 0u {
        return empty_sample();
    }

    let radiance_index = brick.radiance_index[local_index];
    if radiance_index == EMPTY_INDEX {
        return empty_sample();
    }

    let value = base_radiance[radiance_index].value;
    return vec4<f32>(value.rgb, 1.0);
}

fn aggregate_base(first_voxel: vec3<u32>) -> vec4<f32> {
    var radiance_sum = vec3<f32>(0.0);
    var opacity_sum = 0.0;

    for (var z = 0u; z < MIP_BASE_VOXELS; z++) {
        for (var y = 0u; y < MIP_BASE_VOXELS; y++) {
            for (var x = 0u; x < MIP_BASE_VOXELS; x++) {
                let child = sample_voxel(first_voxel + vec3<u32>(x, y, z));
                radiance_sum += child.rgb * child.a;
                opacity_sum += child.a;
            }
        }
    }

    let sample_count = f32(MIP_BASE_VOXELS * MIP_BASE_VOXELS * MIP_BASE_VOXELS);

    // Store a premultiplied average so every mip level preserves the same energy.
    return vec4<f32>(radiance_sum / sample_count, opacity_sum / sample_count);
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_info = mip_info.levels[0u];
    let output_index = global_id.x;
    if output_index >= output_info.offset_count_dimensions.y {
        return;
    }

    let dimensions = vec3<u32>(
        output_info.offset_count_dimensions.z,
        output_info.offset_count_dimensions.w,
        output_info.dimensions_and_padding.x,
    );
    let cell = vec3<u32>(
        output_index % dimensions.x,
        (output_index / dimensions.x) % dimensions.y,
        output_index / (dimensions.x * dimensions.y),
    );
    textureStore(output_mip, vec3<i32>(cell), aggregate_base(cell * MIP_BASE_VOXELS));
}
