//! File bridge: feeds OS-level files (clipboard images, drag-and-dropped
//! files) into the WhatsApp Web page.
//!
//! Transport design adapted from whatRust (MIT, github.com/karem505/whatRust):
//! files stream as begin/chunk/end/commit messages keyed by a drop id.
//! Stanzas are multiples of 3 raw bytes so each base64-encodes standalone,
//! and several stanzas ride in one eval to stay far below IPC ceilings.

use base64::Engine as _;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::WebviewWindow;

static DROP_SEQ: AtomicU64 = AtomicU64::new(1);

/// Caps mirror whatRust's (the page must hold decoded bytes to build Files).
const MAX_FILE_BYTES: u64 = 500 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 500 * 1024 * 1024;
const MAX_FILES: usize = 200;
/// Raw bytes per base64 stanza. MUST stay a multiple of 3 (except the final
/// partial stanza) so every stanza decodes standalone without padding issues.
const CHUNK_BYTES: usize = 8 * 1024 * 1024 - ((8 * 1024 * 1024) % 3);
const CHUNKS_PER_EVAL: usize = 2;

pub fn next_drop_id() -> u64 {
    DROP_SEQ.fetch_add(1, Ordering::Relaxed)
}

fn mime_for(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" | "jpe" | "jfif" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "heic" => "image/heic",
        "heif" | "hif" => "image/heif",
        "mp4" | "m4v" => "video/mp4",
        "mov" | "qt" => "video/quicktime",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "3gp" | "3gpp" => "video/3gpp",
        "avi" => "video/x-msvideo",
        "mp3" => "audio/mpeg",
        "ogg" | "oga" => "audio/ogg",
        "opus" => "audio/opus",
        "wav" => "audio/wav",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

fn msg_begin(id: u64, idx: usize, name: &str, mime: &str, len: u64) -> String {
    let name_json = serde_json::to_string(name).unwrap_or_else(|_| "\"file\"".into());
    format!(
        "window.__dropFeed&&window.__dropFeed({{op:\"begin\",drop:{id},file:{idx},name:{name_json},type:\"{mime}\",size:{len}}});"
    )
}

fn msg_end(id: u64, idx: usize) -> String {
    format!("window.__dropFeed&&window.__dropFeed({{op:\"end\",drop:{id},file:{idx}}});")
}

fn msg_abort(id: u64, idx: usize) -> String {
    format!("window.__dropFeed&&window.__dropFeed({{op:\"abort\",drop:{id},file:{idx}}});")
}

fn msg_commit(id: u64, files: usize) -> String {
    format!(
        "window.__dropFeed?window.__dropFeed({{op:\"commit\",drop:{id},files:{files}}}):\"NOHANDLER\""
    )
}

fn emit_chunks(
    w: &WebviewWindow,
    id: u64,
    idx: usize,
    stanzas: Vec<String>,
) -> Result<(), String> {
    let mut js = format!(
        "window.__dropFeed&&window.__dropFeed({{op:\"chunk\",drop:{id},file:{idx},parts:["
    );
    for (i, s) in stanzas.iter().enumerate() {
        if i > 0 {
            js.push(',');
        }
        js.push('"');
        js.push_str(s);
        js.push('"');
    }
    js.push_str("]});");
    w.eval(&js).map_err(|e| e.to_string())
}

/// Stream `len` bytes from `r` as chunk batches. Reads from a worker thread
/// so the UI thread never blocks on disk I/O or base64.
fn stream_reader(
    w: &WebviewWindow,
    id: u64,
    idx: usize,
    r: &mut impl Read,
    len: u64,
) -> Result<(), String> {
    let mut buf = [0u8; 48 * 1024];
    let mut chunk: Vec<u8> = Vec::new();
    let mut batch: Vec<String> = Vec::with_capacity(CHUNKS_PER_EVAL);
    let mut total: u64 = 0;
    loop {
        let want = std::cmp::min(buf.len() as u64, len.saturating_sub(total)) as usize;
        if want == 0 {
            break;
        }
        let n = r.read(&mut buf[..want]).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("file shrank while reading".into());
        }
        total += n as u64;
        chunk.extend_from_slice(&buf[..n]);
        while chunk.len() > CHUNK_BYTES {
            let stanza =
                base64::engine::general_purpose::STANDARD.encode(&chunk[..CHUNK_BYTES]);
            batch.push(stanza);
            chunk.drain(..CHUNK_BYTES);
            if batch.len() == CHUNKS_PER_EVAL {
                emit_chunks(w, id, idx, std::mem::take(&mut batch))?;
            }
        }
    }
    // Probe for growth past the planned size.
    let mut probe = [0u8; 1];
    if r.read(&mut probe).map_err(|e| e.to_string())? != 0 {
        return Err("file grew while reading".into());
    }
    if !chunk.is_empty() {
        let stanza = base64::engine::general_purpose::STANDARD.encode(&chunk);
        batch.push(stanza);
    } else if len == 0 {
        batch.push(String::new());
    }
    if !batch.is_empty() {
        emit_chunks(w, id, idx, batch)?;
    }
    Ok(())
}

struct Planned {
    name: String,
    mime: &'static str,
    len: u64,
    // Either a path (streamed from disk) or bytes (clipboard, in memory).
    source: Source,
}

