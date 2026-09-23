import package::common::voxel::mip::GpuVoxelMipInfo;
import package::common::voxel::types::{GpuBrick, GpuVoxelGrid, GpuVoxelRadiance, VoxelSample};
import package::common::voxel::constants::{BRICK_SIZE, EMPTY_INDEX, MIP_LEVELS};

@group(1) @binding(0)
var<storage, read> bricks: array<GpuBrick>;

@group(1) @binding(1)
var<uniform> grid: GpuVoxelGrid;

@group(1) @binding(2)
var<storage, read> brick_lookup: array<u32>;

@group(2) @binding(0)
var<storage, read> radiance: array<GpuVoxelRadiance>;

@group(3) @binding(0)
var mip_texture: texture_3d<f32>;

@group(3) @binding(1)
var mip_sampler: sampler;

@group(3) @binding(2)
var<uniform> mip_info: GpuVoxelMipInfo;

fn empty_voxel_sample() -> VoxelSample {
    var sample: VoxelSample;
    sample.radiance = vec3<f32>(0.0);
    sample.opacity = 0.0;
    return sample;
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

fn sample_voxel(world_position: vec3<f32>) -> VoxelSample {
    let coordinate = vec3<i32>(floor(
        (world_position - grid.origin_and_voxel_size.xyz)
            / grid.origin_and_voxel_size.w
    ));
    let dimensions = vec3<i32>(grid.dimensions_and_brick_size.xyz);
    if any(coordinate < vec3<i32>(0)) || any(coordinate >= dimensions) {
        return empty_voxel_sample();
    }

    let voxel = vec3<u32>(coordinate);
    let brick_index = lookup_brick(voxel / BRICK_SIZE);
    if brick_index == EMPTY_INDEX {
        return empty_voxel_sample();
    }

    let local = voxel % BRICK_SIZE;
    let local_index = local.x
        + local.y * BRICK_SIZE
        + local.z * BRICK_SIZE * BRICK_SIZE;
    let brick = bricks[brick_index];
    let word_index = local_index / 32u;
    let bit_offset = local_index % 32u;
    if (brick.occupancy[word_index] & (1u << bit_offset)) == 0u {
        return empty_voxel_sample();
    }

    let radiance_index = brick.radiance_index[local_index];
    if radiance_index == EMPTY_INDEX {
        return empty_voxel_sample();
    }

    var sample: VoxelSample;
    sample.radiance = radiance[radiance_index].value.rgb;
    sample.opacity = 1.0;
    return sample;
}

fn mip_level_dimensions(level: u32) -> vec3<u32> {
    let info = mip_info.levels[level];
    return vec3<u32>(
        info.offset_count_dimensions.z,
        info.offset_count_dimensions.w,
        info.dimensions_and_padding.x,
    );
}

fn unflatten(index: u32, dimensions: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(
        index % dimensions.x,
        (index / dimensions.x) % dimensions.y,
        index / (dimensions.x * dimensions.y),
    );
}

fn mip_level_count(level: u32) -> u32 {
    if level >= MIP_LEVELS {
        return 0u;
    }

    return mip_info.levels[level].offset_count_dimensions.y;
}

fn voxel_mip_for_diameter(diameter: f32) -> u32 {
    let base_cell_size = mip_info.levels[0u].cell_size_and_padding.x;
    let lod = floor(log2(max(diameter, base_cell_size) / base_cell_size) + 0.5);
    return u32(clamp(lod, 0.0, f32(MIP_LEVELS - 1u)));
}

fn sample_voxel_lod(world_position: vec3<f32>, level: u32) -> VoxelSample {
    if level >= MIP_LEVELS {
        return empty_voxel_sample();
    }

    let base_cell_size = mip_info.levels[0u].cell_size_and_padding.x;
    let local = (world_position - grid.origin_and_voxel_size.xyz) / base_cell_size;
    let base_dimensions = vec3<f32>(mip_level_dimensions(0u));
    if any(local < vec3<f32>(0.0)) || any(local >= base_dimensions) {
        return empty_voxel_sample();
    }

    let coordinates = (local + vec3<f32>(0.5)) / base_dimensions;
    let value = textureSampleLevel(mip_texture, mip_sampler, coordinates, f32(level));

    var sample: VoxelSample;
    sample.radiance = select(vec3<f32>(0.0), value.rgb / value.a, value.a > 0.0001);
    sample.opacity = value.a;
    return sample;
}
