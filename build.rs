use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let infer = root.join("src").join("infer.py");
    println!("cargo:rustc-env=INFER_PY={}", infer.display());
    println!("cargo:rerun-if-changed=src/infer.py");
    println!("cargo:rerun-if-changed=build.rs");
}
