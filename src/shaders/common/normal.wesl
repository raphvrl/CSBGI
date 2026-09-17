fn matrix(matrix: mat3x3<f32>) -> mat3x3<f32> {
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

fn tangent(
    surface_normal: vec3<f32>,
    tangent_input: vec4<f32>,
    tangent_normal: vec3<f32>,
) -> vec3<f32> {
    let normal = normalize(surface_normal);
    let tangent = normalize(
        tangent_input.xyz - normal * dot(normal, tangent_input.xyz)
    );
    let bitangent = cross(normal, tangent) * tangent_input.w;

    return normalize(
        tangent * tangent_normal.x +
        bitangent * tangent_normal.y +
        normal * tangent_normal.z
    );
}
