# 安装指南 (Installation Guide)

本指南提供了在不同操作系统上安装和配置 LRA 音频响度范围计算器的详细说明。

## 📋 系统要求

### 最低要求
- **操作系统**: Windows 10+, macOS 10.14+, Linux (Ubuntu 18.04+)
- **内存**: 2GB RAM
- **存储空间**: 100MB（不包括音频文件）
- **网络**: 下载依赖时需要互联网连接

### 推荐配置
- **操作系统**: 最新版本的 Windows 11, macOS 12+, Ubuntu 20.04+
- **内存**: 8GB RAM 或更多（处理大量文件时）
- **CPU**: 多核处理器（充分利用并行处理能力）
- **存储空间**: SSD 硬盘（提高 I/O 性能）

## 🛠️ 依赖软件安装

### 1. Rust 编程语言环境

LRA 计算器使用 Rust 编写，需要安装 Rust 工具链。

#### Windows 安装
```powershell
# 方法一：使用 rustup（推荐）
# 访问 https://rustup.rs/ 下载 rustup-init.exe
# 运行安装程序并按照提示操作

# 方法二：使用 Chocolatey
choco install rust

# 验证安装
rustc --version
cargo --version
```

#### macOS 安装
```bash
# 方法一：使用 rustup（推荐）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 方法二：使用 Homebrew
brew install rust

# 验证安装
rustc --version
cargo --version
```

#### Linux 安装
```bash
# Ubuntu/Debian
# 方法一：使用 rustup（推荐）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 方法二：使用包管理器
sudo apt update
sudo apt install rustc cargo

# CentOS/RHEL/Fedora
sudo dnf install rust cargo

# 验证安装
rustc --version
cargo --version
```

### 2. FFmpeg 音频处理工具

FFmpeg 是本工具的核心依赖，用于音频分析。

#### Windows 安装
```powershell
# 方法一：使用 Chocolatey（推荐）
choco install ffmpeg

# 方法二：手动安装
# 1. 访问 https://ffmpeg.org/download.html#build-windows
# 2. 下载 Windows 构建版本
# 3. 解压到 C:\ffmpeg
# 4. 将 C:\ffmpeg\bin 添加到系统 PATH 环境变量

# 方法三：使用 Scoop
scoop install ffmpeg

# 验证安装
ffmpeg -version
```

#### macOS 安装
```bash
# 使用 Homebrew（推荐）
brew install ffmpeg

# 使用 MacPorts
sudo port install ffmpeg

# 验证安装
ffmpeg -version
```

#### Linux 安装
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install ffmpeg

# CentOS/RHEL 8+
sudo dnf install ffmpeg

# CentOS/RHEL 7（需要 EPEL 仓库）
sudo yum install epel-release
sudo yum install ffmpeg

# Fedora
sudo dnf install ffmpeg

# Arch Linux
sudo pacman -S ffmpeg

# 验证安装
ffmpeg -version
```

## 📦 获取项目源码

### 方法一：从 Git 仓库克隆（推荐）
```bash
# 克隆项目
git clone <repository-url>
cd LRA-Calculator-Rust

# 检查项目结构
ls -la
```

### 方法二：下载源码包
```bash
# 如果提供了发布包
wget <release-package-url>
unzip LRA-Calculator-Rust-v*.zip
cd LRA-Calculator-Rust
```

## 🔨 编译和构建

### 开发版本构建
```bash
# 进入项目目录
cd LRA-Calculator-Rust

# 检查依赖
cargo check

# 构建开发版本（包含调试信息）
cargo build

# 运行开发版本
cargo run
```

### 发布版本构建（推荐）
```bash
# 构建优化的发布版本
cargo build --release

# 可执行文件位置
# Linux/macOS: ./target/release/LRA-Calculator-Rust
# Windows: .\target\release\LRA-Calculator-Rust.exe
```

### 构建选项说明
```bash
# 详细输出构建过程
cargo build --release --verbose

# 指定目标平台（交叉编译）
cargo build --release --target x86_64-pc-windows-gnu

# 清理构建缓存
cargo clean
```

## ⚙️ 配置和验证

### 环境变量配置

#### Windows
```powershell
# 添加到用户 PATH（可选，方便全局使用）
$env:PATH += ";C:\path\to\LRA-Calculator-Rust\target\release"

# 永久添加到系统 PATH
# 控制面板 → 系统 → 高级系统设置 → 环境变量
```

#### Linux/macOS
```bash
# 添加到 PATH（可选）
echo 'export PATH="$HOME/path/to/LRA-Calculator-Rust/target/release:$PATH"' >> ~/.bashrc
source ~/.bashrc

# 或者创建符号链接
sudo ln -s /path/to/LRA-Calculator-Rust/target/release/LRA-Calculator-Rust /usr/local/bin/lra-calc
```

### 安装验证

运行以下命令验证安装：

```bash
# 检查可执行文件
./target/release/LRA-Calculator-Rust --help 2>/dev/null || echo "程序启动正常"

# 检查 FFmpeg 集成
./target/release/LRA-Calculator-Rust
# 应该显示 "✓ FFmpeg 检测成功"
```

## 🚀 性能优化配置

### 编译器优化
```toml
# 在 Cargo.toml 中添加（已包含）
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### 系统级优化
```bash
# Linux: 增加文件描述符限制
ulimit -n 65536

# 设置 CPU 亲和性（可选）
taskset -c 0-7 ./target/release/LRA-Calculator-Rust
```

## 🔧 故障排除

### 常见编译错误

#### 错误：链接器未找到
```bash
# Ubuntu/Debian
sudo apt install build-essential

# CentOS/RHEL
sudo yum groupinstall "Development Tools"
```

#### 错误：OpenSSL 相关
```bash
# Ubuntu/Debian
sudo apt install pkg-config libssl-dev

# CentOS/RHEL
sudo yum install openssl-devel
```

### 运行时错误

#### FFmpeg 未找到
```bash
# 确认 FFmpeg 在 PATH 中
which ffmpeg  # Linux/macOS
where ffmpeg  # Windows

# 如果未找到，重新安装或添加到 PATH
```

#### 权限错误
```bash
# 确保对目标目录有读写权限
chmod 755 /path/to/audio/directory
```

## 📱 容器化部署（高级）

### Docker 部署
```dockerfile
# Dockerfile 示例
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ffmpeg
COPY --from=builder /app/target/release/LRA-Calculator-Rust /usr/local/bin/
CMD ["LRA-Calculator-Rust"]
```

```bash
# 构建和运行
docker build -t lra-calculator .
docker run -v /path/to/audio:/audio lra-calculator
```

## 📚 下一步

安装完成后，您可以：

1. 📖 查看 [快速开始指南](./quick-start.md) 进行首次使用
2. 📘 阅读 [用户手册](./user-guide.md) 了解详细功能
3. 🔍 查看 [常见问题](./faq.md) 解决使用问题

---

*如果在安装过程中遇到问题，请查看 [常见问题](./faq.md) 或创建 GitHub Issue。*
