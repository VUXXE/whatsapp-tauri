use tauri::WebviewUrl;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
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
            .on_navigation(|url| {
                let url_str = url.as_str();

                // Intercept custom redirect trigger from iframe/JS
                if url_str.contains("open-external-link.invalid") {
                    for (k, v) in url.query_pairs() {
                        if k == "url" {
                            let _ = open::that(v.as_ref());
                            break;
                        }
                    }
                    return false;
                }

                let host = url.host_str().unwrap_or("");
                if !host.is_empty() && !host.ends_with("whatsapp.com") && !host.ends_with("whatsapp.net") {
                    let _ = open::that(url_str);
                    false
                } else {
                    true
                }
            })
            .initialization_script(r#"
                // 1. Auto-grant notification permissions
                if ('Notification' in window) {
                    Notification.requestPermission = function(cb) {
                        if (typeof cb === 'function') cb('granted');
                        return Promise.resolve('granted');
                    };
                    try {
                        Object.defineProperty(Notification, 'permission', { get: function() { return 'granted'; } });
                    } catch(e) {}
                }

                // Helper to trigger Rust navigation interceptor
                function openInBrowser(url) {
                    if (!url || typeof url !== 'string') return;
                    if (url.startsWith('http://') || url.startsWith('https://')) {
                        if (!url.includes('web.whatsapp.com') && !url.includes('whatsapp.net')) {
                            try {
                                var iframe = document.createElement('iframe');
                                iframe.style.display = 'none';
                                iframe.src = 'https://open-external-link.invalid/?url=' + encodeURIComponent(url);
                                (document.body || document.documentElement).appendChild(iframe);
                                setTimeout(function() {
                                    if (iframe && iframe.parentNode) {
                                        iframe.parentNode.removeChild(iframe);
                                    }
                                }, 1000);
                            } catch(err) {
                                console.error('Tauri Link Interceptor Error:', err);
                            }
                        }
                    }
                }

                // 2. Override window.open
                var realWindowOpen = window.open;
                window.open = function(url, target, features) {
                    if (url && typeof url === 'string' && !url.includes('web.whatsapp.com') && !url.includes('whatsapp.net')) {
                        openInBrowser(url);
                        return null;
                    }
                    return realWindowOpen.apply(this, arguments);
                };

                // 3. Intercept clicks on links globally
                document.addEventListener('click', function(e) {
                    var target = e.target;
                    while (target && target !== document) {
                        if (target.tagName === 'A' && target.href) {
                            var href = target.href;
                            if (!href.includes('web.whatsapp.com') && !href.includes('whatsapp.net')) {
                                if (href.startsWith('http://') || href.startsWith('https://')) {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    openInBrowser(href);
                                    return;
                                }
                            }
                        }
                        target = target.parentNode;
                    }
                }, true);
            "#)
            .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
