# Bank Statement Vision Pipeline

Rust batch pipeline for bank statement processing using Ollama (gemma4:e2b) — reads images/PDFs directly, outputs structured CSV/JSON. No OCR step, no daemon, no listening port.

## Architecture

- `src/main.rs` — CLI (clap) entry point
- `src/pdf.rs` — PDF to image rendering via pdfium-render (auto-downloads pdfium.dll)
- `src/llm.rs` — Vision inference: encodes page PNGs as base64, POSTs to Ollama `/api/generate`, parses JSON
- `src/pipeline.rs` — Batch processing

## Runtime Dependencies

- Ollama running at `http://localhost:11434` with `gemma4:e2b` model pulled
- pdfium.dll auto-downloaded next to the executable

## Build

Requires Rust stable (rustup). If chocolatey rust is also installed, ensure rustup's rustc takes precedence:

```
set PATH=C:\Users\user\.cargo\bin;%PATH%
cargo build --release
```

No build script — `build.rs` removed. `ureq` requires `features = ["json"]` in Cargo.toml (not enabled by default in ureq 3).

## Usage

```
ollama pull gemma4:e2b
ollama serve
ocr --input ./statements --output ./results --format csv
ocr --input ./statements --output ./results --format text
```

## Output Formats

- `text` (default): Page listing only
- `csv`: date,description,amount,balance CSV extracted by vision LLM
- `json`: Structured ExtractionResult with confidence scores

## CI / Releases

GitHub Actions builds `ocr.exe` on `windows-latest` on every push to master and uploads it as a rolling `latest` release. Workflow: `.github/workflows/release.yml`.
