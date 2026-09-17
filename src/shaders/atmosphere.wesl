const PI: f32 = 3.14159265359;

const PLANET_RADIUS_KM: f32 = 6371.0;
const ATMOSPHERE_RADIUS_KM: f32 = 6471.0;
const RAYLEIGH_SCALE_HEIGHT_KM: f32 = 8.0;

const VIEW_STEPS: u32 = 16u;
const SUN_STEPS: u32 = 4u;

// Rayleigh coefficients in RGB order, expressed per kilometre.
const RAYLEIGH_BETA: vec3<f32> = vec3<f32>(
    0.0058,
    0.0135,
    0.0331,
);

fn ray_sphere_intersection(
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

    return vec2<f32>(
        -b - root,
        -b + root,
    );
}

fn atmosphere_density(
    position: vec3<f32>,
    planet_center: vec3<f32>,
) -> f32 {
    let altitude = length(position - planet_center) - PLANET_RADIUS_KM;

    if altitude < 0.0 || altitude > ATMOSPHERE_RADIUS_KM - PLANET_RADIUS_KM {
        return 0.0;
    }

    return exp(-altitude / RAYLEIGH_SCALE_HEIGHT_KM);
}

fn rayleigh_phase(cos_angle: f32) -> f32 {
    return 3.0 / (16.0 * PI) * (1.0 + cos_angle * cos_angle);
}

fn sun_optical_depth(
    position: vec3<f32>,
    sun_direction: vec3<f32>,
    planet_center: vec3<f32>,
) -> f32 {
    let planet_hit = ray_sphere_intersection(
        position,
        sun_direction,
        planet_center,
        PLANET_RADIUS_KM,
    );

    if planet_hit.x > 0.0 {
        return 1000.0;
    }

    let atmosphere_hit = ray_sphere_intersection(
        position,
        sun_direction,
        planet_center,
        ATMOSPHERE_RADIUS_KM,
    );

    let start = max(atmosphere_hit.x, 0.0);
    let end = atmosphere_hit.y;
    let step_size = (end - start) / f32(SUN_STEPS);

    var optical_depth = 0.0;

    for (var index = 0u; index < SUN_STEPS; index++) {
        let distance = start + (f32(index) + 0.5) * step_size;
        let sample_position = position + sun_direction * distance;

        optical_depth += atmosphere_density(
            sample_position,
            planet_center,
        ) * step_size;
    }

    return optical_depth;
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
    let optical_depth = sun_optical_depth(
        camera_position,
        sun_direction,
        planet_center,
    );

    return exp(-RAYLEIGH_BETA * optical_depth);
}

fn calculate_sky(
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

    let atmosphere_hit = ray_sphere_intersection(
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

    let planet_hit = ray_sphere_intersection(
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
    let phase = rayleigh_phase(
        dot(ray_direction, sun_direction)
    );
    let scattering_step = RAYLEIGH_BETA
        * phase
        * light_color
        * light_intensity
        * step_size;

    var view_optical_depth = 0.0;
    var sky_color = vec3<f32>(0.0);

    for (var index = 0u; index < VIEW_STEPS; index++) {
        let distance = start + (f32(index) + 0.5) * step_size;
        let sample_position = camera_position + ray_direction * distance;

        let density = atmosphere_density(
            sample_position,
            planet_center,
        );

        let sun_depth = sun_optical_depth(
            sample_position,
            sun_direction,
            planet_center,
        );

        let attenuation = exp(
            -RAYLEIGH_BETA * (view_optical_depth + sun_depth)
        );

        sky_color += attenuation
            * scattering_step
            * density;

        view_optical_depth += density * step_size;
    }

    return sky_color;
}
