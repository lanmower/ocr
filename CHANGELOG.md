# Changelog

## 0.2.0

- Add Gemini API integration for CSV output
- Auth via gcloud ADC or GEMINI_API_KEY env var
- CSV mode sends OCR text to Gemini for structured extraction
- Default model: gemini-2.5-flash
- Remove regex-based CSV heuristics in favor of LLM parsing

## 0.1.0

- Initial batch OCR pipeline
- PDF rendering via pdfium-render
- OCR via ocrs (pure Rust, ONNX models)
- Text output mode
- Rayon-based parallel file processing
- Auto model download from S3
