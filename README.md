# ⚡ StarDash

A cross-platform system monitoring dashboard built with Tauri + Vue + Rust.

![StarDash Screenshot](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![GitHub Release](https://img.shields.io/github/v/release/starsdaisuki/StarDash)
![License](https://img.shields.io/badge/License-MIT-green)

## Features

- **CPU Monitoring** — Real-time usage, per-core breakdown, 90-second history chart
- **Memory Tracking** — Usage stats with live trend visualization
- **Disk Info** — Capacity, usage, and filesystem details for all partitions
- **Network Monitor** — Upload/download speed, local IPs, public IP with VPN detection
- **Process Manager** — Top 10 processes by CPU usage
- **Battery Status** — Charge level, health, cycle count (laptops)
- **Glassmorphism UI** — Dark gradient background with frosted glass cards

## Screenshots

> TODO: Add screenshots here

## Download

Go to [Releases](https://github.com/starsdaisuki/StarDash/releases) and download the installer for your platform:

| Platform | File |
|----------|------|
| Windows | `StarDash_x64-setup.exe` or `.msi` |
| macOS (Apple Silicon) | `StarDash_aarch64.dmg` |
| macOS (Intel) | `StarDash_x64.dmg` |
| Linux (Debian/Ubuntu) | `StarDash_amd64.deb` |
| Linux (Fedora) | `StarDash_x86_64.rpm` |
| Linux (Universal) | `StarDash_amd64.AppImage` |

## Tech Stack

- **Frontend**: Vue 3 + TypeScript
- **Backend**: Rust
- **Framework**: Tauri 2
- **Build Tool**: Vite
- **CI/CD**: GitHub Actions (auto-build for Windows, macOS, Linux)

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/)
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/)

### Setup

```bash
git clone https://github.com/starsdaisuki/StarDash.git
cd StarDash
pnpm install
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

## License

MIT