mod bootstrap;
mod llm;
mod pdf;
mod pipeline;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ocr", about = "Single-exe vision LLM bundle (webcam chat UI + batch OCR)")]
struct Cli {
    #[arg(short, long)]
    input: Option<PathBuf>,

    #[arg(short, long, default_value = "./output")]
    output: PathBuf,

    #[arg(long, default_value = "300")]
    dpi: u32,

    #[arg(long, default_value = "text", value_parser = parse_format)]
    format: pipeline::OutputFormat,

    #[arg(long, default_value = llm::default_model_name())]
    model: String,

    /// Keep llama-server running in foreground (default when no --input given)
    #[arg(long)]
    serve: bool,
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

    let mut server = bootstrap::ensure_running()?;

    if cli.input.is_none() || cli.serve {
        eprintln!();
        eprintln!("==============================================");
        eprintln!("  Webcam chat UI ready at: http://127.0.0.1:8080/");
        eprintln!("  Press Ctrl-C to stop.");
        eprintln!("==============================================");
        let _ = server.wait();
        return Ok(());
    }

    let input = cli.input.expect("checked above");
    let inputs = pipeline::collect_inputs(&input);
    if inputs.is_empty() {
        let _ = server.kill();
        anyhow::bail!("no PDF or image files found in {}", input.display());
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

    let _ = server.kill();

    if err > 0 {
        std::process::exit(1);
    }
    Ok(())
}
