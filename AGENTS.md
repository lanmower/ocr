# AGENTS.md — Non-obvious technical caveats

## Single-exe distribution

`ocr.exe` is a self-contained Windows binary. On first run it populates `<exe-dir>/.ocr-runtime/`:

- `llama/llama-server.exe` (+ DLLs from the variant zip, + cudart DLLs if CUDA)
- `model.gguf` (~3.3 GB official Google Gemma 4 E2B QAT q4_0 from HuggingFace — never bundled, exceeds GH's 2 GB per-asset cap)
- `gemma-4-E2B-it-mmproj.gguf` (~987 MB — bundled on our release, falls back to HF)
- `web/index.html` (extracted from the exe via `include_bytes!`)
- `variant.txt` (cpu / cuda / vulkan)

GPU variant detection lives in `src/bootstrap.rs::detect_variant` — PowerShell `Win32_VideoController` name match: NVIDIA → cuda, AMD/Radeon/Intel Arc → vulkan, else cpu. `-ngl 99` for GPU variants, `-ngl 0` for cpu.

## Run modes

- `ocr.exe` (no args) or `ocr.exe --serve` — bootstrap, start llama-server, print URL, block on Ctrl-C. Webcam chat UI at `http://127.0.0.1:8080/`.
- `ocr.exe --input <path>` — bootstrap, start server, run the OCR batch pipeline, kill server, exit.

## Asset URL fallback

`bootstrap.rs::try_download` prefers `https://github.com/lanmower/ocr/releases/download/latest/<name>` (our own release mirror) and falls back to the upstream URL (ggml-org/llama.cpp for the server zips, HuggingFace for the mmproj). The model.gguf has no mirror — only HuggingFace — because it exceeds the GitHub release per-asset 2 GB cap.

## CI

`.github/workflows/release.yml` runs on every push to master. It cargo builds, downloads the four upstream zips + the mmproj, and attaches all of them plus `ocr.exe` to the `latest` GitHub release. Result: ~5 release assets totaling under ~2 GB each, single source of truth.

## Toolchain note

Build cleanly requires the MSVC toolchain — the windows-gnu toolchain on a typical dev box can struggle to link `ring` (transitive via `ureq` → `rustls`). CI uses MSVC.

@.gm/next-step.md
