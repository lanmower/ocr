mod drive;
mod llm;
mod models;
mod ocr;
mod parse;
mod pdf;
mod pipeline;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ocr", about = "Batch OCR pipeline for bank statements")]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long, default_value = "./output")]
    output: PathBuf,

    #[arg(long, default_value = "300")]
    dpi: u32,

    #[arg(long, default_value = "text", value_parser = parse_format)]
    format: pipeline::OutputFormat,

    #[arg(long)]
    model_dir: Option<PathBuf>,

    #[arg(long, default_value = "gemma4:e4b")]
    model: String,

    #[arg(long, help = "Upload CSV results to Google Sheets")]
    sheets: bool,
}

fn parse_format(s: &str) -> Result<pipeline::OutputFormat, String> {
    match s.to_lowercase().as_str() {
        "csv" => Ok(pipeline::OutputFormat::Csv),
        "text" | "txt" => Ok(pipeline::OutputFormat::Text),
        _ => Err(format!("unknown format: {}, use csv or text", s)),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    eprintln!("Loading OCR models...");
    let (detect, rec) = models::ensure_models(cli.model_dir.as_deref())?;
    let engine = ocr::create_engine(&detect, &rec)?;

    let inputs = pipeline::collect_inputs(&cli.input);
    if inputs.is_empty() {
        anyhow::bail!("no PDF or image files found in {}", cli.input.display());
    }

    eprintln!("Processing {} files...", inputs.len());
    let results = pipeline::run_batch(
        inputs,
        &cli.output,
        cli.dpi,
        cli.format,
        engine,
        &cli.model,
        cli.sheets,
    );

    let ok = results.iter().filter(|r| r.is_ok()).count();
    let err = results.iter().filter(|r| r.is_err()).count();
    eprintln!("Done: {} succeeded, {} failed", ok, err);

    if err > 0 {
        std::process::exit(1);
    }
    Ok(())
}
