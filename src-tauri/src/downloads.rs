use tauri::webview::DownloadEvent;
use tauri::{Manager, Runtime, Webview};
use tauri_plugin_notification::NotificationExt;

/// Accept every download into the user's Downloads folder and toast on
/// finish, so there is always visible feedback (success *and* failure).
pub fn handle_download<R: Runtime>(webview: Webview<R>, event: DownloadEvent<'_>) -> bool {
    match event {
        DownloadEvent::Requested { url, destination } => {
            ensure_destination(destination);
            eprintln!(
                "[DOWNLOAD] requested scheme={} dest={:?}",
                url.scheme(),
                destination
            );
            true
        }
        DownloadEvent::Finished { path, success, .. } => {
            eprintln!(
                "[DOWNLOAD] finished success={} path={:?}",
                success, path
            );
            let app = webview.app_handle();
            if success {
                let body = match path.as_ref().and_then(|p| p.file_name()) {
                    Some(n) => format!(
                        "{} — saved to your Downloads folder.",
                        n.to_string_lossy()
                    ),
                    None => "Saved to your Downloads folder.".to_string(),
                };
                let _ = app
                    .notification()
                    .builder()
                    .title("Download complete")
                    .body(body)
                    .show();
            } else {
                let _ = app
                    .notification()
                    .builder()
                    .title("Download failed")
                    .body("The file could not be downloaded. Please try again.")
                    .show();
            }
            true
        }
        _ => true,
    }
}

/// Keep an absolute destination with a file name as-is; otherwise rebuild it
/// as `<Downloads>/<file name>` (mirrors whatRust's ensure logic).
fn ensure_destination(destination: &mut std::path::PathBuf) {
    if destination.is_absolute() && destination.file_name().is_some() {
        return;
    }
    let name = destination
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_else(|| "download".into());
    let mut dir = dirs::download_dir().unwrap_or_else(std::env::temp_dir);
    dir.push(name);
    *destination = dir;
}
