<picture>
  <source media="(prefers-color-scheme: dark)" srcset="icon.svg">
  <img src="icon.svg" alt="rdesq" height="80">
</picture>

# rdesq

**Remote Desktop Connection Manager** — a cross-platform desktop app for organizing and launching SSH and RDP connections.

Built with [Tauri 2](https://v2.tauri.app) (Rust backend + vanilla JS frontend).

---

## Features

- **SSH & RDP** — launch connections via system `ssh` or `xfreerdp`/`mstsc`
- **Groups** — color-coded, collapsible, drag-and-drop reordering
- **Tags** — filter and organize connections; global tag manager
- **Favorites** — bookmark frequently used connections
- **Ping** — async TCP ping with online/offline indicators
- **Search & Sort** — real-time filtering by name, host, or tags
- **Export / Import** — full JSON backup and restore
- **Password Encryption** — AES-256-GCM at rest
- **Dark & Light themes**
- **Bilingual** — English and Polish (default)
- **Quick Connect** — instant modal to connect without saving
- **System Tray** — close to tray with language-aware menu
- **Autostart** — launch on system startup

---

## Install

Download the latest release for your platform:

| Platform | Format |
|----------|--------|
| Linux    | [`.deb`](https://github.com/eincherjar/rdesq/releases/latest), [`.rpm`](https://github.com/eincherjar/rdesq/releases/latest), [`.AppImage`](https://github.com/eincherjar/rdesq/releases/latest) |
| Windows  | [`.exe` (NSIS installer)](https://github.com/eincherjar/rdesq/releases/latest) |

### Linux

```bash
# Debian / Ubuntu
sudo dpkg -i rdesq_*_amd64.deb

# Fedora / openSUSE
sudo rpm -i rdesq-*.x86_64.rpm

# AppImage (any distro)
chmod +x rdesq-*.AppImage
./rdesq-*.AppImage
```

### Windows

Run the downloaded `.exe` installer — no admin rights required (per-user install).

---

## Build from source

### Prerequisites

- [Rust](https://rustup.rs)
- [Node.js](https://nodejs.org) 18+
- Linux: system dependencies for Tauri

```bash
# Debian / Ubuntu
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev \
  librsvg2-dev patchelf libgtk-3-dev libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev
```

### Build

```bash
npm ci
npm run build
```

Artifacts are written to `src-tauri/target/release/bundle/`.

---

## Usage

1. Add a connection with **Ctrl+N** or the ➕ button.
2. Double-click any entry to connect.
3. Use **Ctrl+F** to search, **Ctrl+Q** for quick connect.
4. Organize connections into groups via drag-and-drop.
5. Right-click for more options — edit, duplicate, delete, toggle favorite.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2 |
| Backend | Rust — rusqlite, aes-gcm, tokio, serde |
| Frontend | Vanilla HTML / CSS / JS (no framework) |
| Database | SQLite (WAL mode) |
| CI/CD | GitHub Actions — builds `.deb`, `.rpm`, `.AppImage`, `.exe` |

---

## License

[MIT](LICENSE)
