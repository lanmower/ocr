# ocr

Batch bank statement processor — vision LLM reads PDF/image pages directly, outputs CSV/JSON. No OCR step.

## Download

Get `ocr.exe` from the [latest release](https://github.com/lanmower/ocr/releases/tag/latest).

## Usage

```
ocr --input ./statements --output ./results --format csv
ocr --input ./statements --output ./results --format text
ocr --input ./statements --output ./results --format json
```

On first run, downloads ~2GB of models into `llm-runtime/` next to the executable. GPU (Vulkan) auto-detected.

## Build

```
cargo build --release
```
