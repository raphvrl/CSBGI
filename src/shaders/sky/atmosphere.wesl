const PI: f32 = 3.14159265359;

const PLANET_RADIUS_KM: f32 = 6371.0;
const ATMOSPHERE_RADIUS_KM: f32 = 6471.0;
const RAYLEIGH_SCALE_HEIGHT_KM: f32 = 8.0;
const MIE_SCALE_HEIGHT_KM: f32 = 1.2;
const OZONE_CENTER_KM: f32 = 25.0;
const OZONE_HALF_WIDTH_KM: f32 = 15.0;

const VIEW_STEPS: u32 = 16u;
const SUN_STEPS: u32 = 4u;

const RAYLEIGH_BETA: vec3<f32> = vec3<f32>(
    0.0058,
    0.0135,
    0.0331,
);
const MIE_SCATTERING_BETA: vec3<f32> = vec3<f32>(
    0.015,
    0.015,
    0.015,
);
const MIE_EXTINCTION_BETA: vec3<f32> = vec3<f32>(
    0.015,
    0.015,
    0.015,
);
const OZONE_ABSORPTION_BETA: vec3<f32> = vec3<f32>(
    0.00065,
    0.00188,
    0.000085,
);
const MIE_G: f32 = 0.76;

fn intersection(
    origin: vec3<f32>,
    direction: vec3<f32>,
    center: vec3<f32>,
    radius: f32,
) -> vec2<f32> {
    let offset = origin - center;
    let b = dot(offset, direction);
    let c = dot(offset, offset) - radius * radius;
    let discriminant = b * b - c;

    if discriminant < 0.0 {
        return vec2<f32>(-1.0);
    }

    let root = sqrt(discriminant);

    return vec2<f32>(-b - root, -b + root);
}

fn density(
    position: vec3<f32>,
    planet_center: vec3<f32>,
    scale_height_km: f32,
) -> f32 {
    let altitude = length(position - planet_center) - PLANET_RADIUS_KM;

    if altitude < 0.0 || altitude > ATMOSPHERE_RADIUS_KM - PLANET_RADIUS_KM {
        return 0.0;
    }

    return exp(-altitude / scale_height_km);
}

fn ozone_density(
    position: vec3<f32>,
    planet_center: vec3<f32>,
) -> f32 {
    let altitude = length(position - planet_center) - PLANET_RADIUS_KM;
    let distance = abs(altitude - OZONE_CENTER_KM);

    return max(1.0 - distance / OZONE_HALF_WIDTH_KM, 0.0);
}

fn phase(cos_angle: f32) -> f32 {
    return 3.0 / (16.0 * PI) * (1.0 + cos_angle * cos_angle);
}

fn mie_phase(cos_angle: f32) -> f32 {
    let g_squared = MIE_G * MIE_G;
    let denominator = pow(
        1.0 + g_squared - 2.0 * MIE_G * cos_angle,
        1.5,
    );

    return (1.0 - g_squared) / (4.0 * PI * denominator);
}

fn optical_depth(
    position: vec3<f32>,
    sun_direction: vec3<f32>,
    planet_center: vec3<f32>,
) -> vec3<f32> {
    let planet_hit = intersection(
        position,
        sun_direction,
        planet_center,
        PLANET_RADIUS_KM,
    );

    if planet_hit.x > 0.0 {
        return vec3<f32>(1000.0);
    }

    let atmosphere_hit = intersection(
        position,
        sun_direction,
        planet_center,
        ATMOSPHERE_RADIUS_KM,
    );

    let start = max(atmosphere_hit.x, 0.0);
    let end = atmosphere_hit.y;
    let step_size = (end - start) / f32(SUN_STEPS);

    var result = vec3<f32>(0.0);

    for (var index = 0u; index < SUN_STEPS; index++) {
        let distance = start + (f32(index) + 0.5) * step_size;
        let sample_position = position + sun_direction * distance;
        let rayleigh_density = density(
            sample_position,
            planet_center,
            RAYLEIGH_SCALE_HEIGHT_KM,
        );
        let mie_density = density(
            sample_position,
            planet_center,
            MIE_SCALE_HEIGHT_KM,
        );
        let sample_ozone_density = ozone_density(sample_position, planet_center);

        result += vec3<f32>(
            rayleigh_density,
            mie_density,
            sample_ozone_density,
        ) * step_size;
    }

    return result;
}

