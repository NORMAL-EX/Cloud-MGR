# Cloud-MGR

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows-blue)](https://github.com/NORMAL-EX/Cloud-MGR)

[English](README_en.md) | 简体中文

## 📖 简介

Cloud-MGR 是一款专为 Windows PE 系统设计的插件管理工具，支持 Cloud-PE、HotPE 两大主流 PE 系统。通过统一的界面管理不同 PE 系统的插件，让 PE 维护更加便捷高效。

## ✨ 功能特性

- 🎯 **多 PE 支持**：一个工具管理两种 PE 系统（Cloud-PE/HotPE）
- 📦 **插件市场**：在线浏览、搜索、下载各类 PE 插件
- 🔧 **插件管理**：启用/禁用已安装的插件，灵活控制 PE 功能
- 💾 **智能安装**：自动检测 PE 启动盘，一键安装插件到正确位置
- 🚀 **高速下载**：支持多线程下载（8/16/32线程可选）
- 🎨 **主题切换**：支持浅色/深色主题，可跟随系统设置
- 🔍 **智能搜索**：快速定位需要的插件
- 📂 **分类浏览**：按类别浏览插件，查找更方便

## 🖥️ 系统要求

- Windows 7 SP1 或更高版本
- 需要管理员权限运行
- 至少 50MB 可用磁盘空间
- 互联网连接（用于下载插件）

## 📥 安装

### 从源码编译

需要先安装 Rust 工具链：

```bash
# 克隆仓库
git clone https://github.com/NORMAL-EX/Cloud-MGR.git
cd Cloud-MGR

# 编译发布版本
cargo build --release

# 运行程序
./target/release/cloud-pe-plugin-market.exe
```

## 🚀 使用方法

### 基本使用

1. 以管理员身份运行程序
2. 程序会自动检测已安装的 PE 启动盘
3. 在插件市场浏览或搜索需要的插件
4. 点击"安装"将插件直接安装到启动盘，或点击"下载"保存到本地

### 命令行参数

```bash
# 默认模式（显示选择界面）
cloud-pe-plugin-market.exe

# 直接启动 Cloud-PE 模式
cloud-pe-plugin-market.exe

# 直接启动 HotPE 模式
cloud-pe-plugin-market.exe --hpm

# 显示源选择器
cloud-pe-plugin-market.exe --select
```

### 插件管理

1. 切换到"插件管理"页面查看已安装的插件
2. 点击"禁用"暂时关闭插件功能
3. 点击"启用"重新激活插件

## 🛠️ 配置文件

配置文件位于：`%APPDATA%\CloudPE\plugin_market.json`

支持的配置项：
- `color_mode`: 主题模式（system/light/dark）
- `download_threads`: 下载线程数（8/16/32）
- `default_boot_drive`: 默认启动盘盘符
- `default_download_path`: 默认下载路径

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 开发环境设置

```bash
# 安装依赖
cargo fetch

# 开发模式运行
cargo run

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

## 📄 开源许可

本项目采用 MIT 许可证，详见 [LICENSE](LICENSE) 文件。

## 👨‍💻 作者

- **NORMAL-EX** (别称：dddffgg)
- GitHub: [@NORMAL-EX](https://github.com/NORMAL-EX)

## 🙏 致谢

- [egui](https://github.com/emilk/egui) - Rust GUI 框架
- [tokio](https://tokio.rs/) - 异步运行时
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端

## 📞 联系方式

- 项目主页：[https://github.com/NORMAL-EX/Cloud-MGR](https://github.com/NORMAL-EX/Cloud-MGR)
- 问题反馈：[Issues](https://github.com/NORMAL-EX/Cloud-MGR/issues)

---

© 2025-present Cloud-PE Dev. All rights reserved.