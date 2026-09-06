<div align="center">

# 🚀 WhatsApp Tauri

### Lightweight • Fast • Cross-Platform WhatsApp Desktop Client

[![GitHub Release](https://img.shields.io/github/v/release/VUXXE/whatsapp-tauri?style=for-the-badge&color=25D366&labelColor=111827)](https://github.com/VUXXE/whatsapp-tauri/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/VUXXE/whatsapp-tauri/release.yml?style=for-the-badge&label=Build&color=3B82F6&labelColor=111827)](https://github.com/VUXXE/whatsapp-tauri/actions)
[![License](https://img.shields.io/github/license/VUXXE/whatsapp-tauri?style=for-the-badge&color=10B981&labelColor=111827)](LICENSE)

[![Tauri v2](https://img.shields.io/badge/Tauri_v2-FFC131?style=for-the-badge&logo=tauri&logoColor=black)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/VUXXE/whatsapp-tauri/releases)
[![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/VUXXE/whatsapp-tauri/releases)
[![macOS](https://img.shields.io/badge/macOS-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/VUXXE/whatsapp-tauri/releases)

<p align="center">
  A high-performance alternative to resource-heavy Electron apps (like Whatsie or Web Wrappers).
</p>

</div>

---

## ⚡ Why WhatsApp Tauri?

| Metric | Whatsie / Official (Electron) | **WhatsApp Tauri** | Improvement |
|---|---|---|---|
| **Binary Size** | ~180 MB | **~5.7 MB** | 🚀 **30x Smaller** |
| **App Process Memory** | ~460 MB | **~120 MB** | ⚡ **8x Lighter** |
| **Backend Engine** | Bundled Chromium | Native OS Webview | 🌿 **Zero Overhead** |
| **Startup Speed** | Slow / Heavy | Instant | ⚡ **Blazing Fast** |

---

## ✨ Features

- 📱 **Full WhatsApp Web Experience**: Chat, voice notes, media sharing, and status updates.
- 🔔 **Native Desktop Notifications**: Direct integration with system notification daemons (GNOME / Windows / macOS).
- 🔗 **Smart Link Handler**: Clicking external URLs opens them directly in your default web browser (Chrome, Firefox, etc.).
- ⚡ **Ultra Low Footprint**: Built with Rust and Tauri v2 for minimal RAM and CPU utilization.
- 🌐 **Cross-Platform Support**: Binaries compiled natively for Windows, macOS, and Linux.

---

## 📥 Downloads & Installation

Download pre-compiled binaries from the **[Releases Page](https://github.com/VUXXE/whatsapp-tauri/releases/latest)**:

| Platform | Recommended Package | Quick Installation Command |
|---|---|---|
| 🪟 **Windows** | `.exe` / `.msi` | Double-click the installer |
| 🍎 **macOS** | `.dmg` | Drag `WhatsApp.app` to `/Applications` |
| 🐧 **Linux (Universal)** | `.AppImage` | `chmod +x WhatsApp_*.AppImage && ./WhatsApp_*.AppImage` |
| 🐧 **Ubuntu / Debian** | `.deb` | `sudo dpkg -i WhatsApp_*.deb` |
| 🐧 **Fedora / RHEL** | `.rpm` | `sudo rpm -i WhatsApp-*.rpm` |

---

## 🛠️ Building from Source

### Prerequisites
- [Node.js](https://nodejs.org/) (v18+)
- [Rust & Cargo](https://www.rust-lang.org/)
- Linux dependencies: `libwebkit2gtk-4.1-dev` (Linux only)

### Commands

```bash
# 1. Clone Repository
git clone https://github.com/VUXXE/whatsapp-tauri.git
cd whatsapp-tauri

# 2. Install Dependencies
npm install

# 3. Development Mode
npm run tauri dev

# 4. Build Production Packages
npm run tauri build
```

---

## 📄 License

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for more details.

<div align="center">
  Crafted with ❤️ by <a href="https://github.com/VUXXE">VUXXE</a>
</div>
