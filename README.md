# ocr

Batch bank statement processor — vision LLM reads PDF/image pages directly, outputs CSV/JSON. No OCR step.

## Requirements

- [Ollama](https://ollama.com) running locally with `gemma4:e2b` pulled

```
ollama pull gemma4:e2b
```

## Download

Get `ocr.exe` from the [latest release](https://github.com/lanmower/ocr/releases/tag/latest).

## Usage

```
ocr --input ./statements --output ./results --format csv
ocr --input ./statements --output ./results --format text
ocr --input ./statements --output ./results --format json
```

## Build

```
cargo build --release
```
