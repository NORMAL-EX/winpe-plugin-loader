# WinPE Plugin Loader

[English](README_en.md) | 简体中文

一个强大的 WinPE 插件加载器，支持 CE、 和 HPM 两种插件格式。

## ✨ 特性

- 🚀 **多格式支持** - 支持 CE 和 HPM 两种主流插件格式
- 🔄 **自动扫描** - 自动搜索所有磁盘上的插件目录
- 📦 **批量加载** - 一键加载所有找到的插件
- 🛠️ **配置驱动** - CE 插件支持 `lnk.cfg` 配置文件实现复杂功能
- ⚡ **高性能** - 使用 Rust 编写，性能优异
- 🔒 **安全可靠** - 严格的错误处理和资源管理

## 📥 安装

从 [Releases](https://github.com/NORMAL-EX/winpe-plugin-loader/releases) 下载最新版本的 `winpe_plugin_loader.exe`。

或者从源码编译：

```bash
git clone https://github.com/NORMAL-EX/winpe-plugin-loader.git
cd winpe-plugin-loader
cargo build --release
```

编译后的文件位于 `target/release/winpe_plugin_loader.exe`

## 🚀 使用方法

### 基本用法

```bash
# 搜索并加载所有插件（仅执行一次）
winpe_plugin_loader.exe main

# 加载指定的 CE 插件
winpe_plugin_loader.exe "X:\Path\To\Plugin.ce"

# 加载指定的 HPM 模块
winpe_plugin_loader.exe "X:\Path\To\Module.hpm"

# 显示帮助信息
winpe_plugin_loader.exe
```

### 自动加载所有插件

`main` 命令会自动扫描所有磁盘驱动器并加载找到的插件：

- **CE 插件** - 搜索 `*:\Cloud-PE\Plugins` 目录下的 `.ce` 文件
- **HPM 模块** - 搜索 `*:\HotPEModule` 目录下的 `.hpm` 文件

## 🏗️ 构建

### 前置要求

- Rust 1.70 或更高版本
- Windows 10/11 或 WinPE 环境

### 编译步骤

```bash
# 克隆仓库
git clone https://github.com/NORMAL-EX/winpe-plugin-loader.git
cd winpe-plugin-loader

# Debug 编译
cargo build

# Release 编译
cargo build --release

# 运行
cargo run -- main
```

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建你的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交你的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开一个 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [Cloud-PE](https://www.cloud-pe.cn/) - CE 插件格式规范
- [HotPE](https://www.hotpe.top/) - HPM 模块格式

## 📞 联系方式

- 项目主页: https://github.com/NORMAL-EX/winpe-plugin-loader
- 问题反馈: [Issues](https://github.com/NORMAL-EX/winpe-plugin-loader/issues)

---

**注意**: 本工具仅供学习和合法用途使用。请尊重软件作者的版权和许可协议。
