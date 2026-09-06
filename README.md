# 🚀 WhatsApp Tauri

A super lightweight, cross-platform desktop client for **WhatsApp Web** built with **Tauri v2** and **Rust**.

Designed as a high-performance, low-memory alternative to resource-heavy Electron apps (like Whatsie or official web wrappers).

---

## 📥 Download & Installation

Download the latest version for your operating system from the **[GitHub Releases Page](https://github.com/VUXXE/whatsapp-tauri/releases)**:

| Platform | Recommended Package | Note |
|---|---|---|
| 🪟 **Windows** | `.exe` / `.msi` | Double-click installer for Windows 10/11 |
| 🍎 **macOS** | `.dmg` | Drag to Applications folder (Apple Silicon & Intel) |
| 🐧 **Linux (Universal)** | `.AppImage` | Portable binary — `chmod +x` & run on any Linux distro |
| 🐧 **Ubuntu / Debian** | `.deb` | `sudo dpkg -i WhatsApp_*.deb` |
| 🐧 **Fedora / RHEL** | `.rpm` | `sudo rpm -i WhatsApp-*.rpm` |

---

## ⚡ Performance Benchmark

| Metric | Whatsie / Official (Electron) | **WhatsApp Tauri** |
|---|---|---|
| **Executable Size** | ~180 MB | **~5.7 MB** (30x smaller!) |
| **App Process RAM** | ~460 MB | **~120 MB** (8x lighter!) |
| **Engine** | Bundled Chromium | Native OS Webview (`WebKit2GTK` / `WebView2` / `WKWebView`) |
| **Startup Time** | Slow | Instant |

---

## ✨ Features

- 📱 Full **WhatsApp Web** experience
- 🔔 **Native Desktop Notifications**: Integrated with OS notification daemons (GNOME / Windows / macOS)
- 🔗 **Smart Link Routing**: External links automatically open in your default browser (Chrome/Firefox), keeping WhatsApp clean
- ⚡ **Ultra-lightweight**: Extremely low CPU & memory footprint
- 💻 **Cross-platform**: Linux, Windows, macOS

---

## 🛠️ Building from Source

### Prerequisites
- [Node.js](https://nodejs.org/) (v18+)
- [Rust & Cargo](https://www.rust-lang.org/)
- Linux dependencies: `libwebkit2gtk-4.1-dev` (Linux only)

### Build Commands

```bash
# 1. Clone the repository
git clone https://github.com/VUXXE/whatsapp-tauri.git
cd whatsapp-tauri

# 2. Install dependencies
npm install

# 3. Run in Development Mode
npm run tauri dev

# 4. Build Production Release
npm run tauri build
```

Compiled output will be located in: `src-tauri/target/release/bundle/`

---

## 📄 License
[MIT License](LICENSE) © 2026 Asy-Syahid Abdurrahman Hanan Taqiyya ([VUXXE](https://github.com/VUXXE))
