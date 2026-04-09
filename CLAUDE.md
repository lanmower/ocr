# OCR Pipeline

Rust batch OCR pipeline for bank statement processing.

## Architecture

- `src/main.rs` — CLI (clap) entry point
- `src/models.rs` — ONNX model auto-download from S3
- `src/pdf.rs` — PDF to image rendering via pdfium-render
- `src/ocr.rs` — OCR engine wrapper around ocrs (rten-based)
- `src/parse.rs` — Heuristic text-to-CSV extraction (date/desc/amount)
- `src/pipeline.rs` — Batch processing with rayon parallelism

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

## Usage

```
ocr --input ./bank-statements --output ./results --dpi 300 --format csv
ocr --input ./bank-statements --output ./results --format text
```

## Output Formats

- `csv`: Attempts heuristic parsing of date/description/amount columns
- `text`: Raw OCR text lines (for LLM post-processing)
