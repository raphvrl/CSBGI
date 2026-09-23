import package::common::voxel::mip::GpuVoxelMipInfo;
import package::common::voxel::types::{GpuVoxelGrid, GpuVoxelTraceSettings, VoxelSample};
import package::common::voxel::constants::MIP_LEVELS;

@group(3) @binding(0)
var mip_texture: texture_3d<f32>;

@group(3) @binding(1)
var mip_sampler: sampler;

@group(3) @binding(2)
var<uniform> mip_info: GpuVoxelMipInfo;

@group(3) @binding(3)
var<uniform> grid: GpuVoxelGrid;

@group(3) @binding(4)
var<uniform> trace_settings: GpuVoxelTraceSettings;

fn empty_voxel_sample() -> VoxelSample {
    var sample: VoxelSample;
    sample.radiance = vec3<f32>(0.0);
    sample.opacity = 0.0;
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

fn voxel_lod_for_diameter(diameter: f32) -> f32 {
    let base_cell_size = mip_info.levels[0u].cell_size_and_padding.x;
    return clamp(
        log2(max(diameter, base_cell_size) / base_cell_size),
        0.0,
        f32(MIP_LEVELS - 1u),
    );
}

fn sample_voxel_lod(world_position: vec3<f32>, lod: f32) -> VoxelSample {
    let local = (world_position - grid.origin_and_voxel_size.xyz)
        / mip_info.levels[0u].cell_size_and_padding.x;
    let base_dimensions = vec3<f32>(mip_level_dimensions(0u));
    if any(local < vec3<f32>(0.0)) || any(local >= base_dimensions) {
        return empty_voxel_sample();
    }

    let coordinates = (local + vec3<f32>(0.5)) / base_dimensions;
    let value = textureSampleLevel(mip_texture, mip_sampler, coordinates, lod);
    let opacity = value.a;

    var sample: VoxelSample;
    sample.radiance = select(vec3<f32>(0.0), value.rgb / opacity, opacity > 0.0001);
    sample.opacity = opacity;
    return sample;
}
