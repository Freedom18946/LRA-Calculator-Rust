# 开发指南 (Development Guide)

本指南为希望参与 LRA 音频响度范围计算器开发的贡献者提供详细的开发环境搭建和贡献流程说明。

## 📋 目录

1. [开发环境搭建](#开发环境搭建)
2. [项目结构说明](#项目结构说明)
3. [开发工作流程](#开发工作流程)
4. [代码规范](#代码规范)
5. [测试指南](#测试指南)
6. [贡献流程](#贡献流程)
7. [发布流程](#发布流程)

## 🛠️ 开发环境搭建

### 必需工具

1. **Rust 工具链**
   ```bash
   # 安装 rustup（Rust 版本管理器）
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # 安装最新稳定版 Rust
   rustup install stable
   rustup default stable
   
   # 安装开发工具
   rustup component add clippy rustfmt rust-src
   ```

2. **FFmpeg**
   ```bash
   # macOS
   brew install ffmpeg
   
   # Ubuntu/Debian
   sudo apt install ffmpeg
   
   # Windows (Chocolatey)
   choco install ffmpeg
   ```

3. **Git**
   ```bash
   # 配置 Git（如果尚未配置）
   git config --global user.name "Your Name"
   git config --global user.email "your.email@example.com"
   ```

### 推荐工具

1. **IDE/编辑器**
   - **VS Code** + Rust Analyzer 插件（推荐）
   - **IntelliJ IDEA** + Rust 插件
   - **Vim/Neovim** + rust.vim + coc-rust-analyzer

2. **调试工具**
   ```bash
   # 安装 cargo-watch（自动重新编译）
   cargo install cargo-watch
   
   # 安装 cargo-expand（宏展开）
   cargo install cargo-expand
   
   # 安装 cargo-audit（安全审计）
   cargo install cargo-audit
   ```

3. **性能分析工具**
   ```bash
   # 安装 flamegraph（性能分析）
   cargo install flamegraph
   
   # 安装 cargo-profdata（LLVM 性能分析）
   cargo install cargo-profdata
   ```

### 项目克隆和初始化

```bash
# 1. Fork 项目到您的 GitHub 账户
# 2. 克隆您的 fork
git clone https://github.com/YOUR_USERNAME/LRA-Calculator-Rust.git
cd LRA-Calculator-Rust

# 3. 添加上游仓库
git remote add upstream https://github.com/ORIGINAL_OWNER/LRA-Calculator-Rust.git

# 4. 安装依赖并构建
cargo build

# 5. 运行测试
cargo test

# 6. 检查代码质量
cargo clippy
cargo fmt --check
```

## 📁 项目结构说明

```
LRA-Calculator-Rust/
├── 📁 src/                    # 源代码目录
│   ├── 📄 main.rs            # 主程序入口，协调各模块
│   ├── 📄 audio.rs           # 音频处理核心模块
│   ├── 📄 processor.rs       # 并行处理引擎
│   ├── 📄 error.rs           # 错误处理系统
│   └── 📄 utils.rs           # 通用工具函数
├── 📁 tests/                  # 集成测试
│   ├── 📄 integration_tests.rs
│   └── 📁 fixtures/          # 测试数据
├── 📁 benches/                # 性能基准测试
│   └── 📄 lra_benchmark.rs
├── 📁 examples/               # 使用示例
│   └── 📄 basic_usage.rs
├── 📁 docs/                   # 文档目录
│   ├── 📄 README.md          # 文档中心
│   ├── 📄 api-reference.md   # API 参考
│   └── 📄 ...                # 其他文档
├── 📁 assets/                 # 资源文件
│   └── 📁 test-audio/        # 测试音频文件
├── 📄 Cargo.toml             # 项目配置
├── 📄 Cargo.lock             # 依赖锁定文件
├── 📄 README.md              # 项目主说明
├── 📄 LICENSE                # 许可证
└── 📄 .gitignore             # Git 忽略规则
```

### 模块职责

| 模块 | 职责 | 关键函数 |
|------|------|----------|
| `main.rs` | 程序入口和流程协调 | `main()` |
| `audio.rs` | 音频文件处理和 LRA 计算 | `scan_audio_files()`, `calculate_lra_direct()` |
| `processor.rs` | 并行处理和进度跟踪 | `process_files_parallel()` |
| `error.rs` | 错误类型定义和处理 | `AppError`, `ProcessFileError` |
| `utils.rs` | 通用工具和辅助功能 | `get_folder_path_from_user()` |

## 🔄 开发工作流程

### 日常开发流程

1. **同步上游更改**
   ```bash
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

2. **创建功能分支**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **开发和测试**
   ```bash
   # 自动重新编译和测试
   cargo watch -x check -x test -x run
   
   # 或者手动执行
   cargo check    # 快速检查编译错误
   cargo test     # 运行测试
   cargo clippy   # 代码质量检查
   cargo fmt      # 代码格式化
   ```

4. **提交更改**
   ```bash
   git add .
   git commit -m "feat: 添加新功能描述"
   ```

5. **推送和创建 PR**
   ```bash
   git push origin feature/your-feature-name
   # 然后在 GitHub 上创建 Pull Request
   ```

### 调试技巧

1. **使用 `dbg!` 宏进行调试**
   ```rust
   let result = calculate_lra_direct(&file_path);
   dbg!(&result);  // 打印调试信息
   ```

2. **环境变量控制日志级别**
   ```bash
   RUST_LOG=debug cargo run
   RUST_LOG=trace cargo run
   ```

3. **使用 GDB 调试**
   ```bash
   cargo build
   rust-gdb target/debug/LRA-Calculator-Rust
   ```

## 📝 代码规范

### Rust 代码风格

1. **使用 `rustfmt` 自动格式化**
   ```bash
   cargo fmt
   ```

2. **遵循 Rust 命名约定**
   ```rust
   // 函数和变量：snake_case
   fn calculate_lra_value() {}
   let file_path = PathBuf::new();
   
   // 类型和 Trait：PascalCase
   struct AudioFile {}
   trait AudioProcessor {}
   
   // 常量：SCREAMING_SNAKE_CASE
   const MAX_FILE_SIZE: usize = 1024;
   ```

3. **文档注释规范**
   ```rust
   /// 计算音频文件的 LRA 值
   /// 
   /// 使用 FFmpeg 的 ebur128 滤波器进行分析，符合 EBU R128 标准。
   /// 
   /// # 参数
   /// 
   /// * `audio_file_path` - 要分析的音频文件路径
   /// 
   /// # 返回值
   /// 
   /// * `Ok(f64)` - 计算得到的 LRA 值（单位：LU）
   /// * `Err(...)` - 分析过程中的错误
   /// 
   /// # 示例
   /// 
   /// ```rust
   /// use std::path::Path;
   /// let lra = calculate_lra_direct(Path::new("audio.mp3"))?;
   /// println!("LRA: {:.1} LU", lra);
   /// ```
   pub fn calculate_lra_direct(
       audio_file_path: &Path,
   ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
       // 实现...
   }
   ```

### 错误处理规范

1. **使用适当的错误类型**
   ```rust
   // 好的做法
   fn process_file(path: &Path) -> Result<f64, ProcessFileError> {
       // ...
   }
   
   // 避免过于宽泛的错误类型
   fn process_file(path: &Path) -> Result<f64, Box<dyn std::error::Error>> {
       // ...
   }
   ```

2. **提供有意义的错误信息**
   ```rust
   Err(ProcessFileError {
       file_path: path.display().to_string(),
       message: format!("无法解析 LRA 值: {}", parse_error),
   })
   ```

### 性能考虑

1. **避免不必要的内存分配**
   ```rust
   // 好的做法：使用引用
   fn process_path(path: &Path) -> Result<(), Error> {}
   
   // 避免：不必要的克隆
   fn process_path(path: PathBuf) -> Result<(), Error> {}
   ```

2. **使用适当的数据结构**
   ```rust
   // 对于大量数据，考虑使用 Vec 而不是 LinkedList
   let mut results: Vec<ProcessResult> = Vec::with_capacity(file_count);
   ```

## 🧪 测试指南

### 测试结构

```rust
// src/audio.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_supported_extensions() {
        assert!(SUPPORTED_EXTENSIONS.contains(&"mp3"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"flac"));
        assert!(!SUPPORTED_EXTENSIONS.contains(&"txt"));
    }

    #[test]
    fn test_scan_audio_files() {
        // 测试文件扫描功能
        let test_dir = Path::new("tests/fixtures/audio");
        let files = scan_audio_files(test_dir, None);
        assert!(!files.is_empty());
    }
}
```

### 集成测试

```rust
// tests/integration_tests.rs
use std::path::Path;
use lra_calculator::{
    audio::{scan_audio_files, calculate_lra_direct},
    processor::process_files_parallel,
};

#[test]
fn test_end_to_end_processing() {
    let test_dir = Path::new("tests/fixtures/audio");
    let files = scan_audio_files(test_dir, None);
    let results = process_files_parallel(files);
    
    // 验证结果
    assert!(!results.is_empty());
    for result in results {
        match result {
            Ok((path, lra)) => {
                assert!(!path.is_empty());
                assert!(lra >= 0.0);
            }
            Err(e) => panic!("处理失败: {}", e),
        }
    }
}
```

### 基准测试

```rust
// benches/lra_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lra_calculator::audio::calculate_lra_direct;
use std::path::Path;

fn benchmark_lra_calculation(c: &mut Criterion) {
    let test_file = Path::new("assets/test-audio/sample.wav");
    
    c.bench_function("calculate_lra_direct", |b| {
        b.iter(|| {
            calculate_lra_direct(black_box(test_file))
        })
    });
}

criterion_group!(benches, benchmark_lra_calculation);
criterion_main!(benches);
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_scan_audio_files

# 运行基准测试
cargo bench

# 生成测试覆盖率报告
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## 🤝 贡献流程

### 提交信息规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<类型>[可选的作用域]: <描述>

[可选的正文]

[可选的脚注]
```

**类型**:
- `feat`: 新功能
- `fix`: 错误修复
- `docs`: 文档更新
- `style`: 代码格式化
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

**示例**:
```
feat(audio): 添加对 Opus 格式的支持

增加了对 Opus 音频格式的 LRA 计算支持，
包括文件扫描和 FFmpeg 参数优化。

Closes #123
```

### Pull Request 流程

1. **确保代码质量**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

2. **更新文档**
   - 更新相关的 API 文档
   - 如果添加新功能，更新用户手册
   - 确保示例代码可以运行

3. **创建 Pull Request**
   - 提供清晰的标题和描述
   - 引用相关的 Issue
   - 包含测试结果截图（如果适用）

4. **代码审查**
   - 响应审查者的反馈
   - 进行必要的修改
   - 保持提交历史清晰

## 🚀 发布流程

### 版本号规范

遵循 [Semantic Versioning](https://semver.org/)：

- `MAJOR.MINOR.PATCH`
- `MAJOR`: 不兼容的 API 更改
- `MINOR`: 向后兼容的功能添加
- `PATCH`: 向后兼容的错误修复

### 发布检查清单

1. **代码质量检查**
   ```bash
   cargo test --all-features
   cargo clippy -- -D warnings
   cargo fmt --check
   cargo audit
   ```

2. **文档更新**
   - 更新 CHANGELOG.md
   - 更新版本号在 Cargo.toml
   - 更新 README.md 中的版本信息

3. **创建发布**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

4. **发布到 crates.io**（如果适用）
   ```bash
   cargo publish
   ```

## 📚 学习资源

### Rust 学习资源
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)

### 音频处理资源
- [EBU R128 标准文档](https://tech.ebu.ch/docs/r/r128.pdf)
- [FFmpeg 文档](https://ffmpeg.org/documentation.html)
- [数字音频处理基础](https://www.dspguide.com/)

---

*欢迎加入我们的开发社区！如有任何问题，请创建 GitHub Issue 或参与讨论。*
