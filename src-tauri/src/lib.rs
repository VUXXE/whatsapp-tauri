use tauri::WebviewUrl;

#[tauri::command]
fn open_external(url: String) {
    if !url.is_empty() {
        let _ = open::that(url);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![open_external])
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
                let host = url.host_str().unwrap_or("");
                if !host.is_empty() && !host.ends_with("whatsapp.com") && !host.ends_with("whatsapp.net") {
                    let _ = open::that(url.as_str());
                    false
                } else {
                    true
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

                function openInBrowser(url) {
                    if (!url) return;
                    if (url.startsWith('http://') || url.startsWith('https://')) {
                        if (!url.includes('web.whatsapp.com') && !url.includes('whatsapp.net')) {
                            if (window.__TAURI__ && window.__TAURI__.core) {
                                window.__TAURI__.core.invoke('open_external', { url: url });
                            }
                        }
                    }
                }

                var realWindowOpen = window.open;
                window.open = function(url, target, features) {
                    if (url && typeof url === 'string' && !url.includes('web.whatsapp.com') && !url.includes('whatsapp.net')) {
                        openInBrowser(url);
                        return null;
                    }
                    return realWindowOpen.apply(this, arguments);
                };

                document.addEventListener('click', function(e) {
                    var anchor = e.target && e.target.closest ? e.target.closest('a') : null;
                    if (anchor && anchor.href) {
                        var href = anchor.href;
                        if (!href.includes('web.whatsapp.com') && !href.includes('whatsapp.net')) {
                            if (href.startsWith('http://') || href.startsWith('https://')) {
                                e.preventDefault();
                                e.stopPropagation();
                                openInBrowser(href);
                            }
                        }
                    }
                }, true);
            "#)
            .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
