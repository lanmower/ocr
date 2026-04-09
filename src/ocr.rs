use anyhow::{Context, Result};
use image::DynamicImage;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use std::path::Path;
use std::sync::Arc;

pub struct Engine {
    inner: OcrEngine,
}

impl Engine {
    pub fn new(detect_path: &Path, rec_path: &Path) -> Result<Self> {
        let detect = rten::Model::load_file(detect_path)
            .context("load detection model")?;
        let rec = rten::Model::load_file(rec_path)
            .context("load recognition model")?;

        let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(detect),
            recognition_model: Some(rec),
            ..Default::default()
        })?;

        Ok(Self { inner: engine })
    }

    pub fn recognize(&self, img: &DynamicImage) -> Result<Vec<String>> {
        let rgb = img.to_rgb8();
        let source = ImageSource::from_bytes(
            rgb.as_raw(),
            rgb.dimensions(),
        )?;
        let input = self.inner.prepare_input(source)?;
        let words = self.inner.detect_words(&input)?;
        let lines = self.inner.find_text_lines(&input, &words);
        let results = self.inner.recognize_text(&input, &lines)?;

        let text: Vec<String> = results
            .into_iter()
            .filter_map(|line| line.map(|l| l.to_string()))
            .collect();

        Ok(text)
    }
}

pub fn create_engine(detect: &Path, rec: &Path) -> Result<Arc<Engine>> {
    Ok(Arc::new(Engine::new(detect, rec)?))
}
