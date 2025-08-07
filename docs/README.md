# LRA 音频响度范围计算器 - 文档中心

欢迎来到 LRA 音频响度范围计算器的文档中心。本项目是一个高性能的 Rust 命令行工具，专门用于计算音频文件的响度范围（Loudness Range, LRA）。

## 📚 文档导航

### 用户文档
- [快速开始指南](./quick-start.md) - 5分钟上手使用
- [安装指南](./installation.md) - 详细的安装和配置说明
- [用户手册](./user-guide.md) - 完整的使用说明和功能介绍
- [常见问题](./faq.md) - 常见问题解答

### 开发者文档
- [架构设计](./architecture.md) - 项目架构和设计理念
- [API 参考](./api-reference.md) - 详细的 API 文档
- [开发指南](./development.md) - 开发环境搭建和贡献指南
- [性能基准](./benchmarks.md) - 性能测试结果和优化记录

### 技术文档
- [LRA 算法说明](./lra-algorithm.md) - 响度范围算法的技术细节
- [FFmpeg 集成](./ffmpeg-integration.md) - FFmpeg 集成方案说明
- [并发处理](./concurrency.md) - 多线程并发处理机制

## 🎯 项目概述

LRA（Loudness Range）是音频工程中的重要指标，用于衡量音频内容的动态范围。本工具基于 EBU R128 标准，提供：

- **高性能并行处理** - 利用多核 CPU 加速大批量文件处理
- **广泛格式支持** - 支持 WAV、MP3、FLAC、AAC 等主流音频格式
- **精确测量** - 基于 FFmpeg 的 ebur128 滤波器，符合国际标准
- **友好的中文界面** - 完全中文化的用户界面和文档

## 🚀 快速开始

```bash
# 1. 克隆项目
git clone <repository-url>
cd LRA-Calculator-Rust

# 2. 构建项目
cargo build --release

# 3. 运行程序
./target/release/LRA-Calculator-Rust
```

## 📊 支持的音频格式

| 格式 | 扩展名 | 说明 |
|------|--------|------|
| WAV | .wav | 无损音频格式 |
| FLAC | .flac | 无损压缩音频 |
| MP3 | .mp3 | 有损压缩音频 |
| AAC | .aac, .m4a | 高效音频编码 |
| OGG | .ogg | 开源音频格式 |
| Opus | .opus | 现代音频编解码器 |
| WMA | .wma | Windows 媒体音频 |
| AIFF | .aiff | 苹果音频格式 |
| ALAC | .alac | 苹果无损音频 |

## 🔧 系统要求

- **操作系统**: Windows 10+, macOS 10.14+, Linux (Ubuntu 18.04+)
- **Rust**: 1.70.0 或更高版本
- **FFmpeg**: 4.0 或更高版本（必须在 PATH 中可用）
- **内存**: 建议 4GB 以上（处理大量文件时）
- **存储**: 根据音频文件数量而定

## 📈 性能特性

- **并行处理**: 自动利用所有可用 CPU 核心
- **内存优化**: 流式处理，避免大文件内存占用
- **进度显示**: 实时显示处理进度和线程状态
- **错误恢复**: 单个文件错误不影响整体处理

## 🤝 贡献指南

我们欢迎社区贡献！请查看 [开发指南](./development.md) 了解如何参与项目开发。

## 📄 许可证

本项目采用 MIT 许可证。详情请查看 LICENSE 文件。

## 📞 支持与反馈

如果您在使用过程中遇到问题或有改进建议，请：

1. 查看 [常见问题](./faq.md)
2. 搜索现有的 GitHub Issues
3. 创建新的 Issue 描述问题
4. 联系维护者

---

*最后更新: 2025-08-07*

## 📁 项目结构

```
LRA-Calculator-Rust/
├── 📁 src/                    # 源代码目录
│   ├── 📄 main.rs            # 主程序入口
│   ├── 📄 audio.rs           # 音频处理模块
│   ├── 📄 processor.rs       # 并行处理模块
│   ├── 📄 error.rs           # 错误处理模块
│   └── 📄 utils.rs           # 实用工具模块
├── 📁 docs/                   # 文档目录
│   ├── 📄 README.md          # 文档中心（本文件）
│   ├── 📄 quick-start.md     # 快速开始指南
│   ├── 📄 installation.md    # 安装指南
│   ├── 📄 user-guide.md      # 用户手册
│   ├── 📄 faq.md             # 常见问题
│   ├── 📄 architecture.md    # 架构设计
│   ├── 📄 api-reference.md   # API 参考
│   ├── 📄 development.md     # 开发指南
│   ├── 📄 benchmarks.md      # 性能基准
│   ├── 📄 lra-algorithm.md   # LRA 算法说明
│   ├── 📄 ffmpeg-integration.md # FFmpeg 集成
│   └── 📄 concurrency.md     # 并发处理
├── 📁 tests/                  # 测试目录
├── 📁 benches/                # 基准测试目录
├── 📁 examples/               # 示例代码目录
├── 📁 assets/                 # 资源文件目录
├── 📄 Cargo.toml             # 项目配置文件
├── 📄 README.md              # 项目主说明文件
└── 📄 LICENSE                # 许可证文件
```
