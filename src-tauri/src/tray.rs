use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{
    menu::{Menu, MenuItem},
    Manager,
};

/// Create and build the system tray icon with a "Show" / "Quit" menu.
///
/// - Tray menu "Show": reveals and focuses the "main" webview window.
/// - Tray menu "Quit": terminates the process.
/// - Left-click on the tray icon: reveals and focuses the "main" window.
pub fn create_tray(app: &tauri::App) -> tauri::Result<()> {
    // Menu items
    let show_i = MenuItem::with_id(app, "show", "Show WhatsApp", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    // Load tray icon explicitly from bundled 32x32 PNG, decoded to 8-bit RGBA.
    // NOTE: do NOT use app.default_window_icon() — generated icons can be
    // 16-bit depth which tray-icon rejects ("expected 4096 got 8192").
    let rgba = image::load_from_memory(include_bytes!("../icons/32x32.png"))
        .map_err(|e| tauri::Error::Anyhow(e.into()))?
        .to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    let icon = tauri::image::Image::new_owned(rgba.into_raw(), w, h);

    // Build tray icon
    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("WhatsApp")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
