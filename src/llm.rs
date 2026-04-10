use anyhow::{Context, Result};
use base64::Engine as _;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;

const DEFAULT_MODEL: &str = "gemma4:e4b";
const OLLAMA_URL: &str = "http://localhost:11434/api/generate";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionField {
    pub value: Option<String>,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub date: TransactionField,
    pub description: TransactionField,
    pub amount: TransactionField,
    pub balance: TransactionField,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractionResult {
    #[serde(default)]
    pub transactions: Vec<TransactionRecord>,
}

pub fn default_model_name() -> &'static str {
    DEFAULT_MODEL
}

fn image_to_base64(img: &DynamicImage) -> Result<String> {
    let mut buf: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .context("encode image to PNG")?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

fn prompt_text() -> &'static str {
    "You are a bank statement extraction system. Extract every transaction from this bank statement image. Return only valid JSON matching this schema exactly: {\"transactions\": [{\"date\": {\"value\": string|null, \"confidence\": number, \"source\": string}, \"description\": {\"value\": string|null, \"confidence\": number, \"source\": string}, \"amount\": {\"value\": string|null, \"confidence\": number, \"source\": string}, \"balance\": {\"value\": string|null, \"confidence\": number, \"source\": string}}]}. Rules: extract only transaction rows, not headers or page furniture. Keep values exactly as shown. Use null for missing. Confidence is 0.0 to 1.0. Source is the exact text from the image."
}

fn call_ollama(images_b64: &[String], model: &str) -> Result<String> {
    let payload = json!({
        "model": model,
        "prompt": prompt_text(),
        "images": images_b64,
        "stream": false
    });

    let resp = ureq::post(OLLAMA_URL)
        .send_json(&payload)
        .context("call Ollama vision API")?;

    let body: serde_json::Value = resp.into_body().read_json()
        .context("parse Ollama response")?;

    body["response"]
        .as_str()
        .map(|s| s.to_string())
        .context("missing 'response' field in Ollama output")
}

fn extract_json(raw: &str) -> String {
    let s = raw.trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let bytes = s.as_bytes();
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (i, b) in bytes.iter().enumerate() {
        match *b {
            b'\\' if in_string => escaped = !escaped,
            b'"' if !escaped => in_string = !in_string,
            b'{' if !in_string => {
                if start.is_none() {
                    start = Some(i);
                }
                depth += 1;
            }
            b'}' if !in_string => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(st) = start {
                            return s[st..=i].to_string();
                        }
                    }
                }
            }
            _ => escaped = false,
        }
    }
    s.to_string()
}

fn csv_escape(v: &str) -> String {
    if v.contains([',', '"', '\n']) {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}

fn to_csv(records: &[TransactionRecord]) -> String {
    let mut out = String::from("date,description,amount,balance\n");
    for r in records {
        out.push_str(&format!(
            "{},{},{},{}\n",
            csv_escape(r.date.value.as_deref().unwrap_or("")),
            csv_escape(r.description.value.as_deref().unwrap_or("")),
            csv_escape(r.amount.value.as_deref().unwrap_or("")),
            csv_escape(r.balance.value.as_deref().unwrap_or(""))
        ));
    }
    out
}

pub fn process_images_to_csv(images: &[DynamicImage], model: &str) -> Result<String> {
    let result = process_images_to_json(images, model)?;
    Ok(to_csv(&result.transactions))
}

pub fn process_images_to_json(images: &[DynamicImage], model: &str) -> Result<ExtractionResult> {
    let encoded: Vec<String> = images
        .iter()
        .enumerate()
        .map(|(i, img)| image_to_base64(img).with_context(|| format!("encode image {}", i)))
        .collect::<Result<_>>()?;

    eprintln!("[llm] sending {} image(s) to {} via Ollama", encoded.len(), model);
    let raw = call_ollama(&encoded, model)?;
    let json = extract_json(&raw);
    serde_json::from_str::<ExtractionResult>(&json)
        .with_context(|| format!("parse LLM extraction JSON: {}", &raw[..raw.len().min(200)]))
}

pub fn write_text(images: &[DynamicImage], w: &mut impl Write) -> Result<()> {
    for (i, _) in images.iter().enumerate() {
        writeln!(w, "[page {}]", i + 1)?;
    }
    Ok(())
}
