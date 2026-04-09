use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const DETECT_URL: &str = "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten";
const REC_URL: &str = "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten";

fn model_dir() -> PathBuf {
    let dir = dirs();
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn dirs() -> PathBuf {
    let base = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("."))
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();
    base.join("models")
}

fn fetch(url: &str, dest: &Path) -> Result<()> {
    if dest.exists() {
        return Ok(());
    }
    eprintln!("Downloading {} ...", url);
    let body = ureq::get(url).call().context("download failed")?;
    let bytes = body.into_body().read_to_vec().context("read body failed")?;
    std::fs::write(dest, &bytes).context("write model failed")?;
    eprintln!("Saved to {}", dest.display());
    Ok(())
}

pub fn ensure_models(custom_dir: Option<&Path>) -> Result<(PathBuf, PathBuf)> {
    let dir = custom_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(model_dir);
    std::fs::create_dir_all(&dir)?;

    let detect = dir.join("text-detection.rten");
    let rec = dir.join("text-recognition.rten");

    fetch(DETECT_URL, &detect)?;
    fetch(REC_URL, &rec)?;

    Ok((detect, rec))
}
