use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

const BASE: &str = "https://github.com/lanmower/ocr/releases/download/latest";
const CLI_FILE: &str = "llama-mtmd-cli.exe";
const DLL_FILE: &str = "mtmd.dll";
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

        for file in [CLI_FILE, DLL_FILE, MODEL_FILE, MMPROJ_FILE] {
            download(&format!("{BASE}/{file}"), &dir.join(file))?;
        }

        Ok(Runtime {
            cli: dir.join(CLI_FILE),
            model: dir.join(MODEL_FILE),
            mmproj: dir.join(MMPROJ_FILE),
        })
    })
}
