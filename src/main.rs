mod llm;
mod pdf;
mod pipeline;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ocr", about = "Batch bank statement processing via vision LLM")]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long, default_value = "./output")]
    output: PathBuf,

    #[arg(long, default_value = "300")]
    dpi: u32,

    #[arg(long, default_value = "text", value_parser = parse_format)]
    format: pipeline::OutputFormat,

    #[arg(long, default_value = llm::default_model_name())]
    model: String,
}

fn parse_format(s: &str) -> Result<pipeline::OutputFormat, String> {
    match s.to_lowercase().as_str() {
        "csv" => Ok(pipeline::OutputFormat::Csv),
        "json" => Ok(pipeline::OutputFormat::Json),
        "text" | "txt" => Ok(pipeline::OutputFormat::Text),
        "describe" => Ok(pipeline::OutputFormat::Describe),
        _ => Err(format!("unknown format: {}, use csv, json, text or describe", s)),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

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
        &cli.model,
    );

    let ok = results.iter().filter(|r| r.is_ok()).count();
    let err = results.iter().filter(|r| r.is_err()).count();
    eprintln!("Done: {} succeeded, {} failed", ok, err);

    if err > 0 {
        std::process::exit(1);
    }
    Ok(())
}
