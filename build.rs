use std::io::{self, Read};
use std::fs::{self, File};
use std::path::PathBuf;

const VULKAN_ZIP: &str = "https://github.com/ggml-org/llama.cpp/releases/download/b8741/llama-b8741-bin-win-vulkan-x64.zip";

const EXTRACT: &[&str] = &[
    "llama-mtmd-cli.exe",
    "mtmd.dll",
    "ggml-vulkan.dll",
    "ggml-base.dll",
    "ggml.dll",
    "llama.dll",
    "libomp140.x86_64.dll",
    "ggml-cpu-x64.dll",
];

fn main() {
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let all_present = EXTRACT.iter().all(|f| out.join(f).exists());

    if !all_present {
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
            let fname = name.split('/').last().unwrap_or("");
            if EXTRACT.contains(&fname) {
                let dest = out.join(fname);
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf).unwrap();
                fs::write(&dest, buf).unwrap();
                eprintln!("build: extracted {}", fname);
            }
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
    for f in EXTRACT {
        println!("cargo:rustc-env=LLAMA_{}={}", f.replace('.', "_").replace('-', "_").to_uppercase(), out.join(f).display());
    }
}
