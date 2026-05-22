use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const RELEASE_BASE: &str = "https://github.com/lanmower/ocr/releases/download/latest";
const UPSTREAM_LLAMA: &str = "https://github.com/ggml-org/llama.cpp/releases/download/b8785";
const MODEL_URL: &str =
    "https://huggingface.co/bartowski/google_gemma-4-E2B-it-GGUF/resolve/main/google_gemma-4-E2B-it-Q4_K_M.gguf";
const MMPROJ_NAME: &str = "mmproj-google_gemma-4-E2B-it-f16.gguf";
const MMPROJ_UPSTREAM: &str =
    "https://huggingface.co/bartowski/google_gemma-4-E2B-it-GGUF/resolve/main/mmproj-google_gemma-4-E2B-it-f16.gguf";

static WEB_INDEX: &[u8] = include_bytes!("../web/index.html");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Cpu,
    Cuda,
    Vulkan,
}

impl Variant {
    fn as_str(self) -> &'static str {
        match self {
            Variant::Cpu => "cpu",
            Variant::Cuda => "cuda",
            Variant::Vulkan => "vulkan",
        }
    }
    fn ngl(self) -> &'static str {
        if matches!(self, Variant::Cpu) { "0" } else { "99" }
    }
    fn llama_zip_name(self) -> &'static str {
        match self {
            Variant::Cpu => "llama-b8785-bin-win-cpu-x64.zip",
            Variant::Cuda => "llama-b8785-bin-win-cuda-12.4-x64.zip",
            Variant::Vulkan => "llama-b8785-bin-win-vulkan-x64.zip",
        }
    }
}

pub fn detect_variant() -> Variant {
    let out = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name",
        ])
        .output();
    let names = match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => String::new(),
    };
    let low = names.to_lowercase();
    if low.contains("nvidia") {
        Variant::Cuda
    } else if low.contains("amd") || low.contains("radeon") || low.contains("intel arc") {
        Variant::Vulkan
    } else {
        Variant::Cpu
    }
}

fn exe_dir() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("current_exe")?;
    Ok(exe.parent().context("exe has no parent")?.to_path_buf())
}

pub struct Paths {
    pub root: PathBuf,
    pub llama_dir: PathBuf,
    pub server_exe: PathBuf,
    pub model: PathBuf,
    pub mmproj: PathBuf,
    pub web_dir: PathBuf,
    pub variant_file: PathBuf,
}

pub fn paths() -> Result<Paths> {
    let root = exe_dir()?.join(".ocr-runtime");
    let llama_dir = root.join("llama");
    Ok(Paths {
        server_exe: llama_dir.join("llama-server.exe"),
        model: root.join("model.gguf"),
        mmproj: root.join(MMPROJ_NAME),
        web_dir: root.join("web"),
        variant_file: root.join("variant.txt"),
        llama_dir,
        root,
    })
}

fn download(url: &str, dest: &Path, label: &str) -> Result<()> {
    eprintln!("[bootstrap] downloading {label}: {url}");
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).ok();
    }
    let tmp = dest.with_extension("partial");
    let mut resp = ureq::get(url).call().with_context(|| format!("GET {url}"))?;
    let mut reader = resp.body_mut().as_reader();
    let mut file = fs::File::create(&tmp).with_context(|| format!("create {}", tmp.display()))?;
    let mut buf = vec![0u8; 1 << 20];
    let mut total: u64 = 0;
    let mut next_tick: u64 = 0;
    loop {
        let n = reader.read(&mut buf).context("read body")?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).context("write file")?;
        total += n as u64;
        if total >= next_tick {
            eprintln!("[bootstrap]   {label}: {} MB", total / (1024 * 1024));
            next_tick = total + (32 * 1024 * 1024);
        }
    }
    drop(file);
    fs::rename(&tmp, dest).context("rename partial")?;
    eprintln!("[bootstrap] done {label}: {} MB", total / (1024 * 1024));
    Ok(())
}

fn try_download(urls: &[&str], dest: &Path, label: &str) -> Result<()> {
    let mut last: Option<anyhow::Error> = None;
    for url in urls {
        match download(url, dest, label) {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("[bootstrap]   failed {url}: {e}");
                last = Some(e);
            }
        }
    }
    Err(last.unwrap_or_else(|| anyhow!("no urls")))
}

fn extract_zip(zip_path: &Path, dest: &Path) -> Result<()> {
    eprintln!("[bootstrap] extracting {} -> {}", zip_path.display(), dest.display());
    fs::create_dir_all(dest).ok();
    let file = fs::File::open(zip_path).context("open zip")?;
    let mut zip = zip::ZipArchive::new(file).context("zip archive")?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).context("zip entry")?;
        let rel = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => continue,
        };
        let out_path = dest.join(&rel);
        if entry.is_dir() {
            fs::create_dir_all(&out_path).ok();
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let mut out = fs::File::create(&out_path).with_context(|| format!("create {}", out_path.display()))?;
        std::io::copy(&mut entry, &mut out).context("copy zip entry")?;
    }
    Ok(())
}

