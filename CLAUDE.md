# Bank Statement Vision Pipeline

Rust batch pipeline for bank statement processing using llama-mtmd-cli (llama.cpp) with Gemma 4 E2B vision model — reads images/PDFs directly, outputs structured CSV/JSON. No OCR step, no daemon, no listening port.

## Architecture

- `src/main.rs` — CLI (clap) entry point
- `src/pdf.rs` — PDF to image rendering via pdfium-render (auto-downloads pdfium.dll)
- `src/runtime.rs` — Auto-downloads llama.cpp + GGUF model + mmproj, GPU detection (Vulkan vs CPU)
- `src/llm.rs` — Vision inference: writes page PNGs to temp, calls llama-mtmd-cli subprocess, parses JSON
- `src/pipeline.rs` — Batch processing

## Runtime Dependencies

- All auto-downloaded on first run into `llm-runtime/` next to the executable:
  - `llama-mtmd-cli.exe` + `mtmd.dll` (from llama.cpp b8740 Vulkan or CPU zip)
  - `google_gemma-4-E2B-it-Q4_K_M.gguf` (~1.5GB)
  - `mmproj-google_gemma-4-E2B-it-f16.gguf` (~300MB)
- pdfium.dll auto-downloaded next to the executable

## GPU Detection

At startup, checks for `nvidia-smi` or `vulkaninfo`. If found, downloads the Vulkan-accelerated llama.cpp build. Otherwise downloads the CPU build. RTX 3060+ recommended.

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
