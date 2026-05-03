use anyhow::{Context, Result};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::io::Write;

pub const DEFAULT_MODEL: &str = "local";

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

fn encode_image(img: &DynamicImage) -> Result<String> {
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).context("encode png")?;
    Ok(base64(buf.get_ref()))
}

fn base64(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b = match chunk.len() {
            3 => [chunk[0], chunk[1], chunk[2]],
            2 => [chunk[0], chunk[1], 0],
            _ => [chunk[0], 0, 0],
        };
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() >= 2 { CHARS[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() == 3 { CHARS[(n & 63) as usize] as char } else { '=' });
    }
    out
}

fn prompt() -> &'static str {
    "Extract every transaction from this bank statement. Return only valid JSON: {\"transactions\": [{\"date\": {\"value\": string|null, \"confidence\": number, \"source\": string}, \"description\": {\"value\": string|null, \"confidence\": number, \"source\": string}, \"amount\": {\"value\": string|null, \"confidence\": number, \"source\": string}, \"balance\": {\"value\": string|null, \"confidence\": number, \"source\": string}}]}. Extract only transaction rows. Keep values exactly as shown. Null for missing. Confidence 0.0-1.0."
}

fn describe_prompt() -> &'static str {
    "Describe every possible detail in this image. Include all visible text, objects, colors, spatial layout, numbers, context, and any other observable information. Be thorough and precise."
}

fn strip_think(s: &str) -> &str {
    if let Some(end) = s.find("</think>") { &s[end + 8..] } else { s }
}

fn extract_json(raw: &str) -> String {
    let s = strip_think(raw.trim()).trim().trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```").trim();
    let bytes = s.as_bytes();
    let mut start = None;
    let mut depth = 0usize;
    let mut in_str = false;
    let mut esc = false;
    for (i, b) in bytes.iter().enumerate() {
        match *b {
            b'\\' if in_str => esc = !esc,
            b'"' if !esc => in_str = !in_str,
            b'{' if !in_str => { if start.is_none() { start = Some(i); } depth += 1; }
            b'}' if !in_str => { if depth > 0 { depth -= 1; if depth == 0 { if let Some(st) = start { return s[st..=i].to_string(); } } } }
            _ => esc = false,
        }
    }
    s.to_string()
}

fn csv_escape(v: &str) -> String {
    if v.contains([',', '"', '\n']) { format!("\"{}\"", v.replace('"', "\"\"")) } else { v.to_string() }
}

fn to_csv(records: &[TransactionRecord]) -> String {
    let mut out = String::from("date,description,amount,balance\n");
    for r in records {
        out.push_str(&format!("{},{},{},{}\n",
            csv_escape(r.date.value.as_deref().unwrap_or("")),
            csv_escape(r.description.value.as_deref().unwrap_or("")),
            csv_escape(r.amount.value.as_deref().unwrap_or("")),
            csv_escape(r.balance.value.as_deref().unwrap_or(""))));
    }
    out
}

pub fn process_images_to_csv(images: &[DynamicImage], model: &str) -> Result<String> {
    Ok(to_csv(&process_images_to_json(images, model)?.transactions))
}

pub fn process_images_to_json(images: &[DynamicImage], model: &str) -> Result<ExtractionResult> {
    let content: Vec<serde_json::Value> = std::iter::once(serde_json::json!({"type": "text", "text": prompt()}))
        .chain(images.iter().map(|img| -> Result<serde_json::Value> {
            Ok(serde_json::json!({"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", encode_image(img)?)}}))
        }).collect::<Result<Vec<_>>>()?.into_iter())
        .collect();
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "stream": false
    });
    let host = std::env::var("LLAMA_HOST").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    eprintln!("[llm] calling llama-server at {} with {} image(s)", host, images.len());
    let mut resp = ureq::post(&format!("http://{}/v1/chat/completions", host))
        .send_json(&body)
        .context("llama-server request")?;
    let raw = resp.body_mut().read_to_string().context("read llama-server response")?;
    let obj: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse llama-server response: {}", &raw[..raw.len().min(200)]))?;
    let msg = &obj["choices"][0]["message"];
    let text = msg["content"].as_str()
        .or_else(|| msg["reasoning_content"].as_str())
        .with_context(|| format!("missing content field: {}", &raw[..raw.len().min(200)]))?;
    let json = extract_json(text);
    serde_json::from_str::<ExtractionResult>(&json)
        .with_context(|| format!("parse llm json: {}", &json[..json.len().min(300)]))
}

pub fn write_text(images: &[DynamicImage], w: &mut impl Write) -> Result<()> {
    for (i, _) in images.iter().enumerate() {
        writeln!(w, "[page {}]", i + 1)?;
    }
    Ok(())
}

pub fn process_images_to_describe(images: &[DynamicImage], model: &str) -> Result<String> {
    let content: Vec<serde_json::Value> = std::iter::once(serde_json::json!({"type": "text", "text": describe_prompt()}))
        .chain(images.iter().map(|img| -> Result<serde_json::Value> {
            Ok(serde_json::json!({"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", encode_image(img)?)}}))
        }).collect::<Result<Vec<_>>>()?.into_iter())
        .collect();
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "stream": false
    });
    let host = std::env::var("LLAMA_HOST").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    eprintln!("[llm] calling llama-server at {} with {} image(s) [describe]", host, images.len());
    let mut resp = ureq::post(&format!("http://{}/v1/chat/completions", host))
        .send_json(&body)
        .context("llama-server request")?;
    let raw = resp.body_mut().read_to_string().context("read llama-server response")?;
    let obj: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse llama-server response: {}", &raw[..raw.len().min(200)]))?;
    let msg = &obj["choices"][0]["message"];
    let text = msg["content"].as_str()
        .or_else(|| msg["reasoning_content"].as_str())
        .with_context(|| format!("missing content field: {}", &raw[..raw.len().min(200)]))?;
    Ok(strip_think(text).trim().to_string())
}
