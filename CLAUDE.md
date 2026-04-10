# Bank Statement Vision Pipeline

Rust batch pipeline for bank statement processing using Ollama vision LLM (Gemma 4) — reads images/PDFs directly, outputs structured CSV/JSON with no intermediate OCR step.

## Architecture

- `src/main.rs` — CLI (clap) entry point
- `src/pdf.rs` — PDF to image rendering via pdfium-render (auto-downloads pdfium.dll)
- `src/llm.rs` — Ollama vision API integration: encodes images as base64, calls Gemma 4, parses JSON
- `src/pipeline.rs` — Batch processing

## Runtime Dependencies

- Ollama running locally (`ollama serve`) with a vision-capable model pulled
- pdfium.dll auto-downloaded on first run next to the executable

## Build

Requires Rust 1.89+ (use rustup stable, not chocolatey rust).

```
set PATH=C:\Users\user\.cargo\bin;%PATH%
cargo build --release
```

## Usage

```
ocr --input ./statements --output ./results --format csv
ocr --input ./statements --output ./results --format text
ocr --input ./statements --format csv --model gemma4:e4b
```

## Output Formats

- `text` (default): Page listing only
- `csv`: date,description,amount,balance CSV extracted by vision LLM
- `json`: Structured ExtractionResult with confidence scores