fn flatten_llama_dir(dir: &Path) -> Result<()> {
    // Some llama zips nest under a build/bin/ or similar prefix; bring llama-server.exe to dir root.
    if dir.join("llama-server.exe").exists() {
        return Ok(());
    }
    fn find_server(d: &Path) -> Option<PathBuf> {
        for entry in fs::read_dir(d).ok()?.flatten() {
            let p = entry.path();
            if p.is_file() && p.file_name().map_or(false, |n| n == "llama-server.exe") {
                return Some(p);
            }
            if p.is_dir() {
                if let Some(found) = find_server(&p) {
                    return Some(found);
                }
            }
        }
        None
    }
    let server = find_server(dir).ok_or_else(|| anyhow!("llama-server.exe not in extracted zip"))?;
    let source_dir = server.parent().ok_or_else(|| anyhow!("server has no parent"))?.to_path_buf();
    if source_dir == dir {
        return Ok(());
    }
    for entry in fs::read_dir(&source_dir)?.flatten() {
        let p = entry.path();
        let target = dir.join(p.file_name().unwrap());
        if target.exists() {
            continue;
        }
        fs::rename(&p, &target).ok();
    }
    Ok(())
}

pub fn ensure_runtime(variant: Variant) -> Result<Paths> {
    let p = paths()?;
    fs::create_dir_all(&p.root).ok();
    fs::create_dir_all(&p.llama_dir).ok();
    fs::create_dir_all(&p.web_dir).ok();

    // Web assets (always overwrite — embedded copy is canonical)
    fs::write(p.web_dir.join("index.html"), WEB_INDEX).context("write web/index.html")?;

    // Variant marker
    fs::write(&p.variant_file, variant.as_str()).context("write variant.txt")?;

    // llama-server
    if !p.server_exe.exists() {
        let zip_name = variant.llama_zip_name();
        let zip_path = p.llama_dir.join(zip_name);
        let urls = [
            format!("{RELEASE_BASE}/{zip_name}"),
            format!("{UPSTREAM_LLAMA}/{zip_name}"),
        ];
        let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
        try_download(&url_refs, &zip_path, &format!("llama-{}", variant.as_str()))?;
        extract_zip(&zip_path, &p.llama_dir)?;
        fs::remove_file(&zip_path).ok();

        if variant == Variant::Cuda {
            let cudart = "cudart-llama-bin-win-cuda-12.4-x64.zip";
            let cudart_path = p.llama_dir.join(cudart);
            let urls = [
                format!("{RELEASE_BASE}/{cudart}"),
                format!("{UPSTREAM_LLAMA}/{cudart}"),
            ];
            let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
            try_download(&url_refs, &cudart_path, "cudart")?;
            extract_zip(&cudart_path, &p.llama_dir)?;
            fs::remove_file(&cudart_path).ok();
        }

        flatten_llama_dir(&p.llama_dir)?;
    }

    if !p.server_exe.exists() {
        return Err(anyhow!("llama-server.exe still missing after extract"));
    }

    // mmproj
    if !p.mmproj.exists() {
        let urls = [
            format!("{RELEASE_BASE}/{MMPROJ_NAME}"),
            MMPROJ_UPSTREAM.to_string(),
        ];
        let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
        try_download(&url_refs, &p.mmproj, "mmproj")?;
    }

    // Model — only HuggingFace; too big for our release
    if !p.model.exists() {
        download(MODEL_URL, &p.model, "model")?;
    }

    Ok(p)
}

pub fn spawn_server(p: &Paths, variant: Variant) -> Result<Child> {
    eprintln!("[bootstrap] starting llama-server ({}) -ngl {}", variant.as_str(), variant.ngl());
    let child = Command::new(&p.server_exe)
        .arg("-m").arg(&p.model)
        .arg("--mmproj").arg(&p.mmproj)
        .arg("--host").arg("127.0.0.1")
        .arg("--port").arg("8080")
        .arg("-ngl").arg(variant.ngl())
        .arg("-rea").arg("off")
        .arg("--path").arg(&p.web_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("spawn {}", p.server_exe.display()))?;
    Ok(child)
}

pub fn wait_healthy(timeout: Duration) -> Result<()> {
    let start = std::time::Instant::now();
    eprintln!("[bootstrap] waiting for llama-server /health ...");
    while start.elapsed() < timeout {
        if let Ok(resp) = ureq::get("http://127.0.0.1:8080/health").call() {
            if resp.status().is_success() {
                eprintln!("[bootstrap] llama-server is ready");
                return Ok(());
            }
        }
        std::thread::sleep(Duration::from_millis(1000));
    }
    Err(anyhow!("llama-server did not become healthy within {:?}", timeout))
}

pub fn ensure_running() -> Result<Child> {
    let variant = detect_variant();
    eprintln!("[bootstrap] detected GPU variant: {}", variant.as_str());
    let p = ensure_runtime(variant)?;
    let child = spawn_server(&p, variant)?;
    wait_healthy(Duration::from_secs(300))?;
    Ok(child)
}
