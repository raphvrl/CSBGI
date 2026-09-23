import package::common::frame::Frame;
import package::common::voxel::sampler::{
    bricks,
    brick_lookup,
    grid,
    lookup_brick,
    mip_info,
    mip_level_count,
    mip_level_dimensions,
    radiance,
    sample_voxel,
    sample_voxel_lod,
    unflatten,
    VoxelSample,
};
import package::common::voxel::constants::{BRICK_SIZE, EMPTY_INDEX};
import package::common::voxel::types::{GpuVoxel, GpuVoxelFace};

struct MipmapDebugSettings {
    level: u32,
    padding: vec3<u32>,
};

@group(0) @binding(0)
var<uniform> frame: Frame;

@group(1) @binding(3)
var<storage, read> occupied_voxels: array<GpuVoxel>;

@group(1) @binding(4)
var<storage, read> visible_faces: array<GpuVoxelFace>;

@group(3) @binding(3)
var<uniform> mipmap_debug: MipmapDebugSettings;

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,

    @location(0)
    color: vec4<f32>,
};

const CUBE_EDGES = array<vec3<f32>, 24>(
    // X edges
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 0.0),
    vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 0.0),
    vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(1.0, 0.0, 1.0),
    vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(1.0, 1.0, 1.0),

    // Y edges
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 0.0),
    vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 1.0),
    vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(1.0, 1.0, 1.0),

    // Z edges
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 1.0),
    vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 1.0),
    vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 1.0, 1.0),
    vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 1.0),
);

const CUBE_TRIANGLES = array<vec3<f32>, 36>(
      // Front face
      vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0),
      vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 0.0),

      // Back face
      vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(1.0, 1.0, 1.0),
      vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(0.0, 1.0, 1.0),

    // Left face
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 1.0),
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(0.0, 1.0, 0.0),

    // Right face
    vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(1.0, 0.0, 1.0),
    vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 1.0),

    // Bottom face
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(0.0, 0.0, 1.0),
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 1.0),

    // Top face
    vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(1.0, 1.0, 0.0),
);

fn unpack_albedo(packed: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(packed & 0xffu),
        f32((packed >> 8u) & 0xffu),
        f32((packed >> 16u) & 0xffu),
        f32((packed >> 24u) & 0xffu),
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

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var output: VertexOutput;
    output.color = vec4<f32>(0.1, 0.8, 0.2, 0.85);

    let source_brick = bricks[instance_index];
    let brick_index = lookup_brick(source_brick.key.xyz);

    if (brick_index == EMPTY_INDEX) {
        output.position = vec4<f32>(2.0, 2.0, 2.0, 1.0);
        return output;
    }

    let brick = bricks[brick_index];
    let brick_size = f32(grid.dimensions_and_brick_size.w) * grid.origin_and_voxel_size.w;
    let brick_origin = grid.origin_and_voxel_size.xyz + vec3<f32>(brick.key.xyz) * brick_size;
    let world_position = brick_origin + CUBE_EDGES[vertex_index] * brick_size;

    output.position = frame.camera.view_projection * vec4<f32>(world_position, 1.0);
    return output;
}

@vertex
fn vs_voxel(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let voxel = occupied_voxels[instance_index];
    let brick_index = voxel.brick_index_and_voxel_index.x;
    let voxel_index = voxel.brick_index_and_voxel_index.y;
    var output: VertexOutput;
    output.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    let brick = bricks[brick_index];
    let word_index = voxel_index / 32u;
    let bit_offset = voxel_index % 32u;
    let occupied = (brick.occupancy[word_index] & (1u << bit_offset)) != 0u;

    if (!occupied) {
        output.position = vec4<f32>(2.0, 2.0, 2.0, 1.0);
        return output;
    }

    let local_voxel = vec3<u32>(
        voxel_index % BRICK_SIZE,
        (voxel_index / BRICK_SIZE) % BRICK_SIZE,
        voxel_index / (BRICK_SIZE * BRICK_SIZE),
    );
    let voxel_size = grid.origin_and_voxel_size.w;
    let brick_size = f32(grid.dimensions_and_brick_size.w) * voxel_size;
    let brick_origin = grid.origin_and_voxel_size.xyz + vec3<f32>(brick.key.xyz) * brick_size;
    let voxel_origin = brick_origin + vec3<f32>(local_voxel) * voxel_size;
    let world_position = voxel_origin + CUBE_EDGES[vertex_index] * voxel_size;

    output.position = frame.camera.view_projection * vec4<f32>(world_position, 1.0);

    return output;
}

