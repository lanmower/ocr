use crate::runtime;
use anyhow::{Context, Result};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

pub const DEFAULT_MODEL: &str = "gemma4-e2b-q4km";

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

fn write_image(img: &DynamicImage, path: &PathBuf) -> Result<()> {
    let f = std::fs::File::create(path).context("create temp image")?;
    let mut w = std::io::BufWriter::new(f);
    img.write_to(&mut w, image::ImageFormat::Png).context("encode image png")
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

pub fn process_images_to_csv(images: &[DynamicImage], _model: &str) -> Result<String> {
    Ok(to_csv(&process_images_to_json(images, _model)?.transactions))
}

pub fn process_images_to_json(images: &[DynamicImage], _model: &str) -> Result<ExtractionResult> {
    let rt = runtime::ensure()?;
    let tmp_dir = std::env::temp_dir().join("ocr-imgs");
    std::fs::create_dir_all(&tmp_dir).context("create tmp img dir")?;

    let mut paths: Vec<PathBuf> = Vec::new();
    for (i, img) in images.iter().enumerate() {
        let p = tmp_dir.join(format!("page-{}.png", i));
        write_image(img, &p)?;
        paths.push(p);
    }

    let mut cmd = std::process::Command::new(&rt.cli);
    cmd.arg("-m").arg(&rt.model)
        .arg("--mmproj").arg(&rt.mmproj)
        .arg("-n").arg("2048")
        .arg("-p").arg(prompt());
    for p in &paths {
        cmd.arg("--image").arg(p);
    }

    eprintln!("[llm] running llama-mtmd-cli with {} image(s)", paths.len());
    let out = cmd.output().context("run llama-mtmd-cli")?;

    for p in &paths {
        let _ = std::fs::remove_file(p);
    }

    if !out.status.success() {
        anyhow::bail!("llama-mtmd-cli failed: {}", String::from_utf8_lossy(&out.stderr).trim());
    }

    let raw = String::from_utf8_lossy(&out.stdout).to_string();
    let json = extract_json(&raw);
    serde_json::from_str::<ExtractionResult>(&json)
        .with_context(|| format!("parse llm json: {}", &raw[..raw.len().min(300)]))
}

pub fn write_text(images: &[DynamicImage], w: &mut impl Write) -> Result<()> {
    for (i, _) in images.iter().enumerate() {
        writeln!(w, "[page {}]", i + 1)?;
    }
    Ok(())
}
