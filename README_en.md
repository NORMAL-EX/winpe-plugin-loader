# WinPE Plugin Loader

English | [简体中文](README.md)

A powerful WinPE plugin loader that supports CE and HPM plugin formats.

## ✨ Features

- 🚀 **Multi-Format Support** - Supports CE and HPM plugin formats
- 🔄 **Auto Scan** - Automatically scans all disk drives for plugin directories
- 📦 **Batch Loading** - Load all discovered plugins with one command
- 🛠️ **Configuration-Driven** - CE plugins support `lnk.cfg` for advanced features
- ⚡ **High Performance** - Written in Rust for excellent performance
- 🔒 **Safe & Reliable** - Strict error handling and resource management

## 📥 Installation

Download the latest `winpe_plugin_loader.exe` from [Releases](https://github.com/NORMAL-EX/winpe-plugin-loader/releases).

Or build from source:

```bash
git clone https://github.com/NORMAL-EX/winpe-plugin-loader.git
cd winpe-plugin-loader
cargo build --release
```

The compiled binary is located at `target/release/winpe_plugin_loader.exe`

## 🚀 Usage

### Basic Usage

```bash
# Search and load all plugins (execute once only)
winpe_plugin_loader.exe main

# Load a specific CE plugin
winpe_plugin_loader.exe "X:\Path\To\Plugin.ce"

# Load a specific HPM module
winpe_plugin_loader.exe "X:\Path\To\Module.hpm"

# Show help information
winpe_plugin_loader.exe
```

### Auto Load All Plugins

The `main` command automatically scans all disk drives and loads discovered plugins:

- **CE Plugins** - Searches for `.ce` files in `*:\Cloud-PE\Plugins`
- **HPM Modules** - Searches for `.hpm` files in `*:\HotPEModule`

## 🏗️ Building

### Prerequisites

- Rust 1.70 or higher
- Windows 10/11 or WinPE environment

### Build Steps

```bash
# Clone repository
git clone https://github.com/NORMAL-EX/winpe-plugin-loader.git
cd winpe-plugin-loader

# Debug build
cargo build

# Release build
cargo build --release

# Run
cargo run -- main
```

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Cloud-PE](https://www.cloud-pe.cn/) - CE plugin format specification
- [HotPE](https://www.hotpe.top/) - HPM module format

## 📞 Contact

- Project Homepage: https://github.com/NORMAL-EX/winpe-plugin-loader
- Issue Tracker: [Issues](https://github.com/NORMAL-EX/winpe-plugin-loader/issues)

---

**Note**: This tool is for educational and legitimate use only. Please respect software authors' copyrights and license agreements.
