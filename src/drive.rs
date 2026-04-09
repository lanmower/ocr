use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

const UPLOAD_URL: &str =
    "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&convert=true";

#[derive(Deserialize)]
struct DriveFile {
    id: Option<String>,
}

fn get_token() -> Result<String> {
    if let Ok(key) = std::env::var("GOOGLE_ACCESS_TOKEN") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    let output = std::process::Command::new("gcloud")
        .args(["auth", "application-default", "print-access-token"])
        .output()
        .context("gcloud not found")?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        if err.contains("Could not automatically determine credentials") {
            eprintln!("Drive access requires login with drive scope.");
            eprintln!("Run: gcloud auth application-default login \\");
            eprintln!("  --scopes=openid,https://www.googleapis.com/auth/userinfo.email,\\");
            eprintln!("  https://www.googleapis.com/auth/cloud-platform,\\");
            eprintln!("  https://www.googleapis.com/auth/drive.file");
            anyhow::bail!("not authenticated");
        }
        anyhow::bail!("gcloud token failed: {}", err);
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

pub fn upload_as_sheet(csv_path: &Path) -> Result<String> {
    let token = get_token()?;
    let name = csv_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let csv_bytes = std::fs::read(csv_path).context("read csv file")?;
    let boundary = "ocr_boundary_2024";

    let metadata = format!(
        r#"{{"name":"{}","mimeType":"application/vnd.google-apps.spreadsheet"}}"#,
        name
    );

    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
    body.extend_from_slice(metadata.as_bytes());
    body.extend_from_slice(format!("\r\n--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Type: text/csv\r\n\r\n");
    body.extend_from_slice(&csv_bytes);
    body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let content_type = format!("multipart/related; boundary={}", boundary);

    let resp: DriveFile = ureq::post(UPLOAD_URL)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", &content_type)
        .send(&body[..])
        .context("drive upload failed")?
        .body_mut()
        .read_json()
        .context("parse drive response")?;

    let file_id = resp.id.context("no file id in response")?;
    let url = format!("https://docs.google.com/spreadsheets/d/{}", file_id);
    Ok(url)
}
