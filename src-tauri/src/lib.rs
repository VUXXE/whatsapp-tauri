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

                // Global handler called from Rust when clipboard has an image
                window.__injectPastedImage = async function(b64) {
                    try {
                        console.log('[CLIPBOARD] Injected image received from Rust!');
                        const res = await fetch('data:image/png;base64,' + b64);
                        const blob = await res.blob();
                        const file = new File([blob], 'image.png', { type: 'image/png' });

                        const dt = new DataTransfer();
                        dt.items.add(file);

                        // Method 1: Find file input directly or trigger attachment button
                        let fileInput = document.querySelector('input[type="file"][accept*="image"]') ||
                                        document.querySelector('input[type="file"]');

                        if (!fileInput) {
                            const attachBtn = document.querySelector('[data-testid="clip"]') ||
                                              document.querySelector('[data-icon="attach-menu-plus"]') ||
                                              document.querySelector('[data-icon="plus"]') ||
                                              document.querySelector('button[aria-label="Attach"]') ||
                                              document.querySelector('button[title="Attach"]') ||
                                              document.querySelector('span[data-icon="plus"]') ||
                                              document.querySelector('span[data-icon="attach-menu-plus"]');
                            if (attachBtn) {
                                attachBtn.click();
                                await new Promise(r => setTimeout(r, 80));
                                fileInput = document.querySelector('input[type="file"][accept*="image"]') ||
                                            document.querySelector('input[type="file"]');
                            }
                        }

                        if (fileInput) {
                            fileInput.files = dt.files;
                            fileInput.dispatchEvent(new Event('change', { bubbles: true }));
                            console.log('[CLIPBOARD] Fired change event on fileInput successfully!');
                            return;
                        }

                        // Method 2: Dispatch synthetic paste and drop on active elements
                        const targets = [
                            document.activeElement,
                            document.querySelector('[data-testid="conversation-compose-box-input"]'),
                            document.querySelector('div[contenteditable="true"]'),
                            document.querySelector('#main'),
                            document.body
                        ];

                        for (const target of targets) {
                            if (!target) continue;
                            try {
                                const pasteEvt = new Event('paste', { bubbles: true, cancelable: true });
                                Object.defineProperty(pasteEvt, 'clipboardData', { get: () => dt });
                                target.dispatchEvent(pasteEvt);

                                const dropEvt = new DragEvent('drop', { bubbles: true, cancelable: true });
                                Object.defineProperty(dropEvt, 'dataTransfer', { get: () => dt });
                                target.dispatchEvent(dropEvt);
                            } catch(e) {}
                        }
                    } catch(err) {
                        console.error('[CLIPBOARD] Injection error:', err);
                    }
                };

                // Trigger Rust clipboard check on Ctrl+V / Cmd+V
                document.addEventListener('keydown', function(e) {
                    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'v') {
                        window.location.href = 'https://get-clipboard-image.invalid/?t=' + Date.now();
                    }
                }, false);

                // External link handling
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

                document.addEventListener('click', function(e) {
                    var target = e.target;
                    while (target && target !== document) {
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
                                    triggerExternalOpen(href);
                                    return;
                                }
                            }
                        }
                        target = target.parentNode;
                    }
                }, true);
            "#)
            .on_navigation(move |url| {
                let url_str = url.as_str();

                // Clipboard image bridge: check Linux system clipboard
                if url_str.contains("get-clipboard-image.invalid") {
                    eprintln!("[RUST CLIPBOARD] Intercepted navigation trigger to read system clipboard");

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
                            eprintln!("[RUST CLIPBOARD] Found {} bytes of image in system clipboard! Injecting into webview...", out.stdout.len());

                            let js = format!(
                                "if (window.__injectPastedImage) {{ window.__injectPastedImage('{}'); }}",
                                b64
                            );
                            if let Some(win) = app_handle.get_webview_window("main") {
                                let _ = win.eval(&js);
                            }
                        } else {
                            eprintln!("[RUST CLIPBOARD] Clipboard has text or no image data");
                        }
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
            })
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