enum Source {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

fn stream_planned(w: &WebviewWindow, id: u64, idx: usize, f: &Planned) -> Result<(), String> {
    w.eval(&msg_begin(id, idx, &f.name, f.mime, f.len))
        .map_err(|e| e.to_string())?;
    let result = match &f.source {
        Source::Path(p) => {
            let mut file = std::fs::File::open(p).map_err(|e| e.to_string())?;
            let actual = file.metadata().map_err(|e| e.to_string())?.len();
            if actual != f.len {
                Err("file changed while reading".to_string())
            } else {
                stream_reader(w, id, idx, &mut file, f.len)
            }
        }
        Source::Bytes(b) => {
            let mut cursor = std::io::Cursor::new(b);
            stream_reader(w, id, idx, &mut cursor, f.len)
        }
    };
    match result {
        Ok(()) => w.eval(&msg_end(id, idx)).map_err(|e| e.to_string()),
        Err(e) => {
            let _ = w.eval(&msg_abort(id, idx));
            Err(e)
        }
    }
}

fn commit(w: &WebviewWindow, id: u64, files: usize) {
    let js = msg_commit(id, files);
    let res = w.eval_with_callback(js, move |ack| {
        eprintln!("[DROP] drop #{} commit ack: {}", id, ack.trim());
    });
    if let Err(e) = res {
        eprintln!("[DROP] drop #{} commit eval failed: {}", id, e);
    }
}

fn run_stream(w: WebviewWindow, id: u64, planned: Vec<Planned>) {
    let mut streamed = 0usize;
    for (idx, f) in planned.iter().enumerate() {
        match stream_planned(&w, id, idx, f) {
            Ok(()) => {
                eprintln!("[DROP] drop #{} file {}: streamed {} bytes ({})", id, idx, f.len, f.mime);
                streamed += 1;
            }
            Err(e) => eprintln!("[DROP] drop #{} file {}: skipped: {}", id, idx, e),
        }
    }
    if streamed > 0 {
        commit(&w, id, streamed);
    } else {
        eprintln!("[DROP] drop #{}: nothing to commit", id);
    }
}

fn plan_paths(paths: &[PathBuf]) -> Vec<Planned> {
    let mut planned = Vec::new();
    let mut budget = MAX_TOTAL_BYTES;
    for p in paths {
        if planned.len() >= MAX_FILES {
            eprintln!("[DROP] skip {:?}: over file-count cap", p);
            continue;
        }
        let meta = match std::fs::metadata(p) {
            Ok(m) => m,
            Err(_) => {
                eprintln!("[DROP] skip {:?}: unreadable", p);
                continue;
            }
        };
        if !meta.is_file() {
            eprintln!("[DROP] skip {:?}: not a regular file", p);
            continue;
        }
        let len = meta.len();
        if len > MAX_FILE_BYTES {
            eprintln!("[DROP] skip {:?}: over size cap", p);
            continue;
        }
        if len > budget {
            eprintln!("[DROP] skip {:?}: over batch budget", p);
            continue;
        }
        budget -= len;
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "file".into());
        let mime = mime_for(&name);
        planned.push(Planned { name, mime, len, source: Source::Path(p.clone()) });
    }
    planned
}

/// Inject dropped files (by path) into the page. Runs on a worker thread.
pub fn inject_paths(window: &WebviewWindow, paths: &[PathBuf]) {
    let planned = plan_paths(paths);
    if planned.is_empty() {
        eprintln!("[DROP] ignored: no injectable files");
        return;
    }
    let id = next_drop_id();
    eprintln!("[DROP] drop #{}: {} file(s) accepted", id, planned.len());
    let w = window.clone();
    std::thread::spawn(move || run_stream(w, id, planned));
}

/// Read an image from the Linux system clipboard and stream it into the page.
/// Silent no-op when the clipboard holds no image, so text paste keeps
/// working through the native path.
pub fn inject_clipboard(window: &WebviewWindow) {
    let out = std::process::Command::new("wl-paste")
        .args(["-t", "image/png"])
        .output()
        .or_else(|_| {
            std::process::Command::new("xclip")
                .args(["-selection", "clipboard", "-t", "image/png", "-o"])
                .output()
        });
    let bytes = match out {
        Ok(o) if o.status.success() && !o.stdout.is_empty() => o.stdout,
        _ => {
            eprintln!("[DROP] clipboard: no image data, ignoring");
            return;
        }
    };
    let id = next_drop_id();
    eprintln!("[DROP] clipboard image ({} bytes) as drop #{}", bytes.len(), id);
    let planned = vec![Planned {
        name: "pasted-image.png".to_string(),
        mime: "image/png",
        len: bytes.len() as u64,
        source: Source::Bytes(bytes),
    }];
    let w = window.clone();
    std::thread::spawn(move || run_stream(w, id, planned));
}

/// Resolve a dropped path that may be a directory: expand one level so
/// dropping a folder attaches the files inside it (non-recursive).
pub fn expand_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for p in paths {
        if p.is_dir() {
            match std::fs::read_dir(p) {
                Ok(entries) => {
                    for e in entries.flatten() {
                        let ep = e.path();
                        if ep.is_file() {
                            out.push(ep);
                        }
                    }
                }
                Err(_) => eprintln!("[DROP] skip {:?}: unreadable directory", p),
            }
        } else {
            out.push(p.clone());
        }
    }
    out
}
