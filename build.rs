use std::io::{self, Read};
use std::fs::{self, File};
use std::path::PathBuf;

const VULKAN_ZIP: &str = "https://github.com/ggml-org/llama.cpp/releases/download/b8740/llama-b8740-bin-win-vulkan-x64.zip";

fn main() {
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let cli = out.join("llama-mtmd-cli.exe");
    let dll = out.join("mtmd.dll");

    if !cli.exists() || !dll.exists() {
        let zip_path = out.join("llama.zip");
        if !zip_path.exists() {
            eprintln!("build: downloading llama zip...");
            let resp = ureq::get(VULKAN_ZIP).call().unwrap();
            let mut body = resp.into_body();
            let mut r = body.as_reader();
            let mut f = File::create(&zip_path).unwrap();
            io::copy(&mut r, &mut f).unwrap();
        }
        eprintln!("build: extracting llama binaries...");
        let f = File::open(&zip_path).unwrap();
        let mut zip = zip::ZipArchive::new(f).unwrap();
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).unwrap();
            let name = entry.name().to_string();
            let dest = if name.ends_with("llama-mtmd-cli.exe") {
                Some(cli.clone())
            } else if name.ends_with("mtmd.dll") {
                Some(dll.clone())
            } else {
                None
            };
            if let Some(dest) = dest {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf).unwrap();
                fs::write(&dest, buf).unwrap();
            }
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=LLAMA_CLI_PATH={}", cli.display());
    println!("cargo:rustc-env=MTMD_DLL_PATH={}", dll.display());
}
