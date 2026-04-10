use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

const VULKAN_ZIP: &str = "https://github.com/ggml-org/llama.cpp/releases/download/b8740/llama-b8740-bin-win-vulkan-x64.zip";
const CPU_ZIP: &str = "https://github.com/ggml-org/llama.cpp/releases/download/b8740/llama-b8740-bin-win-cpu-x64.zip";
const MODEL_URL: &str = "https://github.com/lanmower/ocr/releases/download/latest/google_gemma-4-E2B-it-Q4_K_M.gguf";
const MMPROJ_URL: &str = "https://github.com/lanmower/ocr/releases/download/latest/mmproj-google_gemma-4-E2B-it-f16.gguf";
const MODEL_FILE: &str = "google_gemma-4-E2B-it-Q4_K_M.gguf";
const MMPROJ_FILE: &str = "mmproj-google_gemma-4-E2B-it-f16.gguf";

pub struct Runtime {
    pub cli: PathBuf,
    pub model: PathBuf,
    pub mmproj: PathBuf,
}

static RT: OnceCell<Runtime> = OnceCell::new();

fn exe_dir() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("resolve exe")?;
    exe.parent().map(Path::to_path_buf).context("exe parent")
}

fn has_gpu() -> bool {
    std::process::Command::new("nvidia-smi")
        .arg("--query-gpu=name")
        .arg("--format=csv,noheader")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || std::process::Command::new("vulkaninfo")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

fn download(url: &str, dest: &Path) -> Result<()> {
    if dest.exists() {
        return Ok(());
    }
    if let Some(p) = dest.parent() {
        std::fs::create_dir_all(p)?;
    }
    let tmp = dest.with_extension("tmp");
    eprintln!("[runtime] downloading {} ...", url);
    let resp = ureq::get(url).call().with_context(|| format!("download {url}"))?;
    let mut body = resp.into_body();
    let mut r = body.as_reader();
    let mut f = File::create(&tmp).context("create tmp")?;
    io::copy(&mut r, &mut f).context("stream download")?;
    std::fs::rename(&tmp, dest).context("finalize download")?;
    Ok(())
}

fn extract_files(zip_path: &Path, dest_dir: &Path, names: &[&str]) -> Result<()> {
    let f = File::open(zip_path).context("open zip")?;
    let mut arc = ZipArchive::new(f).context("parse zip")?;
    for i in 0..arc.len() {
        let mut entry = arc.by_index(i).context("zip entry")?;
        let name = entry.name().to_string();
        let base = Path::new(&name).file_name().map(|n| n.to_string_lossy().to_string());
        if let Some(base) = base {
            if names.contains(&base.as_str()) {
                let out = dest_dir.join(&base);
                if !out.exists() {
                    let mut f = File::create(&out).with_context(|| format!("create {base}"))?;
                    io::copy(&mut entry, &mut f).with_context(|| format!("extract {base}"))?;
                }
            }
        }
    }
    Ok(())
}

pub fn ensure() -> Result<&'static Runtime> {
    RT.get_or_try_init(|| {
        let dir = exe_dir()?.join("llm-runtime");
        std::fs::create_dir_all(&dir).context("create runtime dir")?;

        let zip_url = if has_gpu() { VULKAN_ZIP } else { CPU_ZIP };
        let zip_name = zip_url.split('/').last().unwrap();
        let zip_path = dir.join(zip_name);
        download(zip_url, &zip_path)?;
        extract_files(&zip_path, &dir, &["llama-mtmd-cli.exe", "mtmd.dll"])?;

        let model_path = dir.join(MODEL_FILE);
        download(MODEL_URL, &model_path)?;

        let mmproj_path = dir.join(MMPROJ_FILE);
        download(MMPROJ_URL, &mmproj_path)?;

        Ok(Runtime {
            cli: dir.join("llama-mtmd-cli.exe"),
            model: model_path,
            mmproj: mmproj_path,
        })
    })
}
