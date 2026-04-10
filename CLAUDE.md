# Bank Statement Vision Pipeline

Rust batch pipeline for bank statement processing using llama-mtmd-cli (llama.cpp) with Gemma 4 E2B vision model — reads images/PDFs directly, outputs structured CSV/JSON. No OCR step, no daemon, no listening port.

## Architecture

- `src/main.rs` — CLI (clap) entry point
- `src/pdf.rs` — PDF to image rendering via pdfium-render (auto-downloads pdfium.dll)
- `src/runtime.rs` — Extracts embedded DLLs + downloads GGUF shards from GitHub release on first run
- `src/llm.rs` — Vision inference: writes page PNGs to temp, calls llama-mtmd-cli subprocess, parses JSON
- `src/pipeline.rs` — Batch processing

## Runtime Dependencies

- `llama-mtmd-cli.exe` + 7 DLLs (`mtmd`, `ggml-vulkan`, `ggml-base`, `ggml`, `llama`, `libomp140.x86_64`, `ggml-cpu-x64`) embedded in `ocr.exe` via `include_bytes!` at build time (from llama.cpp b8741 Vulkan zip), extracted to `llm-runtime/` on first run
- 3 model GGUF shards (~1.5GB total) + `mmproj-google_gemma-4-E2B-it-f16.gguf` (~300MB) downloaded from GitHub release on first run into `llm-runtime/`
- pdfium.dll auto-downloaded next to the executable

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
```

## Output Formats

- `text` (default): Page listing only
- `csv`: date,description,amount,balance CSV extracted by vision LLM
- `json`: Structured ExtractionResult with confidence scores

## CI / Releases

GitHub Actions builds `ocr.exe` on `windows-latest` on every push to master and uploads it as a rolling `latest` release. Workflow: `.github/workflows/release.yml`.
