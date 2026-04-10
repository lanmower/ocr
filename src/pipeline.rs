use crate::llm;
use crate::pdf;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone)]
pub enum OutputFormat {
    Csv,
    Text,
    Json,
}

pub struct Job {
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub dpi: u32,
    pub format: OutputFormat,
    pub model: String,
}

fn is_pdf(p: &Path) -> bool {
    p.extension()
        .map(|e| e.to_ascii_lowercase() == "pdf")
        .unwrap_or(false)
}

fn is_image(p: &Path) -> bool {
    let exts = ["png", "jpg", "jpeg", "tiff", "tif", "bmp", "webp"];
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| exts.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn collect_inputs(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|p| is_pdf(p) || is_image(p))
        .collect()
}

fn process_one(job: &Job) -> Result<PathBuf> {
    let stem = job.input.file_stem().unwrap().to_string_lossy();
    let ext = match job.format {
        OutputFormat::Csv => "csv",
        OutputFormat::Text => "txt",
        OutputFormat::Json => "json",
    };
    let out_path = job.output_dir.join(format!("{}.{}", stem, ext));

    let images = if is_pdf(&job.input) {
        pdf::render_pages(&job.input, job.dpi)?
    } else {
        let img = image::open(&job.input)
            .context(format!("open image: {}", job.input.display()))?;
        vec![img]
    };

    let mut file = std::fs::File::create(&out_path)?;
    match job.format {
        OutputFormat::Csv => {
            let csv = llm::process_images_to_csv(&images, &job.model)?;
            std::io::Write::write_all(&mut file, csv.as_bytes())?;
        }
        OutputFormat::Text => {
            llm::write_text(&images, &mut file)?;
        }
        OutputFormat::Json => {
            let result = llm::process_images_to_json(&images, &job.model)?;
            serde_json::to_writer_pretty(&mut file, &result)?;
        }
    }

    Ok(out_path)
}

pub fn run_batch(
    inputs: Vec<PathBuf>,
    output_dir: &Path,
    dpi: u32,
    format: OutputFormat,
    model: &str,
) -> Vec<Result<PathBuf>> {
    let model = model.to_string();
    std::fs::create_dir_all(output_dir).ok();

    inputs
        .iter()
        .map(|input| {
            let job = Job {
                input: input.clone(),
                output_dir: output_dir.to_path_buf(),
                dpi,
                format: format.clone(),
                model: model.clone(),
            };
            let result = process_one(&job);
            match &result {
                Ok(p) => eprintln!("[ok] {} -> {}", input.display(), p.display()),
                Err(e) => eprintln!("[err] {}: {:#}", input.display(), e),
            }
            result
        })
        .collect()
}
