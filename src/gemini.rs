use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta/models";

#[derive(Serialize)]
struct Request {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct Response {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

enum Auth {
    ApiKey(String),
    Bearer(String),
}

fn get_gcloud_token() -> Result<String> {
    let output = std::process::Command::new("gcloud")
        .args(["auth", "application-default", "print-access-token"])
        .output()
        .context("gcloud not found")?;
    if !output.status.success() {
        anyhow::bail!("gcloud token failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn load_auth() -> Result<Auth> {
    if let Ok(key) = std::env::var("GEMINI_API_KEY") {
        if !key.is_empty() {
            return Ok(Auth::ApiKey(key));
        }
    }

    if let Ok(token) = get_gcloud_token() {
        if !token.is_empty() {
            return Ok(Auth::Bearer(token));
        }
    }

    anyhow::bail!(
        "No auth found. Either:\n\
         1. Run: gcloud auth application-default login\n\
         2. Set GEMINI_API_KEY env var (get free key at https://aistudio.google.com/apikey)"
    )
}

pub fn process_ocr_text(ocr_text: &str, model: &str) -> Result<String> {
    let auth = load_auth()?;

    let prompt = format!(
        "Convert this OCR text from a bank statement into CSV format.\n\
         Columns: date,description,amount,balance\n\
         Rules:\n\
         - Output ONLY valid CSV with header row, no markdown, no explanation\n\
         - Use exact values from text, do not invent data\n\
         - Leave unclear fields empty\n\
         - Negative amounts for debits, positive for credits\n\
         - Skip non-transaction header/footer lines\n\n\
         OCR Text:\n{}",
        ocr_text
    );

    let url = match &auth {
        Auth::ApiKey(key) => format!("{}/{}:generateContent?key={}", ENDPOINT, model, key),
        Auth::Bearer(_) => format!("{}/{}:generateContent", ENDPOINT, model),
    };

    let mut req = ureq::post(&url).header("Content-Type", "application/json");
    if let Auth::Bearer(token) = &auth {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }

    let body = Request {
        contents: vec![Content {
            parts: vec![Part { text: prompt }],
        }],
    };

    let resp: Response = req
        .send_json(&body)
        .context("gemini api call failed")?
        .body_mut()
        .read_json()
        .context("parse gemini response")?;

    let text = resp
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content.parts.into_iter().next())
        .and_then(|p| p.text)
        .unwrap_or_default();

    Ok(text.trim()
        .trim_start_matches("```csv")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string())
}
