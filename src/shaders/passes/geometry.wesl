import package::common::frame::Frame;
import package::common::material::Material;
import package::common::normal::{matrix, tangent};
import package::common::object::ObjectUniform;
import package::common::shadow::visibility;
import package::common::vertex::{VertexInput, VertexOutput};
import package::common::voxel::cone::trace_diffuse;
import package::surface::pbr::{calculate, MaterialData};

@group(0) @binding(0)
var<uniform> frame: Frame;

@group(0) @binding(1)
var shadow_map: texture_depth_2d;

@group(0) @binding(2)
var shadow_sampler: sampler_comparison;

@group(1) @binding(0)
var<storage, read> objects: array<ObjectUniform>;

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

const ALPHA_CUTOFF: f32 = 0.1;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance: u32) -> VertexOutput {
    let model = objects[instance].model;
    let model_3x3 = mat3x3<f32>(model[0].xyz, model[1].xyz, model[2].xyz);
    let world_position = (model * vec4<f32>(in.position, 1.0)).xyz;

    var output: VertexOutput;
    output.position = frame.camera.view_projection * model * vec4<f32>(in.position, 1.0);
    output.normal = normalize(matrix(model_3x3) * in.normal);
    output.uv = in.uv;
    output.tangent = vec4<f32>(model_3x3 * in.tangent.xyz, in.tangent.w);
    output.light_position = frame.light.view_projection * model * vec4<f32>(in.position, 1.0);
    output.world_position = world_position;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(
        base_color_texture,
        material_sampler,
        input.uv,
    ) * material.base_color_factor;

    if base_color.a < ALPHA_CUTOFF {
        discard;
    }

    let orm = textureSample(
        orm_texture,
        material_sampler,
        input.uv,
    ).rgb;

    let encoded_normal = textureSample(
        normal_texture,
        material_sampler,
        input.uv,
    ).xyz * 2.0 - 1.0;

    let normal = tangent(
        input.normal,
        input.tangent,
        normalize(vec3<f32>(
            encoded_normal.x * material.normal_scale.x,
            encoded_normal.y * material.normal_scale.x,
            encoded_normal.z,
        )),
    );

    let shadow = visibility(
        shadow_map,
        shadow_sampler,
        input.light_position,
    );

    let indirect_radiance = trace_diffuse(
        input.world_position,
        normal,
    );

    let color = calculate(
        normal,
        input.world_position,
        MaterialData(
            base_color.rgb,
            orm,
            material.pbr_factors,
        ),
        frame.camera,
        frame.light,
        shadow,
        indirect_radiance,
    );

    return vec4<f32>(color, base_color.a);
}
