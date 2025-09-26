# Cloud-MGR

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows-blue)](https://github.com/NORMAL-EX/Cloud-MGR)

English | [简体中文](README.md)

## 📖 Introduction

Cloud-MGR is a plugin management tool designed for Windows PE systems, supporting two major PE systems: Cloud-PE and HotPE. It provides a unified interface to manage plugins for different PE systems, making PE maintenance more convenient and efficient.

## ✨ Features

- 🎯 **Multi-PE Support**: Manage two PE systems with one tool (Cloud-PE/HotPE)
- 📦 **Plugin Market**: Browse, search, and download various PE plugins online
- 🔧 **Plugin Management**: Enable/disable installed plugins for flexible PE functionality control
- 💾 **Smart Installation**: Auto-detect PE boot drives and install plugins with one click
- 🚀 **High-Speed Download**: Multi-threaded download support (8/16/32 threads)
- 🎨 **Theme Switching**: Light/Dark theme support with system theme following
- 🔍 **Smart Search**: Quickly locate the plugins you need
- 📂 **Category Browsing**: Browse plugins by category for easier discovery

## 🖥️ System Requirements

- Windows 7 SP1 or higher
- Administrator privileges required
- At least 50MB available disk space
- Internet connection (for downloading plugins)

## 📥 Installation

### Build from Source

Requires Rust toolchain installation:

```bash
# Clone repository
git clone https://github.com/NORMAL-EX/Cloud-MGR.git
cd Cloud-MGR

# Build release version
cargo build --release

# Run the program
./target/release/cloud-pe-plugin-market.exe
```

## 🚀 Usage

### Basic Usage

1. Run the program as administrator
2. The program will automatically detect installed PE boot drives
3. Browse or search for plugins in the plugin market
4. Click "Install" to install directly to boot drive, or "Download" to save locally

### Command Line Arguments

```bash
# Default mode (shows selection interface)
cloud-pe-plugin-market.exe

# Launch Cloud-PE mode directly
cloud-pe-plugin-market.exe

# Launch HotPE mode directly
cloud-pe-plugin-market.exe --hpm

# Show source selector
cloud-pe-plugin-market.exe --select
```

### Plugin Management

1. Switch to "Plugin Management" page to view installed plugins
2. Click "Disable" to temporarily disable plugin functionality
3. Click "Enable" to reactivate plugins

## 🛠️ Configuration

Configuration file location: `%APPDATA%\CloudPE\plugin_market.json`

Supported configuration options:
- `color_mode`: Theme mode (system/light/dark)
- `download_threads`: Download thread count (8/16/32)
- `default_boot_drive`: Default boot drive letter
- `default_download_path`: Default download path

## 🤝 Contributing

Issues and Pull Requests are welcome!

### Development Setup

```bash
# Install dependencies
cargo fetch

# Run in development mode
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👨‍💻 Author

- **NORMAL-EX** (aka: dddffgg)
- GitHub: [@NORMAL-EX](https://github.com/NORMAL-EX)

## 🙏 Acknowledgments

- [egui](https://github.com/emilk/egui) - Rust GUI framework
- [tokio](https://tokio.rs/) - Async runtime
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client

## 📞 Contact

- Project Homepage: [https://github.com/NORMAL-EX/Cloud-MGR](https://github.com/NORMAL-EX/Cloud-MGR)
- Issue Tracker: [Issues](https://github.com/NORMAL-EX/Cloud-MGR/issues)

---

© 2025-present Cloud-PE Dev. All rights reserved.