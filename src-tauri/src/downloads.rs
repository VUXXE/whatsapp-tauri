use tauri::webview::DownloadEvent;
use tauri::Webview;
use tauri::Runtime;

pub fn handle_download<R: Runtime>(
    _webview: Webview<R>,
    event: DownloadEvent<'_>,
) -> bool {
    match event {
        DownloadEvent::Requested { url, destination } => {
            eprintln!("[DOWNLOAD REQUESTED] {}", url);
            if let Some(download_dir) = dirs::download_dir() {
                let file_name = destination.file_name()
                    .map(|f| f.to_os_string())
                    .unwrap_or_else(|| {
                        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                        format!("WhatsApp_Download_{}.dat", timestamp).into()
                    });
                *destination = download_dir.join(file_name);
                eprintln!("[DOWNLOAD DESTINATION] {:?} Setting up native download", destination);
            }
            true // Allow download
        }
        DownloadEvent::Finished { url, path, success } => {
            eprintln!("[DOWNLOAD FINISHED] Url: {}, Path: {:?}, Success: {}", url, path, success);
            true
        }
        _ => true
    }
}
