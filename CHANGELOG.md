## 1.0.0

- Replace llama-mtmd-cli subprocess with Google Gemini API (gemini-2.0-flash) via Python
- Add src/infer.py: Python script using google-genai SDK, embedded in binary via include_str!
- Rewrite runtime.rs: extracts infer.py to llm-runtime/, verifies python + google-genai on startup
- Rewrite llm.rs: spawns `python infer.py --images ... --prompt ...` instead of llama-mtmd-cli
- Rewrite build.rs: emits INFER_PY env var pointing to src/infer.py, no more DLL download logic
- Simplify release.yml: uploads ocr.exe only, no model shard downloads
- Downgrade image crate to 0.24 (0.25 uses unstable Rust features); pdfium-render uses image_024 feature
- Remove build-dependencies (zip crate no longer needed)

## 0.9.0

- Embed all llama.cpp DLLs (ggml-vulkan, ggml-base, ggml, llama, mtmd, libomp, ggml-cpu-x64) into ocr.exe via include_bytes!
- Upgrade llama.cpp from b8740 to b8741 (Vulkan build)
- Split model GGUF into 3 shards (was 2) to stay under 1900MB per shard
- runtime.rs downloads 3 model shards from GitHub release
- Remove --no-display-prompt flag from llama-mtmd-cli invocation

## 0.8.0

- Upload 5 separate release assets instead of single bundle (bypasses GitHub 2GB per-file limit)
- runtime.rs downloads all 4 runtime files directly from GitHub release assets
- Remove zip crate dependency (no longer needed)

## 0.7.0

- Bundle all runtime files (ocr.exe + llama-mtmd-cli.exe + mtmd.dll + model GGUF + mmproj GGUF) into single ocr-bundle.zip release asset
- CI downloads models from HuggingFace (HF_TOKEN secret) during build
- runtime.rs fallback URLs updated to pull from GitHub release assets instead of HuggingFace

## 0.6.0

- Add GitHub Actions CI workflow: builds ocr.exe on windows-latest, uploads as rolling `latest` release on every push to master

# Changelog

## 0.5.0

- Replace Ollama HTTP with llama-mtmd-cli subprocess (no daemon, no port)
- Auto-detect GPU: downloads Vulkan zip (GPU) or CPU zip based on nvidia-smi/vulkaninfo
- Auto-downloads llama.cpp b8740 + Gemma 4 E2B Q4_K_M GGUF + mmproj-f16 on first run
- All runtime binaries/models stored in llm-runtime/ next to executable
- Split llm.rs (inference) and runtime.rs (download/setup)

## 0.4.0

- Replace two-step OCR→LLM pipeline with direct vision LLM processing
- Images sent as base64 directly to Ollama /api/generate (no OCR step)
- Remove ocrs, rten, models.rs, ocr.rs, parse.rs — no ONNX models needed
- Gemma 4 reads bank statement images and outputs structured JSON/CSV in one pass

## 0.3.0

- Replace Gemini API with local Ollama + Gemma 4 (no auth needed)
- Add document chunking with overlap for multi-page statements
- Improved extraction prompt inspired by llm-extract patterns
- Add Google Sheets upload via --sheets flag (uses gcloud auth)
- Default model: gemma4:e4b

## 0.2.0

- Add Gemini API integration for CSV output
- Auth via gcloud ADC or GEMINI_API_KEY env var

## 0.1.0

- Initial batch OCR pipeline
- PDF rendering via pdfium-render
- OCR via ocrs (pure Rust, ONNX models)
- Rayon-based parallel file processing
- Auto model download from S3
