use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use image::DynamicImage;
use once_cell::sync::OnceCell;
use pdfium_render::prelude::*;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use tar::Archive;

const PDFIUM_URL: &str =
    "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7543/pdfium-win-x64.tgz";
static PDFIUM: OnceCell<Pdfium> = OnceCell::new();

fn exe_dir() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("resolve current executable")?;
    exe.parent()
        .map(Path::to_path_buf)
        .context("resolve executable directory")
}

fn pdfium_dll_path() -> Result<PathBuf> {
    Ok(exe_dir()?.join("pdfium.dll"))
}

fn ensure_pdfium_dll() -> Result<PathBuf> {
    let dll_path = pdfium_dll_path()?;
    if dll_path.exists() {
        return Ok(dll_path);
    }

    let temp_path = exe_dir()?.join("pdfium-win-x64.tgz");
    eprintln!("Downloading Pdfium runtime...");

    let response = ureq::get(PDFIUM_URL)
        .call()
        .context("download pdfium archive failed")?;
    let bytes = response
        .into_body()
        .read_to_vec()
        .context("read pdfium archive failed")?;
    std::fs::write(&temp_path, &bytes).context("write pdfium archive failed")?;

    let file = File::open(&temp_path).context("open pdfium archive failed")?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let mut extracted = false;

    for entry in archive.entries().context("read pdfium archive entries failed")? {
        let mut entry = entry.context("read pdfium archive entry failed")?;
        let path = entry.path().context("read pdfium archive path failed")?;
        if path.as_ref() == Path::new("bin/pdfium.dll") {
            let mut out = File::create(&dll_path).context("create pdfium.dll failed")?;
            io::copy(&mut entry, &mut out).context("extract pdfium.dll failed")?;
            extracted = true;
            break;
        }
    }

    let _ = std::fs::remove_file(&temp_path);

    if !extracted {
        anyhow::bail!("pdfium.dll not found in downloaded archive");
    }

    Ok(dll_path)
}

fn bind_pdfium() -> Result<Pdfium> {
    PDFIUM
        .get_or_try_init(|| {
            let dll_path = ensure_pdfium_dll()?;
            let bindings = Pdfium::bind_to_library(&dll_path)
                .or_else(|_| Pdfium::bind_to_system_library())
                .context("bind to pdfium library failed")?;
            Ok(Pdfium::new(bindings))
        })
        .cloned()
}

pub fn render_pages(path: &Path, dpi: u32) -> Result<Vec<DynamicImage>> {
    let pdfium = bind_pdfium()?;
    let doc = pdfium
        .load_pdf_from_file(path, None)
        .context(format!("failed to open PDF: {}", path.display()))?;

    let scale = dpi as f32 / 72.0;
    let mut images = Vec::new();

    for (i, page) in doc.pages().iter().enumerate() {
        let w = (page.width().value * scale) as i32;
        let h = (page.height().value * scale) as i32;

        let config = PdfRenderConfig::new()
            .set_target_width(w)
            .set_maximum_height(h);

        let bmp = page
            .render_with_config(&config)
            .context(format!("render page {} failed", i))?;
        let img = bmp.as_image();

        images.push(img?);
    }

    Ok(images)
}
