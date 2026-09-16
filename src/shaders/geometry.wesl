struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,

    @location(0)
    @interpolate(perspective, sample)
    normal: vec3<f32>,

    @location(1)
    @interpolate(perspective, sample)
    uv: vec2<f32>,

    @location(2)
    @interpolate(perspective, sample)
    light_position: vec4<f32>,

    @location(3)
    @interpolate(perspective, sample)
    tangent: vec4<f32>,

    @location(4)
    @interpolate(perspective, sample)
    world_position: vec3<f32>,
}

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
    pbr_factors: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: Material;

@group(2) @binding(1)
var base_color_texture: texture_2d<f32>;

@group(2) @binding(2)
var normal_texture: texture_2d<f32>;

@group(2) @binding(3)
var orm_texture: texture_2d<f32>;

@group(2) @binding(4)
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

    let world_position = (model * vec4<f32>(in.position, 1.0)).xyz;

    var out: VertexOutput;
    out.position = view_projection * model * vec4<f32>(in.position, 1.0);
    out.normal = normalize(normal_matrix * in.normal);
    out.uv = in.uv;
    out.tangent = vec4<f32>(
        model_3x3 * in.tangent.xyz,
        in.tangent.w,
    );
    out.light_position = frame.light_view_projection * model * vec4<f32>(in.position, 1.0);
    out.world_position = world_position.xyz;
    return out;
}

const SHADOW_MAP_SIZE: f32 = 4096.0;
const SHADOW_TEXEL_SIZE: f32 = 1.0 / SHADOW_MAP_SIZE;

fn shadow_sample(
    uv: vec2<f32>,
    depth: f32,
    offset: vec2<f32>,
) -> f32 {
    return textureSampleCompare(
        shadow_map,
        shadow_sampler,
        uv + offset * SHADOW_TEXEL_SIZE,
        depth,
    );
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

    let depth = light_ndc.z;

    var visibility = 0.0;

    visibility += shadow_sample(shadow_uv, depth, vec2<f32>(-1.0, -1.0));
    visibility += shadow_sample(shadow_uv, depth, vec2<f32>(0.0, -1.0));
    visibility += shadow_sample(shadow_uv, depth, vec2<f32>(1.0, -1.0));

    visibility += shadow_sample(shadow_uv, depth, vec2<f32>(-1.0, 0.0));
    visibility += shadow_sample(shadow_uv, depth, vec2<f32>(0.0, 0.0));
    visibility += shadow_sample(shadow_uv, depth, vec2<f32>(1.0, 0.0));

    visibility += shadow_sample(shadow_uv, depth, vec2<f32>(-1.0, 1.0));
    visibility += shadow_sample(shadow_uv, depth, vec2<f32>(0.0, 1.0));
    visibility += shadow_sample(shadow_uv, depth, vec2<f32>(1.0, 1.0));

    return visibility / 9.0;
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

const PI: f32 = 3.14159265359;

fn distribution_ggx(
    n_dot_h: f32,
    roughness: f32,
) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;

    let denominator = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;

    return a2 / max(PI * denominator * denominator, 0.0001);
}

fn geometry_schlick_ggx(
    n_dot_x: f32,
    roughness: f32,
) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;

    return n_dot_x / max(n_dot_x * (1.0 - k) + k, 0.0001);
}

fn geometry_smith(
    n_dot_v: f32,
    n_dot_l: f32,
    roughness: f32,
) -> f32 {
    let ggx_v = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx_l = geometry_schlick_ggx(n_dot_l, roughness);

    return ggx_v * ggx_l;
}

fn fresnel_schlick(
    cos_theta: f32,
    f0: vec3<f32>,
) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - cos_theta, 5.0);
}

fn specular_aa_roughness(
    normal: vec3<f32>,
    roughness: f32,
) -> f32 {
    let dndx = dpdx(normal);
    let dndy = dpdy(normal);

    let variance = 0.15 * (dot(dndx, dndx) + dot(dndy, dndy));

    let kernel_roughness2 = min(
        2.0 * variance,
        0.18
    );

    return min(
        sqrt(roughness * roughness + kernel_roughness2),
        1.0
    );
}

fn calculate_pbr(
    normal: vec3<f32>,
    world_position: vec3<f32>,
    albedo: vec3<f32>,
    orm: vec3<f32>,
    light_position: vec4<f32>,
) -> vec3<f32> {
    let occlusion = clamp(material.pbr_factors.z, 0.0, 1.0);

    let base_roughness = clamp(
        orm.g * material.pbr_factors.y,
        0.04,
        1.0,
    );

    let n = normalize(normal);

    let roughness = specular_aa_roughness(
        n,
        base_roughness,
    );

    let metallic = clamp(
        orm.b * material.pbr_factors.x,
        0.0,
        1.0,
    );

    let v = normalize(frame.camera_position - world_position);

    let l = normalize(-frame.light_direction);
    let h = normalize(v + l);

    let n_dot_v = max(dot(n, v), 0.0);
    let n_dot_l = max(dot(n, l), 0.0);
    let n_dot_h = max(dot(n, h), 0.0);
    let h_dot_v = max(dot(h, v), 0.0);

    let f0 = vec3<f32>(0.04) * (1.0 - metallic) + albedo * metallic;

    let f = fresnel_schlick(h_dot_v, f0);
    let d = distribution_ggx(n_dot_h, roughness);
    let g = geometry_smith(n_dot_v, n_dot_l, roughness);

    let specular = (d * g * f) / max(4.0 * n_dot_v * n_dot_l, 0.0001);

    let diffuse_weight = (vec3<f32>(1.0) - f) * (1.0 - metallic);

    let diffuse = diffuse_weight * albedo / PI;

    let shadow = shadow_visibility(light_position);

    let direct_light = (diffuse + specular)
                * frame.light_color
                * frame.light_intensity
                * n_dot_l
                * shadow;

    let ambient_light = diffuse
                * frame.light_color
                * frame.ambient
                * occlusion;

    return direct_light + ambient_light;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(
        base_color_texture,
        material_sampler,
        in.uv,
    ) * material.base_color_factor;

    let orm = textureSample(
        orm_texture,
        material_sampler,
        in.uv,
    ).rgb;

    let normal = normal_from_tangent_space(in);

    let color = calculate_pbr(
        normal,
        in.world_position,
        base_color.rgb,
        orm,
        in.light_position,
    );

    return vec4<f32>(color, base_color.a);
}
