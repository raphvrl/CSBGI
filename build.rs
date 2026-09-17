fn build_shader(module: &str, artifact: &str) {
    wesl::Wesl::new("src/shaders")
        .build_artifact(&module.parse().expect("invalid WESL module path"), artifact);
}

fn main() {
    build_shader("package::passes::geometry", "geometry");
    build_shader("package::passes::mipmap", "mipmap");
    build_shader("package::passes::shadow", "shadow");
    build_shader("package::passes::sky", "sky");
    build_shader("package::passes::tonemap", "tonemapping");
    build_shader("package::passes::lut", "lut");
}
