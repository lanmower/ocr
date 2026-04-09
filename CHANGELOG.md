# Changelog

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
