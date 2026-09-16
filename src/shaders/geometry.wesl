struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) light_position: vec4<f32>,
    @location(3) tangent: vec4<f32>,
};

struct Frame {
    view_projection: mat4x4<f32>,
    light_view_projection: mat4x4<f32>,
    camera_position: vec3<f32>,
    pad0: f32,
    light_direction: vec3<f32>,
    light_intensity: f32,
    light_color: vec3<f32>,
    ambient: f32,
};

@group(0) @binding(0)
var<uniform> frame: Frame;

@group(0) @binding(1)
var shadow_map: texture_depth_2d;

@group(0) @binding(2)
var shadow_sampler: sampler_comparison;

struct ObjectUniform {
    model: mat4x4<f32>,
};

@group(1) @binding(0)
var<storage, read> objects: array<ObjectUniform>;

struct Material {
    base_color_factor: vec4<f32>,
    normal_scale: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: Material;

@group(2) @binding(1)
var base_color_texture: texture_2d<f32>;

@group(2) @binding(2)
var normal_texture: texture_2d<f32>;

@group(2) @binding(3)
var material_sampler: sampler;

fn normal_matrix(matrix: mat3x3<f32>) -> mat3x3<f32> {
    let cofactor0 = cross(matrix[1], matrix[2]);
    let cofactor1 = cross(matrix[2], matrix[0]);
    let cofactor2 = cross(matrix[0], matrix[1]);
    let inverse_determinant = 1.0 / dot(matrix[0], cofactor0);

    return mat3x3<f32>(
        cofactor0 * inverse_determinant,
        cofactor1 * inverse_determinant,
        cofactor2 * inverse_determinant,
    );
}

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance: u32) -> VertexOutput {
    let view_projection = frame.view_projection;
    let model = objects[instance].model;
    let model_3x3 = mat3x3<f32>(
        model[0].xyz,
        model[1].xyz,
        model[2].xyz,
    );
    let normal_matrix = normal_matrix(model_3x3);

    var out: VertexOutput;
    out.position = view_projection * model * vec4<f32>(in.position, 1.0);
    out.normal = normalize(normal_matrix * in.normal);
    out.uv = in.uv;
    out.tangent = vec4<f32>(
        model_3x3 * in.tangent.xyz,
        in.tangent.w,
    );
    out.light_position = frame.light_view_projection * model * vec4<f32>(in.position, 1.0);
    return out;
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

    if shadow_uv.x < 0.0 || shadow_uv.x > 1.0 ||
        shadow_uv.y < 0.0 || shadow_uv.y > 1.0 ||
        light_ndc.z < 0.0 || light_ndc.z > 1.0 {
        return 1.0;
    }

    return textureSampleCompare(
        shadow_map,
        shadow_sampler,
        shadow_uv,
        light_ndc.z,
    );
}

fn sample_normal_map(uv: vec2<f32>) -> vec3<f32> {
    var normal = textureSample(
        normal_texture,
        material_sampler,
        uv,
    ).xyz * 2.0 - 1.0;

    normal.x *= material.normal_scale.x;
    normal.y *= material.normal_scale.x;

    return normalize(normal);
}

fn normal_from_tangent_space(in: VertexOutput) -> vec3<f32> {
    let surface_normal = normalize(in.normal);
    let tangent = normalize(
        in.tangent.xyz - surface_normal * dot(surface_normal, in.tangent.xyz)
    );
    let bitangent = cross(surface_normal, tangent) * in.tangent.w;
    let tangent_normal = sample_normal_map(in.uv);

    return normalize(
        tangent * tangent_normal.x +
        bitangent * tangent_normal.y +
        surface_normal * tangent_normal.z
    );
}

fn calculate_lighting(
    normal: vec3<f32>,
    light_position: vec4<f32>,
) -> vec3<f32> {
    let light_direction = normalize(frame.light_direction);
    let surface_to_light = -light_direction;

    let diffuse_factor = max(dot(normal, surface_to_light), 0.0);
    let diffuse_intensity = frame.light_intensity * diffuse_factor;
    let shadow = shadow_visibility(light_position);

    let ambient_color = frame.light_color * frame.ambient;
    let diffuse_color = frame.light_color * diffuse_intensity * shadow;

    return ambient_color + diffuse_color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(
        base_color_texture,
        material_sampler,
        in.uv,
    ) * material.base_color_factor;

    let normal = normal_from_tangent_space(in);
    let lighting = calculate_lighting(normal, in.light_position);

    return vec4<f32>(albedo.rgb * lighting, albedo.a);
}
