use anyhow::{Context, Result};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::io::Write;

pub const DEFAULT_MODEL: &str = "gemma4:e2b";

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

fn extract_json(raw: &str) -> String {
    let s = raw.trim().trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```").trim();
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
    let imgs: Vec<String> = images.iter().map(encode_image).collect::<Result<_>>()?;
    let body = serde_json::json!({
        "model": model,
        "prompt": prompt(),
        "images": imgs,
        "stream": false
    });
    eprintln!("[llm] calling ollama with {} image(s) model={}", imgs.len(), model);
    let mut resp = ureq::post("http://localhost:11434/api/generate")
        .send_json(&body)
        .context("ollama request")?;
    let raw = resp.body_mut().read_to_string().context("read ollama response")?;
    let obj: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse ollama response: {}", &raw[..raw.len().min(200)]))?;
    let text = obj["response"].as_str()
        .with_context(|| format!("missing response field: {}", &raw[..raw.len().min(200)]))?;
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
