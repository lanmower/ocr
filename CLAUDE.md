# OCR Pipeline

Rust batch OCR pipeline for bank statement processing with Gemini AI post-processing.

## Architecture

- `src/main.rs` — CLI (clap) entry point
- `src/models.rs` — ONNX model auto-download from S3
- `src/pdf.rs` — PDF to image rendering via pdfium-render
- `src/ocr.rs` — OCR engine wrapper around ocrs (rten-based)
- `src/llm.rs` — Ollama LLM integration with chunking for CSV extraction
- `src/drive.rs` — Google Drive/Sheets upload via gcloud auth
- `src/parse.rs` — Text output utilities
- `src/pipeline.rs` — Batch processing with rayon parallelism

## Auth (Gemini)

Tries in order:
1. `GEMINI_API_KEY` env var
2. `gcloud auth application-default print-access-token`

For gcloud: `gcloud auth application-default login` (one-time setup).
For API key: get free key at https://aistudio.google.com/apikey

## Build

Requires Rust 1.89+ (use rustup stable, not chocolatey rust).

```
set PATH=C:\Users\user\.cargo\bin;%PATH%
set RUSTUP_TOOLCHAIN=stable
cargo build --release
```

## Runtime Dependencies

- pdfium library (pdfium.dll) must be in executable directory or system PATH
- ONNX models auto-downloaded on first run to `./models/` next to executable
- Ollama running locally (`ollama serve`) with a model pulled

## Usage

```
ocr --input ./statements --output ./results --format csv
ocr --input ./statements --output ./results --format text
ocr --input ./statements --format csv --model gemma4:e4b
ocr --input ./statements --format csv --sheets
```

## Output Formats

- `text` (default): Raw OCR text lines
- `csv`: OCR text processed by Gemini into date,description,amount,balance CSV
