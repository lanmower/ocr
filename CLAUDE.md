# Bank Statement Vision Pipeline

Rust batch pipeline for bank statement processing using llama-server (gemma4 E2B) — reads images/PDFs directly, outputs structured CSV/JSON. No OCR step, no daemon, no external dependencies.

## Architecture

- `src/main.rs` — CLI (clap) entry point
- `src/pdf.rs` — PDF to image rendering via pdfium-render (auto-downloads pdfium.dll)
- `src/llm.rs` — Vision inference: encodes page PNGs as base64, POSTs to llama-server `/v1/chat/completions`, parses JSON
- `src/pipeline.rs` — Batch processing

## Runtime Dependencies

All bundled in `tmp-llama/`:
- `tmp-llama/model.gguf` — Gemma 4 E2B 4.6B quantized model (~4GB)
- `tmp-llama/mmproj-google_gemma-4-E2B-it-f16.gguf` — multimodal projector (~940MB)
- `tmp-llama/unzipped/llama-server.exe` + DLLs — llama.cpp server

## Build

Requires Rust stable (rustup). If chocolatey rust is also installed, ensure rustup's rustc takes precedence:

```
set PATH=C:\Users\user\.cargo\bin;%PATH%
cargo build --release
```

No build script — `build.rs` removed. `ureq` requires `features = ["json"]` in Cargo.toml (not enabled by default in ureq 3).

## Usage

```
start.bat --input ./test-statements --output ./test-results --format csv
```

`start.bat` launches `llama-server` on `127.0.0.1:8080`, waits for `/health`, then runs the pipeline. Model loads from `tmp-llama/` — no internet required.

To override the server address:
```
set LLAMA_HOST=127.0.0.1:8081
target\release\ocr.exe --input ./statements --output ./results --format csv
```

## Output Formats

- `text` (default): Page listing only
- `csv`: date,description,amount,balance CSV extracted by vision LLM
- `json`: Structured ExtractionResult with confidence scores

## CI / Releases

GitHub Actions builds `ocr.exe` on `windows-latest` on every push to master and uploads it as a rolling `latest` release. Workflow: `.github/workflows/release.yml`.
