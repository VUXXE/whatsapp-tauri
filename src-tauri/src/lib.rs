use tauri::{Manager, WebviewUrl};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let nav_guard = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
            let nav_guard_clone = nav_guard.clone();
            let app_handle = app.handle().clone();

            let _window = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://web.whatsapp.com".parse().unwrap()),
            )
            .title("WhatsApp")
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .inner_size(1100.0, 750.0)
            .min_inner_size(600.0, 500.0)
            .resizable(true)
            .initialization_script(r#"
                if ('Notification' in window) {
                    Notification.requestPermission = function(cb) {
                        if (typeof cb === 'function') cb('granted');
                        return Promise.resolve('granted');
                    };
                    try {
                        Object.defineProperty(Notification, 'permission', { get: function() { return 'granted'; } });
                    } catch(e) {}
                }

                // Global handler to convert Base64 image from Rust system clipboard into native JS File & paste to WhatsApp
                window.__pasteBase64Image = function(dataUrl) {
                    try {
                        var arr = dataUrl.split(',');
                        var mime = arr[0].match(/:(.*?);/)[1];
                        var bstr = atob(arr[1]);
                        var n = bstr.length;
                        var u8arr = new Uint8Array(n);
                        while (n--) {
                            u8arr[n] = bstr.charCodeAt(n);
                        }
                        var file = new File([u8arr], "pasted_image.png", { type: mime });

                        var composerSelectors = [
                            '[data-testid="conversation-compose-box-input"]',
                            '[contenteditable="true"][data-tab="10"]',
                            '[contenteditable="true"][data-tab="9"]',
                            '.copyable-text[contenteditable="true"]',
                            'footer [contenteditable="true"]',
                            'div[contenteditable="true"][role="textbox"]'
                        ];

                        var composer = null;
                        for (var i = 0; i < composerSelectors.length; i++) {
                            var el = document.querySelector(composerSelectors[i]);
                            if (el && (el.offsetWidth > 0 || el.offsetHeight > 0)) {
                                composer = el;
                                break;
                            }
                        }

                        if (!composer) {
                            composer = document.querySelector('div[contenteditable="true"]') || document.querySelector('#main');
                        }

                        if (!composer) {
                            console.log('[CLIPBOARD] No composer element found');
                            return;
                        }

                        composer.focus();

                        var dataTransfer = new DataTransfer();
                        dataTransfer.items.add(file);

                        var pasteEvent = new ClipboardEvent('paste', {
                            bubbles: true,
                            cancelable: true,
                            clipboardData: dataTransfer
                        });

                        composer.dispatchEvent(pasteEvent);
                        console.log('[CLIPBOARD] Dispatched image from system clipboard to WhatsApp Web!');
                    } catch(err) {
                        console.error('[CLIPBOARD] Error processing base64 image:', err);
                    }
                };

                function triggerExternalOpen(url) {
                    if (!url || typeof url !== 'string') return;
                    if (url.startsWith('http://') || url.startsWith('https://')) {
                        if (!url.includes('web.whatsapp.com') && !url.includes('whatsapp.net')) {
                            window.location.href = 'https://open-external-link.invalid/?url=' + encodeURIComponent(url);
                        }
                    }
                }

                var realWindowOpen = window.open;
                window.open = function(url, target, features) {
                    if (url && typeof url === 'string' && !url.includes('web.whatsapp.com') && !url.includes('whatsapp.net')) {
                        triggerExternalOpen(url);
                        return null;
                    }
                    return realWindowOpen.apply(this, arguments);
                };

                // Listen for Ctrl+V / Cmd+V to trigger Rust system clipboard read
                document.addEventListener('keydown', function(e) {
                    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'v') {
                        var iframe = document.createElement('iframe');
                        iframe.style.display = 'none';
                        iframe.src = 'https://get-clipboard-image.invalid/?t=' + Date.now();
                        (document.body || document.documentElement).appendChild(iframe);
                        setTimeout(function() {
                            if (iframe && iframe.parentNode) {
                                iframe.parentNode.removeChild(iframe);
                            }
                        }, 1000);
                    }
                }, true);
            "#)
            .on_navigation(move |url| {
                let url_str = url.as_str();

                let mut last_url = nav_guard_clone.lock().unwrap();
                if *last_url == url_str {
                    return false;
                }
                *last_url = url_str.to_string();
                drop(last_url);

                eprintln!("[RUST ON_NAVIGATION] URL: {}", url_str);

                if url_str.contains("get-clipboard-image.invalid") {
                    eprintln!("[RUST CLIPBOARD] Reading Linux system clipboard image...");

                    let output = std::process::Command::new("wl-paste")
                        .args(&["-t", "image/png"])
                        .output()
                        .or_else(|_| {
                            std::process::Command::new("xclip")
                                .args(&["-selection", "clipboard", "-t", "image/png", "-o"])
                                .output()
                        });

                    if let Ok(out) = output {
                        if out.status.success() && !out.stdout.is_empty() {
                            use base64::Engine;
                            let b64 = base64::engine::general_purpose::STANDARD.encode(&out.stdout);
                            eprintln!("[RUST CLIPBOARD] Successfully read {} bytes of PNG from system clipboard!", out.stdout.len());

                            let js = format!(
                                "if (window.__pasteBase64Image) {{ window.__pasteBase64Image('data:image/png;base64,{}'); }}",
                                b64
                            );
                            if let Some(win) = app_handle.get_webview_window("main") {
                                let _ = win.eval(&js);
                            }
                        } else {
                            eprintln!("[RUST CLIPBOARD] Clipboard does not contain image/png data");
                        }
                    } else {
                        eprintln!("[RUST CLIPBOARD] Failed to run wl-paste or xclip");
                    }
                    return false;
                }

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
            })
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
