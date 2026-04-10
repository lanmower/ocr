# Bank Statement Vision Pipeline

Rust batch pipeline for bank statement processing using Google Gemini API (gemini-2.0-flash) via Python subprocess — reads images/PDFs directly, outputs structured CSV/JSON. No OCR step, no daemon, no listening port.

## Architecture

- `src/main.rs` — CLI (clap) entry point
- `src/pdf.rs` — PDF to image rendering via pdfium-render (auto-downloads pdfium.dll)
- `src/runtime.rs` — Embeds `src/infer.py` via `include_str!`, extracts to `llm-runtime/` on first run, verifies python + google-genai
- `src/llm.rs` — Vision inference: writes page PNGs to temp, spawns `python infer.py --images ... --prompt ...`, parses JSON
- `src/pipeline.rs` — Batch processing
- `src/infer.py` — Python script embedded in binary; calls gemini-2.0-flash with image bytes + prompt, prints JSON

## Runtime Dependencies

- Python 3 on PATH with `google-genai` package installed (`pip install google-genai`)
- `GEMINI_API_KEY` environment variable set to a valid Gemini API key
- pdfium.dll auto-downloaded next to the executable

## Build

Requires Rust stable (rustup). If chocolatey rust is also installed, ensure rustup's rustc takes precedence:

```
set PATH=C:\Users\user\.cargo\bin;%PATH%
cargo build --release
```

`build.rs` embeds `src/infer.py` path via `cargo:rustc-env=INFER_PY`. `runtime.rs` uses `include_str!(env!("INFER_PY"))` to embed the Python script at compile time.

## Usage

```
set GEMINI_API_KEY=<your key>
ocr --input ./statements --output ./results --format csv
ocr --input ./statements --output ./results --format text
```

## Output Formats

- `text` (default): Page listing only
- `csv`: date,description,amount,balance CSV extracted by vision LLM
- `json`: Structured ExtractionResult with confidence scores

## CI / Releases

GitHub Actions builds `ocr.exe` on `windows-latest` on every push to master and uploads it as a rolling `latest` release. Workflow: `.github/workflows/release.yml`.
