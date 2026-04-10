# ocr

Batch bank statement processor — vision LLM reads PDF/image pages directly, outputs CSV/JSON. No OCR step.

## Requirements

- Python 3 with `google-genai` installed: `pip install google-genai`
- `GEMINI_API_KEY` environment variable

## Download

Get `ocr.exe` from the [latest release](https://github.com/lanmower/ocr/releases/tag/latest).

## Usage

```
set GEMINI_API_KEY=<your key>
ocr --input ./statements --output ./results --format csv
ocr --input ./statements --output ./results --format text
ocr --input ./statements --output ./results --format json
```

On first run, extracts `infer.py` to `llm-runtime/` next to the executable.

## Build

```
cargo build --release
```
