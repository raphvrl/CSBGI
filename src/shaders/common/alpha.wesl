import package::common::material::Material;

fn passes_alpha_test(alpha: f32, material: Material) -> bool {
    return material.alpha_parameters.y < 0.5
        || alpha >= material.alpha_parameters.x;
}
