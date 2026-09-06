use tauri::WebviewUrl;

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
            let mut nav_guard = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
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
            .on_navigation(move |url| {
                let url_str = url.as_str();

                // Guard against duplicate navigation events
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

                // --- CLIPBOARD IMAGE PASTE HANDLER (Robust) ---
                (function() {
                    var composerSelectors = [
                        '[data-testid="conversation-compose-box-input"]',
                        '[contenteditable="true"][data-tab="10"]',
                        '[contenteditable="true"][data-tab="9"]',
                        '.copyable-text[contenteditable="true"]',
                        'footer [contenteditable="true"]',
                        'div[contenteditable="true"][role="textbox"]'
                    ];

                    function findComposer() {
                        for (var i = 0; i < composerSelectors.length; i++) {
                            var el = document.querySelector(composerSelectors[i]);
                            if (el && (el.offsetWidth > 0 || el.offsetHeight > 0)) return el;
                        }
                        return null;
                    }

                    document.addEventListener('paste', function(e) {
                        var items = e.clipboardData && e.clipboardData.items;
                        if (!items || !items.length) return;

                        var hasImage = false;
                        var imageFile = null;
                        for (var i = 0; i < items.length; i++) {
                            if (items[i].type.indexOf('image') === 0) {
                                hasImage = true;
                                imageFile = items[i].getAsFile();
                                break;
                            }
                        }
                        if (!hasImage || !imageFile) return;

                        var composer = findComposer();
                        if (!composer) {
                            console.log('[CLIPBOARD] No composer found');
                            return;
                        }

                        console.log('[CLIPBOARD] Image detected, pasting to composer:', imageFile.type, imageFile.size);

                        // Focus the composer first
                        composer.focus();

                        // Create DataTransfer with the file
                        var dataTransfer = new DataTransfer();
                        dataTransfer.items.add(imageFile);

                        // Dispatch paste event with our custom clipboardData
                        var pasteEvent = new ClipboardEvent('paste', {
                            bubbles: true,
                            cancelable: true,
                            clipboardData: dataTransfer
                        });

                        // Also try input event as fallback
                        var inputEvent = new InputEvent('input', {
                            bubbles: true,
                            cancelable: true,
                            inputType: 'insertFromPaste',
                            dataTransfer: dataTransfer
                        });

                        // Small delay to ensure focus
                        setTimeout(function() {
                            composer.dispatchEvent(pasteEvent);
                            composer.dispatchEvent(inputEvent);
                            
                            // Trigger oninput for React/Vue
                            var onInputEvent = new Event('input', { bubbles: true });
                            composer.dispatchEvent(onInputEvent);
                        }, 50);
                    }, true);
                })();
            "#)
            .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}