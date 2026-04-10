use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::path::PathBuf;

const INFER_SRC: &str = include_str!(env!("INFER_PY"));

pub struct Runtime {
    pub py: PathBuf,
    pub script: PathBuf,
}

static RT: OnceCell<Runtime> = OnceCell::new();

fn exe_dir() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("resolve exe")?;
    exe.parent().map(std::path::Path::to_path_buf).context("exe parent")
}

pub fn ensure() -> Result<&'static Runtime> {
    RT.get_or_try_init(|| {
        let dir = exe_dir()?.join("llm-runtime");
        std::fs::create_dir_all(&dir).context("create runtime dir")?;

        let script = dir.join("infer.py");
        if !script.exists() {
            std::fs::write(&script, INFER_SRC).context("write infer.py")?;
        }

        let py = which_python().context("python not found — install Python 3 and ensure it is on PATH")?;

        let check = std::process::Command::new(&py)
            .args(["-c", "import google.genai"])
            .output()
            .context("check google-genai")?;
        if !check.status.success() {
            anyhow::bail!("google-genai not installed — run: pip install google-genai");
        }

        Ok(Runtime { py, script })
    })
}

fn which_python() -> Option<PathBuf> {
    for candidate in ["python", "python3", "python.exe", "python3.exe"] {
        if let Ok(out) = std::process::Command::new(candidate).arg("--version").output() {
            if out.status.success() {
                return Some(PathBuf::from(candidate));
            }
        }
    }
    None
}
