import package::common::voxel::mip::GpuVoxelMipInfo;

@group(0) @binding(0)
var input_mip: texture_3d<f32>;

@group(0) @binding(1)
var output_mip: texture_storage_3d<rgba16float, write>;

@group(0) @binding(2)
var<uniform> mip_info: GpuVoxelMipInfo;

fn aggregate_children(
    first_child: vec3<u32>,
    input_dimensions: vec3<u32>,
) -> vec4<f32> {
    var radiance_sum = vec3<f32>(0.0);
    var opacity_sum = 0.0;

    for (var z = 0u; z < 2u; z++) {
        for (var y = 0u; y < 2u; y++) {
            for (var x = 0u; x < 2u; x++) {
                let child_position = first_child + vec3<u32>(x, y, z);
                if any(child_position >= input_dimensions) {
                    continue;
                }

                let child = textureLoad(input_mip, vec3<i32>(child_position), 0);
                radiance_sum += child.rgb;
                opacity_sum += child.a;
            }
        }
    }

    // Store a premultiplied average so trilinear filtering does not amplify light.
    return vec4<f32>(radiance_sum / 8.0, opacity_sum / 8.0);
}

fn write_mip(
    global_id: vec3<u32>,
    input_level: u32,
    output_level: u32,
) {
    let output_info = mip_info.levels[output_level];
    let output_index = global_id.x;
    if output_index >= output_info.offset_count_dimensions.y {
        return;
    }

    let input_info = mip_info.levels[input_level];
    let input_dimensions = vec3<u32>(
        input_info.offset_count_dimensions.z,
        input_info.offset_count_dimensions.w,
        input_info.dimensions_and_padding.x,
    );
    let output_dimensions = vec3<u32>(
        output_info.offset_count_dimensions.z,
        output_info.offset_count_dimensions.w,
        output_info.dimensions_and_padding.x,
    );
    let parent = vec3<u32>(
        output_index % output_dimensions.x,
        (output_index / output_dimensions.x) % output_dimensions.y,
        output_index / (output_dimensions.x * output_dimensions.y),
    );

    textureStore(
        output_mip,
        vec3<i32>(parent),
        aggregate_children(parent * 2u, input_dimensions),
    );
}

@compute @workgroup_size(64)
fn cs_mip_0_to_1(@builtin(global_invocation_id) global_id: vec3<u32>) {
    write_mip(global_id, 0u, 1u);
}

@compute @workgroup_size(64)
fn cs_mip_1_to_2(@builtin(global_invocation_id) global_id: vec3<u32>) {
    write_mip(global_id, 1u, 2u);
}

@compute @workgroup_size(64)
fn cs_mip_2_to_3(@builtin(global_invocation_id) global_id: vec3<u32>) {
    write_mip(global_id, 2u, 3u);
}

@compute @workgroup_size(64)
fn cs_mip_3_to_4(@builtin(global_invocation_id) global_id: vec3<u32>) {
    write_mip(global_id, 3u, 4u);
}

@compute @workgroup_size(64)
fn cs_mip_4_to_5(@builtin(global_invocation_id) global_id: vec3<u32>) {
    write_mip(global_id, 4u, 5u);
}

@compute @workgroup_size(64)
fn cs_mip_5_to_6(@builtin(global_invocation_id) global_id: vec3<u32>) {
    write_mip(global_id, 5u, 6u);
}

@compute @workgroup_size(64)
fn cs_mip_6_to_7(@builtin(global_invocation_id) global_id: vec3<u32>) {
    write_mip(global_id, 6u, 7u);
}
