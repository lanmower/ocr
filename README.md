# ocr

Batch bank statement processor — Gemma 4 vision LLM reads PDF/image pages directly, outputs CSV/JSON. No OCR step.

## Requirements

- Windows (x64)
- `tmp-llama/` runtime bundle: `model-q4km.gguf`, `mmproj-google_gemma-4-E2B-it-f16.gguf`, `b8785-extracted/llama-server.exe`

## Usage

`start.bat` boots `llama-server` on `127.0.0.1:8080`, waits for `/health`, runs the pipeline, then shuts the server down.

```
start.bat --input ./statements --output ./results --format csv
start.bat --input ./statements --output ./results --format text
start.bat --input ./statements --output ./results --format json
```

## Build

```
cargo build --release
```
