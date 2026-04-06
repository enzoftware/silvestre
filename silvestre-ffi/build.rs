fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out_file = std::path::Path::new(&crate_dir).join("include/silvestre.h");

    if let Some(parent) = out_file.parent() {
        std::fs::create_dir_all(parent).expect("failed to create include directory");
    }

    let config =
        cbindgen::Config::from_file("cbindgen.toml").expect("failed to read cbindgen.toml");

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("failed to generate C bindings")
        .write_to_file(out_file);
}
