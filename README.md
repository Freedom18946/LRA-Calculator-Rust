# Rust LRA Calculator

这是一个高性能的命令行工具，用于递归计算指定文件夹内所有音频文件的响度范围（Loudness Range, LRA）。它利用多线程并行处理来最大化效率，并使用业界标准的 FFmpeg 进行核心分析。

## 功能特性

- **递归扫描**: 自动扫描指定目录及其所有子目录中的音频文件。
- **多种格式支持**: 支持常见的音频格式，如 `.wav`, `.mp3`, `.m4a`, `.flac`, `.aac` 等。
- **高性能并行处理**: 使用 Rayon 库实现多线程，显著加快大量文件的处理速度。
- **精确的 LRA 计算**: 依赖外部工具 [FFmpeg](https://ffmpeg.org/) 和 `ebur128` 滤波器进行精确的响度分析，符合 EBU R128 标准。
- **清晰的结果输出**: 将所有结果保存在一个名为 `lra_results.txt` 的文件中，包含文件相对路径和对应的 LRA 值 (单位: LU)。
- **自动排序**: 结果文件会按照 LRA 值从高到低自动排序，方便查看。

## 系统要求

- **Rust**: 需要安装 Rust 语言环境和 Cargo 包管理器。您可以从 [rust-lang.org](https://www.rust-lang.org/tools/install) 获取。
- **FFmpeg**: 必须在您的系统上安装 FFmpeg，并且其路径需要被包含在系统的 `PATH` 环境变量中，以便本程序可以调用它。

## 安装与构建

1.  **克隆仓库** (如果您从 git 获取):
    ```bash
    git clone <repository-url>
    cd LRA-Calculator-Rust
    ```

2.  **构建项目**:
    使用 Cargo 构建优化的发行版本。
    ```bash
    cargo build --release
    ```
    编译后的可执行文件将位于 `./target/release/LRA-Calculator-Rust`。

## 使用方法

1.  **运行程序**:
    直接运行编译好的可执行文件。
    ```bash
    ./target/release/LRA-Calculator-Rust
    ```

2.  **输入路径**:
    程序启动后，会提示您输入要处理的音乐文件夹的顶层路径。
    ```
    欢迎使用音频 LRA 计算器（高性能版 - 直接分析）！
    当前时间: 2025-07-22 10:30:00
    请输入要递归处理的音乐顶层文件夹路径: /path/to/your/music/folder
    ```

3.  **处理过程**:
    程序将开始扫描文件并使用多个线程并行分析。您会在终端看到实时的处理进度。
    ```
    正在递归扫描文件夹: /path/to/your/music/folder
    扫描完成，找到 150 个音频文件待处理。
    开始多线程直接分析...
      [线程 ThreadId(2)] (1/150) 直接分析: album1/track01.mp3
      [线程 ThreadId(3)] (2/150) 直接分析: album1/track02.flac
      ...
    ```

4.  **查看结果**:
    所有操作完成后，会在您指定的顶层文件夹中生成一个 `lra_results.txt` 文件。

    **`lra_results.txt` 示例内容:**
    ```
    文件路径 (相对) - LRA 数值 (LU)
    classical/symphony_no_5.wav - 18.5
    rock/stairway_to_heaven.flac - 14.2
    pop/single.m4a - 8.1
    podcast/episode_01.mp3 - 5.5
    ```

## 项目结构

```
.
├── Cargo.toml      # 项目配置文件，定义依赖项
├── src/
│   └── main.rs     # 主要的程序逻辑
├── target/         # 编译输出目录
└── README.md       # 本说明文件
```

- `src/main.rs`: 包含程序的所有核心逻辑，包括文件扫描、用户输入处理、调用 FFmpeg、并行计算以及结果的写入和排序。
- `Cargo.toml`: 定义了项目元数据和依赖项，如 `rayon` (用于并行处理) 和 `regex` (用于解析 FFmpeg 输出)。