@vertex
fn vs_voxel_mipmap(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let level = mipmap_debug.level;
    var output: VertexOutput;
    output.color = vec4<f32>(0.0);

    if level > 7u || instance_index >= mip_level_count(level) {
        output.position = vec4<f32>(2.0, 2.0, 2.0, 1.0);
        return output;
    }

    var cell_origin = grid.origin_and_voxel_size.xyz;
    let cell_size = mip_info.levels[level].cell_size_and_padding.x;
    let dimensions = mip_level_dimensions(level);
    let local = unflatten(instance_index, dimensions);
    cell_origin += vec3<f32>(local) * cell_size;

    let sample_position = cell_origin + vec3<f32>(0.5) * cell_size;
    let sample = sample_voxel_lod(sample_position, level);
    if sample.opacity <= 0.001 {
        output.position = vec4<f32>(2.0, 2.0, 2.0, 1.0);
        return output;
    }

    output.color = vec4<f32>(sample.radiance.rgb, 0.85);
    let world_position = cell_origin + CUBE_EDGES[vertex_index] * cell_size;
    output.position = frame.camera.view_projection * vec4<f32>(world_position, 1.0);
    return output;
}

@vertex
fn vs_voxel_albedo(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let face = visible_faces[instance_index];
    let brick_index = face.brick_index_and_voxel_index_and_face.x;
    let voxel_index = face.brick_index_and_voxel_index_and_face.y;
    let face_index = face.brick_index_and_voxel_index_and_face.z;
    var output: VertexOutput;

    let brick = bricks[brick_index];

    output.color = unpack_albedo(brick.albedo[voxel_index]);

    let local_voxel = vec3<u32>(
        voxel_index % BRICK_SIZE,
        (voxel_index / BRICK_SIZE) % BRICK_SIZE,
        voxel_index / (BRICK_SIZE * BRICK_SIZE),
    );
    let voxel_size = grid.origin_and_voxel_size.w;
    let brick_size = f32(grid.dimensions_and_brick_size.w) * voxel_size;
    let brick_origin = grid.origin_and_voxel_size.xyz + vec3<f32>(brick.key.xyz) * brick_size;
    let voxel_origin = brick_origin + vec3<f32>(local_voxel) * voxel_size;
    let triangle_vertex = face_index * 6u + vertex_index;
    let world_position = voxel_origin + CUBE_TRIANGLES[triangle_vertex] * voxel_size;

    output.position = frame.camera.view_projection * vec4<f32>(world_position, 1.0);

    return output;
}

@vertex
fn vs_voxel_normal(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let face = visible_faces[instance_index];
    let brick_index = face.brick_index_and_voxel_index_and_face.x;
    let voxel_index = face.brick_index_and_voxel_index_and_face.y;
    let face_index = face.brick_index_and_voxel_index_and_face.z;
    var output: VertexOutput;

    let brick = bricks[brick_index];
    let normal = unpack_normal(brick.normal[voxel_index]);
    output.color = vec4<f32>(normal * 0.5 + 0.5, 1.0);

    let local_voxel = vec3<u32>(
        voxel_index % BRICK_SIZE,
        (voxel_index / BRICK_SIZE) % BRICK_SIZE,
        voxel_index / (BRICK_SIZE * BRICK_SIZE),
    );
    let voxel_size = grid.origin_and_voxel_size.w;
    let brick_size = f32(grid.dimensions_and_brick_size.w) * voxel_size;
    let brick_origin = grid.origin_and_voxel_size.xyz + vec3<f32>(brick.key.xyz) * brick_size;
    let voxel_origin = brick_origin + vec3<f32>(local_voxel) * voxel_size;
    let triangle_vertex = face_index * 6u + vertex_index;
    let world_position = voxel_origin + CUBE_TRIANGLES[triangle_vertex] * voxel_size;

    output.position = frame.camera.view_projection * vec4<f32>(world_position, 1.0);

    return output;
}

@vertex
fn vs_voxel_radiance(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let face = visible_faces[instance_index];
    let brick_index = face.brick_index_and_voxel_index_and_face.x;
    let voxel_index = face.brick_index_and_voxel_index_and_face.y;
    let face_index = face.brick_index_and_voxel_index_and_face.z;
    var output: VertexOutput;

    let brick = bricks[brick_index];

    let local_voxel = vec3<u32>(
        voxel_index % BRICK_SIZE,
        (voxel_index / BRICK_SIZE) % BRICK_SIZE,
        voxel_index / (BRICK_SIZE * BRICK_SIZE),
    );
    let voxel_size = grid.origin_and_voxel_size.w;
    let brick_size = f32(grid.dimensions_and_brick_size.w) * voxel_size;
    let brick_origin = grid.origin_and_voxel_size.xyz + vec3<f32>(brick.key.xyz) * brick_size;
    let voxel_center = brick_origin + (vec3<f32>(local_voxel) + vec3<f32>(0.5)) * voxel_size;
    let sample = sample_voxel(voxel_center);
    output.color = vec4<f32>(sample.radiance, sample.opacity);
    let voxel_origin = brick_origin + vec3<f32>(local_voxel) * voxel_size;
    let triangle_vertex = face_index * 6u + vertex_index;
    let world_position = voxel_origin + CUBE_TRIANGLES[triangle_vertex] * voxel_size;

    output.position = frame.camera.view_projection * vec4<f32>(world_position, 1.0);

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
