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
