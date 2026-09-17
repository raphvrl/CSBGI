import package::common::frame::{Camera, Light, lux_to_render};

struct MaterialData {
    albedo: vec3<f32>,
    orm: vec3<f32>,
    factors: vec4<f32>,
};

const PI: f32 = 3.14159265359;

fn distribution(
    n_dot_h: f32,
    roughness: f32,
) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denominator = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;

    return a2 / max(PI * denominator * denominator, 0.0001);
}

fn geometry(
    n_dot_x: f32,
    roughness: f32,
) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;

    return n_dot_x / max(n_dot_x * (1.0 - k) + k, 0.0001);
}

fn smith(
    n_dot_v: f32,
    n_dot_l: f32,
    roughness: f32,
) -> f32 {
    return geometry(n_dot_v, roughness) * geometry(n_dot_l, roughness);
}

fn fresnel(
    cos_theta: f32,
    f0: vec3<f32>,
) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - cos_theta, 5.0);
}

fn roughness(
    normal: vec3<f32>,
    base_roughness: f32,
) -> f32 {
    let dndx = dpdx(normal);
    let dndy = dpdy(normal);
    let variance = 0.15 * (dot(dndx, dndx) + dot(dndy, dndy));
    let kernel_roughness2 = min(2.0 * variance, 0.18);

    return min(sqrt(base_roughness * base_roughness + kernel_roughness2), 1.0);
}

fn calculate(
    normal: vec3<f32>,
    world_position: vec3<f32>,
    material: MaterialData,
    camera: Camera,
    light: Light,
    shadow: f32,
) -> vec3<f32> {
    let occlusion = clamp(material.factors.z, 0.0, 1.0);
    let base_roughness = clamp(material.orm.g * material.factors.y, 0.04, 1.0);
    let n = normalize(normal);
    let surface_roughness = roughness(n, base_roughness);
    let metallic = clamp(material.orm.b * material.factors.x, 0.0, 1.0);

    let v = normalize(camera.camera_position - world_position);
    let l = normalize(-light.light_direction);

    let n_dot_v = max(dot(n, v), 0.0);
    let n_dot_l = max(dot(n, l), 0.0);

    let f0 = vec3<f32>(0.04) * (1.0 - metallic) + material.albedo * metallic;
    var direct_light = vec3<f32>(0.0);

    if n_dot_v > 0.0 && n_dot_l > 0.0 {
        let h = normalize(v + l);
        let n_dot_h = max(dot(n, h), 0.0);
        let h_dot_v = max(dot(h, v), 0.0);
        let f = fresnel(h_dot_v, f0);
        let d = distribution(n_dot_h, surface_roughness);
        let g = smith(n_dot_v, n_dot_l, surface_roughness);
        let specular = (d * g * f) / max(4.0 * n_dot_v * n_dot_l, 0.0001);
        let diffuse = (vec3<f32>(1.0) - f) * (1.0 - metallic)
            * material.albedo / PI;

        direct_light = (diffuse + specular)
            * light.light_color
            * lux_to_render(light.light_illuminance_lux)
            * n_dot_l
            * shadow;
    }

    let ambient_diffuse = material.albedo * (1.0 - metallic) / PI;
    let ambient_light = ambient_diffuse
        * light.light_color
        * lux_to_render(light.ambient_illuminance_lux)
        * occlusion;

    return direct_light + ambient_light;
}
