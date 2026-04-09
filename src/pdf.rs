use anyhow::{Context, Result};
use image::DynamicImage;
use pdfium_render::prelude::*;
use std::path::Path;

pub fn render_pages(path: &Path, dpi: u32) -> Result<Vec<DynamicImage>> {
    let pdfium = Pdfium::default();
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
