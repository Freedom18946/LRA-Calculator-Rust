# 架构设计 (Architecture Design)

本文档详细介绍了 LRA 音频响度范围计算器的系统架构、设计理念和技术实现。

## 📋 目录

1. [系统概述](#系统概述)
2. [架构原则](#架构原则)
3. [模块设计](#模块设计)
4. [数据流程](#数据流程)
5. [并发模型](#并发模型)
6. [错误处理](#错误处理)
7. [性能考量](#性能考量)

## 🎯 系统概述

### 设计目标

LRA 计算器的设计围绕以下核心目标：

- **高性能**: 充分利用多核 CPU 进行并行处理
- **可靠性**: 单个文件错误不影响整体处理流程
- **可扩展性**: 模块化设计，便于功能扩展
- **用户友好**: 清晰的中文界面和详细的进度反馈
- **标准兼容**: 严格遵循 EBU R128 标准

### 技术栈选择

| 技术 | 选择理由 |
|------|----------|
| **Rust** | 内存安全、零成本抽象、优秀的并发支持 |
| **Rayon** | 数据并行处理库，简化多线程编程 |
| **FFmpeg** | 业界标准音频处理工具，支持广泛格式 |
| **Regex** | 高效的文本模式匹配，解析 FFmpeg 输出 |
| **WalkDir** | 递归目录遍历，处理复杂目录结构 |
| **Chrono** | 时间处理，提供时间戳功能 |

## 🏗️ 架构原则

### 1. 模块化设计 (Modular Design)

```
src/
├── main.rs          # 主程序协调器
├── audio.rs         # 音频处理核心
├── processor.rs     # 并行处理引擎
├── error.rs         # 错误处理系统
└── utils.rs         # 通用工具函数
```

每个模块职责单一，接口清晰，便于测试和维护。

### 2. 关注点分离 (Separation of Concerns)

- **用户交互**: 输入验证、进度显示、结果输出
- **文件处理**: 目录扫描、文件过滤、路径处理
- **音频分析**: FFmpeg 调用、LRA 计算、结果解析
- **并发控制**: 线程管理、任务分发、结果收集
- **错误处理**: 异常捕获、错误分类、恢复策略

### 3. 数据驱动设计 (Data-Driven Design)

```rust
// 核心数据结构
pub struct AudioFile {
    pub path: PathBuf,
    pub display_path: String,
}

pub struct ProcessingResult {
    pub file_path: String,
    pub lra_value: Option<f64>,
    pub error: Option<ProcessFileError>,
}

pub struct ProcessingStats {
    pub successful: usize,
    pub failed: usize,
    pub error_messages: Vec<String>,
}
```

## 🔧 模块设计

### 主程序模块 (main.rs)

**职责**: 程序入口点和流程协调

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化和环境检查
    check_ffmpeg_availability()?;
    
    // 2. 获取用户输入
    let base_folder_path = get_folder_path_from_user()?;
    
    // 3. 扫描音频文件
    let files_to_process = scan_audio_files(&base_folder_path, exclude_file);
    
    // 4. 并行处理
    let processing_results = process_files_parallel(files_to_process);
    
    // 5. 结果分析和输出
    let (stats, successful_results) = analyze_results(processing_results);
    write_results_file(&results_file_path, successful_results)?;
    sort_lra_results_file(&results_file_path, header_line)?;
    
    Ok(())
}
```

### 音频处理模块 (audio.rs)

**职责**: 音频文件扫描和 LRA 计算

**核心函数**:

1. **文件扫描**:
   ```rust
   pub fn scan_audio_files(
       base_path: &Path,
       exclude_file: Option<&Path>,
   ) -> Vec<(PathBuf, String)>
   ```

2. **LRA 计算**:
   ```rust
   pub fn calculate_lra_direct(
       audio_file_path: &Path,
   ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>>
   ```

3. **FFmpeg 集成**:
   ```rust
   pub fn check_ffmpeg_availability() -> Result<(), AppError>
   ```

**设计特点**:
- 支持多种音频格式的统一处理
- 使用正则表达式解析 FFmpeg 输出
- 错误处理与主流程分离

### 并行处理模块 (processor.rs)

**职责**: 多线程并行处理和进度跟踪

**核心架构**:
```rust
pub fn process_files_parallel(
    files_to_process: Vec<(PathBuf, String)>,
) -> Vec<Result<(String, f64), ProcessFileError>> {
    let total_files = files_to_process.len();
    let processed_count = AtomicUsize::new(0);

    files_to_process
        .into_par_iter()  // Rayon 并行迭代器
        .map(|(file_path, display_path)| {
            // 原子计数器跟踪进度
            let current_count = processed_count.fetch_add(1, Ordering::SeqCst) + 1;
            
            // 线程安全的进度输出
            println!("  [线程 {:?}] ({}/{}) 分析: {}", 
                thread::current().id(), current_count, total_files, display_path);
            
            // 执行 LRA 计算
            match calculate_lra_direct(&file_path) {
                Ok(lra) => Ok((display_path, lra)),
                Err(e) => Err(ProcessFileError { file_path: display_path, message: e.to_string() }),
            }
        })
        .collect()
}
```

**并发特性**:
- 使用 Rayon 的数据并行模型
- 自动负载均衡
- 线程安全的进度报告
- 错误隔离机制

### 错误处理模块 (error.rs)

**职责**: 统一的错误类型定义和处理

**错误层次结构**:
```rust
#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),           // I/O 操作错误
    FileProcessing(ProcessFileError), // 文件处理错误
    Ffmpeg(String),               // FFmpeg 相关错误
    Path(String),                 // 路径相关错误
}

#[derive(Debug)]
pub struct ProcessFileError {
    pub file_path: String,        // 出错文件路径
    pub message: String,          // 错误描述
}
```

**错误处理策略**:
- 区分可恢复和不可恢复错误
- 提供详细的中文错误信息
- 支持错误链追踪

### 工具模块 (utils.rs)

**职责**: 通用工具函数和辅助功能

**主要功能**:
- 用户输入处理和验证
- 文件排序和格式化
- 路径处理和规范化

## 🔄 数据流程

### 整体数据流

```
用户输入路径 → 路径验证 → 递归扫描目录 → 过滤音频文件 → 创建处理任务
                                                                    ↓
文件输出 ← 结果排序 ← 统计分析 ← 结果收集 ← LRA值解析 ← FFmpeg分析 ← 并行处理池
```

### 并行处理流程

```
文件列表 → Rayon并行迭代器 → 线程池
                                ├── 工作线程1 → FFmpeg调用 ┐
                                ├── 工作线程2 → FFmpeg调用 ├→ 结果收集器 → 统计输出
                                └── 工作线程N → FFmpeg调用 ┘
```

## ⚡ 并发模型

### Rayon 数据并行

**选择理由**:
- 简化并行编程复杂性
- 自动工作窃取调度
- 零成本抽象
- 与 Rust 所有权系统完美集成

**实现细节**:
```rust
// 并行迭代器自动处理线程管理
files_to_process
    .into_par_iter()           // 转换为并行迭代器
    .map(|file| process_file(file))  // 并行映射操作
    .collect()                 // 收集结果
```

### 线程安全设计

**原子操作**:
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

let processed_count = AtomicUsize::new(0);
let current = processed_count.fetch_add(1, Ordering::SeqCst);
```

**无锁数据结构**:
- 使用不可变数据结构避免竞争条件
- 通过函数式编程模式减少共享状态
- 利用 Rust 的所有权系统保证内存安全

## 🛡️ 错误处理

### 错误处理策略

1. **快速失败 (Fail Fast)**:
   - 环境检查失败立即退出
   - 关键资源不可用时停止执行

2. **错误隔离 (Error Isolation)**:
   - 单个文件处理失败不影响其他文件
   - 并行处理中的异常被独立捕获

3. **优雅降级 (Graceful Degradation)**:
   - 部分文件失败时继续处理其余文件
   - 提供详细的失败统计和错误信息

### 错误恢复机制

```rust
// 示例：文件处理错误恢复
match calculate_lra_direct(&file_path) {
    Ok(lra) => {
        // 成功处理
        Ok((display_path, lra))
    }
    Err(e) => {
        // 记录错误但继续处理其他文件
        eprintln!("警告: 文件 {} 处理失败: {}", display_path, e);
        Err(ProcessFileError {
            file_path: display_path,
            message: e.to_string(),
        })
    }
}
```

## 🚀 性能考量

### 性能优化策略

1. **编译时优化**:
   ```toml
   [profile.release]
   opt-level = 3        # 最高优化级别
   lto = true          # 链接时优化
   codegen-units = 1   # 单个代码生成单元
   panic = "abort"     # 减少异常处理开销
   ```

2. **运行时优化**:
   - 零拷贝字符串处理
   - 流式文件处理
   - 内存池复用

3. **I/O 优化**:
   - 缓冲写入
   - 异步文件操作（未来版本）
   - 批量处理

### 内存管理

**内存使用模式**:
- 流式处理避免大文件全量加载
- 及时释放临时数据结构
- 使用 Rust 的 RAII 自动内存管理

**内存监控**:
```rust
// 示例：内存使用监控点
#[cfg(debug_assertions)]
fn log_memory_usage() {
    // 开发版本中的内存使用统计
}
```

## 🔮 扩展性设计

### 插件架构（未来版本）

```rust
// 预留的插件接口设计
pub trait AudioAnalyzer {
    fn analyze(&self, file_path: &Path) -> Result<AnalysisResult, AnalysisError>;
    fn supported_formats(&self) -> &[&str];
}

pub struct LRAAnalyzer;
impl AudioAnalyzer for LRAAnalyzer {
    // LRA 分析实现
}
```

### 配置系统（未来版本）

```rust
// 配置文件支持
#[derive(Deserialize)]
pub struct Config {
    pub output_format: OutputFormat,
    pub parallel_threads: Option<usize>,
    pub ffmpeg_path: Option<PathBuf>,
}
```

## 📊 监控和调试

### 日志系统

```rust
// 使用 log crate 进行结构化日志
log::info!("开始处理 {} 个文件", file_count);
log::debug!("FFmpeg 命令: {:?}", command);
log::warn!("文件 {} 处理失败: {}", file_path, error);
```

### 性能指标

- 处理速度（文件/秒）
- 内存使用峰值
- CPU 利用率
- I/O 吞吐量

---

*本架构文档随项目演进持续更新。如有技术问题或改进建议，请创建 GitHub Issue。*