fn sun_transmittance(
    camera_position_world: vec3<f32>,
    light_direction_input: vec3<f32>,
) -> vec3<f32> {
    let camera_position = camera_position_world * 0.001;
    let planet_center = vec3<f32>(
        camera_position.x,
        -PLANET_RADIUS_KM,
        camera_position.z,
    );
    let sun_direction = normalize(-light_direction_input);
    let depths = optical_depth(camera_position, sun_direction, planet_center);

    return exp(-(
        RAYLEIGH_BETA * depths.x
        + MIE_EXTINCTION_BETA * depths.y
        + OZONE_ABSORPTION_BETA * depths.z
    ));
}

fn calculate(
    ray_direction: vec3<f32>,
    camera_position_world: vec3<f32>,
    light_direction_input: vec3<f32>,
    light_color: vec3<f32>,
    light_intensity: f32,
) -> vec3<f32> {
    let camera_position = camera_position_world * 0.001;
    let planet_center = vec3<f32>(
        camera_position.x,
        -PLANET_RADIUS_KM,
        camera_position.z,
    );
    let sun_direction = normalize(-light_direction_input);

    let atmosphere_hit = intersection(
        camera_position,
        ray_direction,
        planet_center,
        ATMOSPHERE_RADIUS_KM,
    );

    if atmosphere_hit.y <= 0.0 {
        return vec3<f32>(0.0);
    }

    let start = max(atmosphere_hit.x, 0.0);
    var end = atmosphere_hit.y;

    let planet_hit = intersection(
        camera_position,
        ray_direction,
        planet_center,
        PLANET_RADIUS_KM,
    );

    if planet_hit.x >= 0.0 {
        end = min(end, planet_hit.x);
    } else if planet_hit.y > 0.0 {
        end = min(end, planet_hit.y);
    }

    if end <= start {
        return vec3<f32>(0.0);
    }

    let step_size = (end - start) / f32(VIEW_STEPS);
    let cos_angle = dot(ray_direction, sun_direction);
    let rayleigh_phase = phase(cos_angle);
    let mie_phase_value = mie_phase(cos_angle);

    var view_rayleigh_depth = 0.0;
    var view_mie_depth = 0.0;
    var view_ozone_depth = 0.0;
    var sky_color = vec3<f32>(0.0);

    for (var index = 0u; index < VIEW_STEPS; index++) {
        let distance = start + (f32(index) + 0.5) * step_size;
        let sample_position = camera_position + ray_direction * distance;
        let rayleigh_density = density(
            sample_position,
            planet_center,
            RAYLEIGH_SCALE_HEIGHT_KM,
        );
        let mie_density = density(
            sample_position,
            planet_center,
            MIE_SCALE_HEIGHT_KM,
        );
        let sample_ozone_density = ozone_density(sample_position, planet_center);
        let sun_depth = optical_depth(sample_position, sun_direction, planet_center);
        let attenuation = exp(-(
            RAYLEIGH_BETA * (view_rayleigh_depth + sun_depth.x)
            + MIE_EXTINCTION_BETA * (view_mie_depth + sun_depth.y)
            + OZONE_ABSORPTION_BETA * (view_ozone_depth + sun_depth.z)
        ));
        let scattering = (
            RAYLEIGH_BETA * rayleigh_phase * rayleigh_density
            + MIE_SCATTERING_BETA * mie_phase_value * mie_density
        )
        * light_color
        * light_intensity
        * step_size;

        sky_color += attenuation * scattering;
        view_rayleigh_depth += rayleigh_density * step_size;
        view_mie_depth += mie_density * step_size;
        view_ozone_depth += sample_ozone_density * step_size;
    }

    return sky_color;
}
