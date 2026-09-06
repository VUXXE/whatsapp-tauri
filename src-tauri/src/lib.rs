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

#[tauri::command]
fn get_clipboard_image() -> Result<String, String> {
    eprintln!("[RUST CLIPBOARD] get_clipboard_image called");
    let output = std::process::Command::new("wl-paste")
        .args(&["-t", "image/png"])
        .output()
        .or_else(|_| {
            std::process::Command::new("xclip")
                .args(&["-selection", "clipboard", "-t", "image/png", "-o"])
                .output()
        })
        .map_err(|e| e.to_string())?;

    if output.status.success() && !output.stdout.is_empty() {
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&output.stdout);
        eprintln!("[RUST CLIPBOARD] Found image in clipboard ({} bytes)", output.stdout.len());
        Ok(format!("data:image/png;base64,{}", b64))
    } else {
        eprintln!("[RUST CLIPBOARD] No image in clipboard");
        Err("No image in clipboard".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![get_clipboard_image])
        .setup(|app| {
            let nav_guard = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
            let nav_guard_clone = nav_guard.clone();

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

                async function tryPasteImageFromSystem() {
                    try {
                        if (!window.__TAURI__ || !window.__TAURI__.core) {
                            return false;
                        }
                        const dataUrl = await window.__TAURI__.core.invoke('get_clipboard_image');
                        if (!dataUrl || typeof dataUrl !== 'string' || !dataUrl.startsWith('data:image')) {
                            return false;
                        }

                        console.log('[CLIPBOARD] Image received from Rust, dispatching to WhatsApp...');
                        const res = await fetch(dataUrl);
                        const blob = await res.blob();
                        const file = new File([blob], 'screenshot.png', { type: 'image/png' });

                        const dt = new DataTransfer();
                        dt.items.add(file);

                        const composer = document.querySelector('[data-testid="conversation-compose-box-input"]') ||
                                         document.querySelector('[contenteditable="true"][role="textbox"]') ||
                                         document.querySelector('.copyable-text[contenteditable="true"]') ||
                                         document.activeElement;

                        const mainPanel = document.querySelector('#main') || document.body;

                        // 1. Dispatch paste event
                        if (composer) {
                            composer.focus();
                            const pasteEvt = new Event('paste', { bubbles: true, cancelable: true });
                            Object.defineProperty(pasteEvt, 'clipboardData', { get: () => dt });
                            composer.dispatchEvent(pasteEvt);
                        }

                        // 2. Dispatch drop event on #main
                        const dropEvt = new DragEvent('drop', { bubbles: true, cancelable: true });
                        Object.defineProperty(dropEvt, 'dataTransfer', { get: () => dt });
                        mainPanel.dispatchEvent(dropEvt);

                        console.log('[CLIPBOARD] Paste and Drop events dispatched successfully!');
                        return true;
                    } catch(err) {
                        return false;
                    }
                }

                // Intercept keyboard shortcut Ctrl+V / Cmd+V
                document.addEventListener('keydown', async function(e) {
                    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'v') {
                        const handled = await tryPasteImageFromSystem();
                        if (handled) {
                            e.preventDefault();
                            e.stopPropagation();
                        }
                    }
                }, true);

                // Intercept context menu paste
                document.addEventListener('paste', async function(e) {
                    const items = e.clipboardData ? e.clipboardData.items : null;
                    let hasNativeImage = false;
                    if (items) {
                        for (let i = 0; i < items.length; i++) {
                            if (items[i].type && items[i].type.startsWith('image/')) {
                                hasNativeImage = true;
                                break;
                            }
                        }
                    }
                    if (!hasNativeImage) {
                        const handled = await tryPasteImageFromSystem();
                        if (handled) {
                            e.preventDefault();
                            e.stopPropagation();
                        }
                    }
                }, true);

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
