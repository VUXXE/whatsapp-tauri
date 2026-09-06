//! File bridge: feeds OS-level files (clipboard images, drag-and-dropped
//! files) into the WhatsApp Web page, working around WebKitGTK sandbox
//! limits that hide binary clipboard/drop data from page JavaScript.

use base64::Engine;
use std::path::{Path, PathBuf};
use tauri::WebviewWindow;

const MAX_BYTES: u64 = 100 * 1024 * 1024;

fn mime_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        Some("heic") | Some("heif") => "image/heic",
        Some("mp4") => "video/mp4",
        Some("mov") => "video/quicktime",
        Some("webm") => "video/webm",
        Some("mkv") => "video/x-matroska",
        Some("mp3") => "audio/mpeg",
        Some("ogg") | Some("opus") => "audio/ogg",
        Some("wav") => "audio/wav",
        Some("m4a") => "audio/mp4",
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain",
        Some("zip") => "application/zip",
        _ => "application/octet-stream",
    }
}

fn entry_for(name: String, mime: &'static str, data: Vec<u8>) -> Option<serde_json::Value> {
    if data.len() as u64 > MAX_BYTES {
        eprintln!("[FILEBRIDGE] Skipping oversized payload: {} ({} bytes)", name, data.len());
        return None;
    }
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    eprintln!("[FILEBRIDGE] Prepared {} ({} bytes, {})", name, data.len(), mime);
    Some(serde_json::json!({ "name": name, "mime": mime, "b64": b64 }))
}

fn eval_inject(window: &WebviewWindow, entries: Vec<serde_json::Value>) {
    if entries.is_empty() {
        return;
    }
    let payload = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());
    let js = format!("if(window.__injectFiles){{window.__injectFiles({});}}", payload);
    if let Err(e) = window.eval(&js) {
        eprintln!("[FILEBRIDGE] eval failed: {}", e);
    }
}

/// Inject dropped files (by path) into the page.
pub fn inject_paths(window: &WebviewWindow, paths: &[PathBuf]) {
    let entries: Vec<serde_json::Value> = paths
        .iter()
        .filter_map(|p| {
            let data = std::fs::read(p).ok()?;
            let name = p.file_name()?.to_string_lossy().to_string();
            entry_for(name, mime_for(p), data)
        })
        .collect();
    if entries.is_empty() {
        eprintln!("[FILEBRIDGE] Drop ignored: no readable files");
    }
    eval_inject(window, entries);
}

/// Read an image from the Linux system clipboard and inject it into the page.
/// Does nothing (by design) when the clipboard holds no image, so text
/// paste keeps working through the native path.
pub fn inject_clipboard(window: &WebviewWindow) {
    let out = std::process::Command::new("wl-paste")
        .args(["-t", "image/png"])
        .output()
        .or_else(|_| {
            std::process::Command::new("xclip")
                .args(["-selection", "clipboard", "-t", "image/png", "-o"])
                .output()
        });
    match out {
        Ok(o) if o.status.success() && !o.stdout.is_empty() => {
            eprintln!(
                "[FILEBRIDGE] Clipboard image: {} bytes, injecting",
                o.stdout.len()
            );
            if let Some(entry) =
                entry_for("pasted-image.png".to_string(), "image/png", o.stdout)
            {
                eval_inject(window, vec![entry]);
            }
        }
        _ => eprintln!("[FILEBRIDGE] No image in system clipboard, ignoring"),
    }
}
