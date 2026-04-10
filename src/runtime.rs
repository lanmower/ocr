use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

const GH_BASE: &str = "https://github.com/lanmower/ocr/releases/download/latest";
const MODEL_SHARD1: &str = "model-00001-of-00003.gguf";
const MODEL_SHARD2: &str = "model-00002-of-00003.gguf";
const MODEL_SHARD3: &str = "model-00003-of-00003.gguf";
const MMPROJ_FILE: &str = "mmproj-google_gemma-4-E2B-it-f16.gguf";

const CLI_BYTES: &[u8] = include_bytes!(env!("LLAMA_LLAMA_MTMD_CLI_EXE"));
const MTMD_DLL: &[u8] = include_bytes!(env!("LLAMA_MTMD_DLL"));
const GGML_VULKAN: &[u8] = include_bytes!(env!("LLAMA_GGML_VULKAN_DLL"));
const GGML_BASE: &[u8] = include_bytes!(env!("LLAMA_GGML_BASE_DLL"));
const GGML_DLL: &[u8] = include_bytes!(env!("LLAMA_GGML_DLL"));
const LLAMA_DLL: &[u8] = include_bytes!(env!("LLAMA_LLAMA_DLL"));
const LIBOMP: &[u8] = include_bytes!(env!("LLAMA_LIBOMP140_X86_64_DLL"));
const GGML_CPU: &[u8] = include_bytes!(env!("LLAMA_GGML_CPU_X64_DLL"));

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

fn extract(bytes: &[u8], dest: &Path) -> Result<()> {
    if dest.exists() {
        return Ok(());
    }
    if let Some(p) = dest.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(dest, bytes).with_context(|| format!("extract {}", dest.display()))
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

pub fn ensure() -> Result<&'static Runtime> {
    RT.get_or_try_init(|| {
        let dir = exe_dir()?.join("llm-runtime");
        std::fs::create_dir_all(&dir).context("create runtime dir")?;

        extract(CLI_BYTES, &dir.join("llama-mtmd-cli.exe"))?;
        extract(MTMD_DLL, &dir.join("mtmd.dll"))?;
        extract(GGML_VULKAN, &dir.join("ggml-vulkan.dll"))?;
        extract(GGML_BASE, &dir.join("ggml-base.dll"))?;
        extract(GGML_DLL, &dir.join("ggml.dll"))?;
        extract(LLAMA_DLL, &dir.join("llama.dll"))?;
        extract(LIBOMP, &dir.join("libomp140.x86_64.dll"))?;
        extract(GGML_CPU, &dir.join("ggml-cpu-x64.dll"))?;

        for file in [MODEL_SHARD1, MODEL_SHARD2, MODEL_SHARD3, MMPROJ_FILE] {
            download(&format!("{GH_BASE}/{file}"), &dir.join(file))?;
        }

        Ok(Runtime {
            cli: dir.join("llama-mtmd-cli.exe"),
            model: dir.join(MODEL_SHARD1),
            mmproj: dir.join(MMPROJ_FILE),
        })
    })
}
