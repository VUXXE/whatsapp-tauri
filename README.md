# 🚀 WhatsApp Tauri

A super lightweight, cross-platform desktop client for **WhatsApp Web** built with **Tauri v2** and **Rust**.

Designed as a high-performance alternative to resource-heavy Electron apps like Whatsie / Official Desktop Web wrappers.

---

## ⚡ Performance Comparison

| Metric | Official / Whatsie (Electron) | **WhatsApp Tauri** |
|---|---|---|
| **Executable Size** | ~180 MB | **~5.7 MB** (30x smaller!) |
| **RAM Usage** | ~1.2 GB | **~140–180 MB** (8x lighter!) |
| **Backend Engine** | Chromium (bundled) | Native OS Webview (WebKitGTK / WebView2) |
| **Startup Time** | Slow | Instant |

---

## 🛠️ Features

- 📱 Full **WhatsApp Web** functionality
- 💻 **Cross-platform**: Linux, Windows, macOS
- ⚡ **Ultra-lightweight**: Extremely low memory & CPU footprint
- 🖥️ Custom User-Agent handling for seamless WebKitGTK compatibility

---

## 📦 Building from Source

### Prerequisites
- Node.js & npm
- Rust (`rustc` & `cargo`)
- `webkit2gtk-4.1` (Linux)

### Commands

```bash
# Clone the repository
git clone https://github.com/VUXXE/whatsapp-tauri.git
cd whatsapp-tauri

# Install dependencies
npm install

# Run in Development Mode
npm run tauri dev

# Build Release Binary
npm run tauri build
```

The compiled binary will be available at:
`src-tauri/target/release/whatsapp-tauri`

---

## 📄 License
MIT License
