mod downloads;
mod filebridge;
mod tray;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn extract_redirect_target(url_str: &str) -> Option<String> {
    let url = url::Url::parse(url_str).ok()?;

    if url.host_str() == Some("l.whatsapp.com") && url.path() == "/r" {
        if let Some(target) = url
            .query_pairs()
            .find(|(k, _)| k == "u")
            .map(|(_, v)| v.into_owned())
        {
            return Some(target);
        }
    }

    if url.host_str() == Some("web.whatsapp.com") && url.path() == "/redirect" {
        if let Some(target) = url
            .query_pairs()
            .find(|(k, _)| k == "u")
            .map(|(_, v)| v.into_owned())
        {
            return Some(target);
        }
    }

    None
}

// Injected Scripts: Powertools + File Bridge + External Link Router
const INJECTED_SCRIPT: &str = concat!(
    include_str!("powertools.js"),
    include_str!("filebridge.js"),
    r#"
    // External link routing
    document.addEventListener('click', function(e) {
        var target = e.target;
        while (target && target !== document) {
            if (target.tagName === 'A' && target.hasAttribute('download')) {
                return;
            }
            if (target.tagName === 'A' && target.href) {
                var href = target.href;
                if (href.includes('l.whatsapp.com') || href.includes('/redirect')) {
                    e.preventDefault();
                    e.stopPropagation();
                    window.location.href = href;
                    return;
                }
                if (!href.includes('web.whatsapp.com') && !href.includes('whatsapp.net')) {
                    if (href.startsWith('http://') || href.startsWith('https://')) {
                        e.preventDefault();
                        e.stopPropagation();
                        window.location.href = 'https://open-external-link.invalid/?url=' + encodeURIComponent(href);
                        return;
                    }
                }
            }
            target = target.parentNode;
        }
    }, true);
    "#
);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Setup System Tray
            tray::create_tray(app)?;

            let nav_guard = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
            let nav_guard_clone = nav_guard.clone();
            let app_handle = app.handle().clone();

            let builder = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://web.whatsapp.com".parse().unwrap()),
            )
            .title("WhatsApp")
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .inner_size(1100.0, 750.0)
            .min_inner_size(600.0, 500.0)
            .resizable(true)
            .initialization_script(INJECTED_SCRIPT)
            // Handle file drops ourselves so we can feed them to WhatsApp
            // through the file bridge (instead of a broken native drop).
            .disable_drag_drop_handler()
            .on_download(downloads::handle_download)
            .on_navigation(move |url| {
                let url_str = url.as_str();

                // JS debug channel: filebridge __report() lands here.
                if url_str.contains("tauri-log.invalid") {
                    if let Some(msg) = url
                        .query_pairs()
                        .find(|(k, _)| k == "m")
                        .map(|(_, v)| v.into_owned())
                    {
                        eprintln!("[JS] {}", msg);
                    }
                    return false;
                }

                // Clipboard image bridge trigger.
                if url_str.contains("tauri-clipboard.invalid") {
                    if let Some(win) = app_handle.get_webview_window("main") {
                        filebridge::inject_clipboard(&win);
                    }
                    return false;
                }

                let mut last_url = nav_guard_clone.lock().unwrap();
                if *last_url == url_str {
                    return false;
                }
                *last_url = url_str.to_string();
                drop(last_url);

                eprintln!("[RUST ON_NAVIGATION] URL: {}", url_str);

                if url_str.contains("open-external-link.invalid") {
                    if let Some(target_url) = url
                        .query_pairs()
                        .find(|(k, _)| k == "url")
                        .map(|(_, v)| v.into_owned())
                    {
                        eprintln!("[RUST OPENING BROWSER] Target: {}", target_url);
                        let _ = open::that(target_url);
                    }
                    return false;
                }

                if let Some(target_url) = extract_redirect_target(url_str) {
                    eprintln!("[RUST REDIRECT TARGET] Opening: {}", target_url);
                    let _ = open::that(target_url);
                    return false;
                }

                let host = url.host_str().unwrap_or("");
                let is_whatsapp_domain =
                    host.ends_with("whatsapp.com") || host.ends_with("whatsapp.net");

                let is_main_app = is_whatsapp_domain
                    && !url_str.contains("/redirect")
                    && host != "l.whatsapp.com";

                if is_main_app {
                    eprintln!("[RUST] Allowing main app navigation");
                    true
                } else if is_whatsapp_domain {
                    if let Some(target_url) = extract_redirect_target(url_str) {
                        eprintln!("[RUST] Opening extracted target: {}", target_url);
                        let _ = open::that(target_url);
                    }
                    false
                } else {
                    eprintln!("[RUST] Opening direct external URL: {}", url_str);
                    let _ = open::that(url_str);
                    false
                }
            });

            let window = builder.build()?;

            // Drag-and-drop: read dropped files in Rust and inject them
            // into the page through the file bridge.
            let drop_window = window.clone();
            window.on_webview_event(move |event| {
                if let tauri::WebviewEvent::DragDrop(e) = event {
                    if let tauri::DragDropEvent::Drop { paths, .. } = e {
                        eprintln!("[DROP] {} path(s) dropped", paths.len());
                        filebridge::inject_paths(&drop_window, paths);
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Minimize to Tray instead of closing the entire application!
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
