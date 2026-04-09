use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: Option<String>,
}

fn ollama_url() -> String {
    std::env::var("OLLAMA_HOST")
        .unwrap_or_else(|_| "http://localhost:11434".to_string())
}

fn build_prompt(ocr_text: &str) -> String {
    format!(
        "You are a bank statement data extraction system.\n\
         Extract transaction rows from this OCR text into CSV.\n\n\
         Schema: date,description,amount,balance\n\n\
         Rules:\n\
         - Output ONLY valid CSV starting with the header row\n\
         - No markdown fences, no explanation, no commentary\n\
         - Use exact values from the text\n\
         - Negative amounts for debits/withdrawals, positive for credits/deposits\n\
         - If a field is unclear or missing, leave it empty\n\
         - Skip headers, footers, page numbers, bank logos, account summaries\n\
         - Each transaction is one row\n\
         - Dates should be in the format they appear in the text\n\n\
         OCR Text:\n{}",
        ocr_text
    )
}

fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    if text.len() <= max_chars {
        return vec![text.to_string()];
    }

    let overlap = max_chars / 5;
    let step = max_chars - overlap;
    let lines: Vec<&str> = text.lines().collect();
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < lines.len() {
        let mut chunk_len = 0;
        let mut end = start;
        while end < lines.len() {
            let line_len = lines[end].len() + 1;
            if chunk_len + line_len > max_chars && end > start {
                break;
            }
            chunk_len += line_len;
            end += 1;
        }
        chunks.push(lines[start..end].join("\n"));
        let lines_in_step = {
            let mut count = 0;
            let mut len = 0;
            for i in start..end {
                len += lines[i].len() + 1;
                count += 1;
                if len >= step {
                    break;
                }
            }
            count
        };
        start += lines_in_step;
    }

    chunks
}

fn call_ollama(prompt: &str, model: &str) -> Result<String> {
    let url = format!("{}/api/generate", ollama_url());

    let body = OllamaRequest {
        model: model.to_string(),
        prompt: prompt.to_string(),
        stream: false,
    };

    let resp: OllamaResponse = ureq::post(&url)
        .header("Content-Type", "application/json")
        .send_json(&body)
        .context("ollama request failed - is ollama running?")?
        .body_mut()
        .read_json()
        .context("parse ollama response")?;

    Ok(resp.response.unwrap_or_default())
}

fn clean_csv(raw: &str) -> String {
    let backticks = "```";
    let csv_fence = "```csv";
    raw.trim()
        .strip_prefix(csv_fence).unwrap_or(raw.trim())
        .strip_prefix(backticks).unwrap_or(raw.trim())
        .strip_suffix(backticks).unwrap_or(raw.trim())
        .trim()
        .to_string()
}

pub fn process_ocr_text(ocr_text: &str, model: &str) -> Result<String> {
    let chunks = chunk_text(ocr_text, 4000);

    if chunks.len() == 1 {
        let prompt = build_prompt(ocr_text);
        let raw = call_ollama(&prompt, model)?;
        return Ok(clean_csv(&raw));
    }

    eprintln!("[llm] processing {} chunks...", chunks.len());
    let mut all_rows: Vec<String> = Vec::new();
    let mut header = String::new();

    for (i, chunk) in chunks.iter().enumerate() {
        let prompt = build_prompt(chunk);
        let raw = call_ollama(&prompt, model)?;
        let csv = clean_csv(&raw);

        for (j, line) in csv.lines().enumerate() {
            if j == 0 && line.contains("date") {
                if header.is_empty() {
                    header = line.to_string();
                }
                continue;
            }
            if !line.trim().is_empty() {
                all_rows.push(line.to_string());
            }
        }
        eprintln!("[llm] chunk {}/{} done ({} rows)", i + 1, chunks.len(), all_rows.len());
    }

    let mut result = header;
    for row in &all_rows {
        result.push('\n');
        result.push_str(row);
    }
    Ok(result)
}
