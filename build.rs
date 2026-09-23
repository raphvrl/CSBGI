fn build_shader(module: &str, artifact: &str) {
    wesl::Wesl::new("src/shaders")
        .build_artifact(&module.parse().expect("invalid WESL module path"), artifact);
}

fn main() {
    build_shader("package::passes::geometry", "geometry");
    build_shader("package::passes::debug::meshlet", "meshlet_debug");
    build_shader("package::passes::depth", "depth");
    build_shader("package::passes::mipmap", "mipmap");
    build_shader("package::passes::shadow", "shadow");
    build_shader("package::passes::sky", "sky");
    build_shader("package::passes::tonemap", "tonemapping");
    build_shader("package::passes::lut", "lut");
    build_shader("package::passes::luminance", "luminance");
    build_shader("package::passes::reduce", "reduce");
    build_shader("package::passes::exposure", "exposure");
    build_shader("package::passes::debug::voxel", "debug_voxel");
    build_shader("package::passes::voxel::lighting", "voxel_lighting");
    build_shader("package::passes::voxel::base", "voxel_mipmap_base");
    build_shader("package::passes::voxel::mip", "voxel_mipmap");
}
