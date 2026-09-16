fn build_shader(module: &str, artifact: &str) {
    wesl::Wesl::new("src/shaders")
        .build_artifact(&module.parse().expect("invalid WESL module path"), artifact);
}

fn main() {
    build_shader("package::geometry", "geometry");
    build_shader("package::mipmap", "mipmap");
    build_shader("package::shadow", "shadow");
    build_shader("package::tonemapping", "tonemapping");
}
