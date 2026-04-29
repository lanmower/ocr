# ocr

Batch bank statement processor — Gemma 4 vision LLM reads PDF/image pages directly, outputs CSV/JSON. No OCR step, no daemon, no external services.

## Quick start (Windows x64)

```
git clone https://github.com/lanmower/ocr.git
cd ocr
start.bat --input ./test-statements --output ./out --format csv
```

First run downloads ~4.3 GB of runtime (model + multimodal projector + llama-server) into `tmp-llama/`. Subsequent runs reuse it. Requires [Rust](https://rustup.rs) for the initial build.

## Output formats

```
start.bat --input ./statements --output ./out --format csv
start.bat --input ./statements --output ./out --format text
start.bat --input ./statements --output ./out --format json
```

## What `start.bat` does

1. `setup.bat` fetches the model, mmproj, and `llama-server.exe` if missing.
2. `cargo build --release` builds `ocr.exe` if missing.
3. Starts `llama-server` on `127.0.0.1:8080`, waits for `/health`.
4. Runs the pipeline.

`ocr.bat` does the same but kills the server on exit.

## Runtime bundle

Auto-downloaded into `tmp-llama/` (gitignored):

- Model: [`bartowski/google_gemma-4-E2B-it-GGUF`](https://huggingface.co/bartowski/google_gemma-4-E2B-it-GGUF) Q4_K_M (~3.3 GB)
- Multimodal projector: `mmproj-google_gemma-4-E2B-it-f16.gguf` (~940 MB)
- Server: [`llama.cpp` b8785 win-cpu-x64](https://github.com/ggml-org/llama.cpp/releases/tag/b8785) (~40 MB)
