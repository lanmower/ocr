# ocr

Single-exe Windows bundle: webcam vision chat UI + batch bank-statement OCR, powered by Gemma 4 E2B running locally via `llama-server`.

## Quick start

1. Download `ocr.exe` from the [latest release](https://github.com/lanmower/ocr/releases/tag/latest).
2. Drop it in any folder.
3. Double-click it (or run `ocr.exe` from a terminal).

On first run it detects your GPU, downloads the matching `llama-server` build, the multimodal projector, and the Q4_K_M model (~4.3 GB total) into `.ocr-runtime/` next to the exe. Subsequent runs reuse the cache.

When the server is ready it prints:

```
Webcam chat UI ready at: http://127.0.0.1:8080/
```

Open that URL in a browser for the chat UI.

## Batch OCR mode

```
ocr.exe --input ./statements --output ./out --format csv
```

Formats: `csv`, `json`, `text`, `describe`.

## GPU support

`ocr.exe` auto-detects:

| GPU                | Variant | `-ngl` |
|--------------------|---------|--------|
| NVIDIA             | cuda    | 99     |
| AMD / Radeon / Intel Arc | vulkan | 99 |
| (none)             | cpu     | 0      |

## Build from source

Requires [Rust](https://rustup.rs) (MSVC toolchain on Windows).

```
cargo build --release
```

The resulting `target/release/ocr.exe` is the same single-file deliverable.

## Releases

Every push to `master` triggers `.github/workflows/release.yml`, which builds `ocr.exe` and uploads it alongside the auxiliary runtime assets (mmproj + four `llama-server` zips) to the `latest` release. The model itself is fetched from HuggingFace at runtime because it exceeds GitHub's 2 GB per-asset cap.
