# API 参考 (API Reference)

本文档提供了 LRA 音频响度范围计算器所有公共 API 的详细参考信息。

## 📋 目录

1. [模块概览](#模块概览)
2. [音频处理模块 (audio.rs)](#音频处理模块)
3. [并行处理模块 (processor.rs)](#并行处理模块)
4. [错误处理模块 (error.rs)](#错误处理模块)
5. [工具模块 (utils.rs)](#工具模块)
6. [数据结构](#数据结构)
7. [常量定义](#常量定义)

## 🏗️ 模块概览

| 模块 | 职责 | 主要类型 |
|------|------|----------|
| `audio` | 音频文件处理和 LRA 计算 | `scan_audio_files`, `calculate_lra_direct` |
| `processor` | 并行处理和进度跟踪 | `process_files_parallel`, `ProcessingStats` |
| `error` | 错误类型定义和处理 | `AppError`, `ProcessFileError` |
| `utils` | 通用工具和辅助功能 | `get_folder_path_from_user`, `sort_lra_results_file` |

## 🎵 音频处理模块 (audio.rs)

### 常量

#### `SUPPORTED_EXTENSIONS`
```rust
pub const SUPPORTED_EXTENSIONS: [&str; 10]
```

**描述**: 支持的音频文件扩展名列表

**值**: `["wav", "mp3", "m4a", "flac", "aac", "ogg", "opus", "wma", "aiff", "alac"]`

**用途**: 文件过滤和格式验证

### 函数

#### `scan_audio_files`
```rust
pub fn scan_audio_files(
    base_path: &Path,
    exclude_file: Option<&Path>,
) -> Vec<(PathBuf, String)>
```

**描述**: 递归扫描指定目录中的音频文件

**参数**:
- `base_path: &Path` - 要扫描的根目录路径
- `exclude_file: Option<&Path>` - 要排除的文件路径（通常是结果文件）

**返回值**: `Vec<(PathBuf, String)>` - 文件路径和显示路径的元组向量
- `PathBuf` - 文件的完整路径
- `String` - 相对于基础路径的显示路径

**示例**:
```rust
use std::path::Path;
use crate::audio::scan_audio_files;

let base_path = Path::new("/Users/username/Music");
let exclude_file = Some(Path::new("/Users/username/Music/lra_results.txt"));
let files = scan_audio_files(base_path, exclude_file);

for (full_path, display_path) in files {
    println!("找到文件: {} ({})", display_path, full_path.display());
}
```

#### `calculate_lra_direct`
```rust
pub fn calculate_lra_direct(
    audio_file_path: &Path,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>>
```

**描述**: 直接计算音频文件的 LRA 值

**参数**:
- `audio_file_path: &Path` - 要分析的音频文件路径

**返回值**: 
- `Ok(f64)` - 计算得到的 LRA 值（单位：LU）
- `Err(Box<dyn std::error::Error + Send + Sync>)` - 分析过程中的错误

**错误情况**:
- FFmpeg 执行失败
- 音频文件格式不支持
- 无法解析 LRA 值
- 文件不存在或无法访问

**示例**:
```rust
use std::path::Path;
use crate::audio::calculate_lra_direct;

let audio_file = Path::new("music/song.mp3");
match calculate_lra_direct(audio_file) {
    Ok(lra) => println!("LRA 值: {:.1} LU", lra),
    Err(e) => eprintln!("分析失败: {}", e),
}
```

#### `check_ffmpeg_availability`
```rust
pub fn check_ffmpeg_availability() -> Result<(), AppError>
```

**描述**: 检查 FFmpeg 是否可用

**返回值**:
- `Ok(())` - FFmpeg 可用
- `Err(AppError::Ffmpeg)` - FFmpeg 不可用或无法运行

**示例**:
```rust
use crate::audio::check_ffmpeg_availability;

match check_ffmpeg_availability() {
    Ok(()) => println!("✓ FFmpeg 检测成功"),
    Err(e) => {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
}
```

## ⚡ 并行处理模块 (processor.rs)

### 函数

#### `process_files_parallel`
```rust
pub fn process_files_parallel(
    files_to_process: Vec<(PathBuf, String)>,
) -> Vec<Result<(String, f64), ProcessFileError>>
```

**描述**: 并行处理音频文件的 LRA 计算

**参数**:
- `files_to_process: Vec<(PathBuf, String)>` - 要处理的文件列表

**返回值**: `Vec<Result<(String, f64), ProcessFileError>>` - 处理结果向量
- `Ok((String, f64))` - 成功：(显示路径, LRA值)
- `Err(ProcessFileError)` - 失败：错误信息

**特性**:
- 自动利用所有可用 CPU 核心
- 实时进度显示
- 线程安全的计数器
- 错误隔离

**示例**:
```rust
use crate::processor::process_files_parallel;

let files = vec![
    (PathBuf::from("song1.mp3"), "song1.mp3".to_string()),
    (PathBuf::from("song2.flac"), "song2.flac".to_string()),
];

let results = process_files_parallel(files);
for result in results {
    match result {
        Ok((path, lra)) => println!("成功: {} - {:.1} LU", path, lra),
        Err(e) => println!("失败: {}", e),
    }
}
```

#### `analyze_results`
```rust
pub fn analyze_results(
    results: Vec<Result<(String, f64), ProcessFileError>>,
) -> (ProcessingStats, Vec<(String, f64)>)
```

**描述**: 分析处理结果并生成统计信息

**参数**:
- `results: Vec<Result<(String, f64), ProcessFileError>>` - 处理结果向量

**返回值**: `(ProcessingStats, Vec<(String, f64)>)`
- `ProcessingStats` - 处理统计信息
- `Vec<(String, f64)>` - 成功的结果列表

**示例**:
```rust
use crate::processor::{process_files_parallel, analyze_results};

let files = scan_audio_files(&base_path, None);
let results = process_files_parallel(files);
let (stats, successful_results) = analyze_results(results);

println!("成功处理: {} 个文件", stats.successful);
println!("处理失败: {} 个文件", stats.failed);
```

#### `display_processing_stats`
```rust
pub fn display_processing_stats(stats: &ProcessingStats)
```

**描述**: 显示处理统计信息

**参数**:
- `stats: &ProcessingStats` - 处理统计信息的引用

**输出**: 向控制台打印格式化的统计信息

**示例**:
```rust
use crate::processor::{analyze_results, display_processing_stats};

let (stats, _) = analyze_results(results);
display_processing_stats(&stats);
// 输出:
// 处理完成！
// 成功处理: 150 个文件
// 处理失败: 2 个文件
// 失败文件详情:
// 文件 'corrupted.mp3': 分析失败: ...
```

## 🛡️ 错误处理模块 (error.rs)

### 错误类型

#### `AppError`
```rust
#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    FileProcessing(ProcessFileError),
    Ffmpeg(String),
    Path(String),
}
```

**描述**: 应用程序的主要错误类型

**变体**:
- `Io(std::io::Error)` - 输入/输出错误
- `FileProcessing(ProcessFileError)` - 文件处理错误
- `Ffmpeg(String)` - FFmpeg 相关错误
- `Path(String)` - 路径相关错误

**实现的 Trait**:
- `Debug` - 调试输出
- `Display` - 用户友好的错误信息
- `std::error::Error` - 标准错误接口

#### `ProcessFileError`
```rust
#[derive(Debug)]
pub struct ProcessFileError {
    pub file_path: String,
    pub message: String,
}
```

**描述**: 文件处理错误结构体

**字段**:
- `file_path: String` - 出错的文件路径
- `message: String` - 错误描述信息

**示例**:
```rust
use crate::error::ProcessFileError;

let error = ProcessFileError {
    file_path: "corrupted.mp3".to_string(),
    message: "无法解析 LRA 值".to_string(),
};

println!("错误: {}", error);
// 输出: 文件 'corrupted.mp3' 处理失败: 无法解析 LRA 值
```

## 🔧 工具模块 (utils.rs)

### 函数

#### `get_folder_path_from_user`
```rust
pub fn get_folder_path_from_user() -> Result<PathBuf, AppError>
```

**描述**: 从用户获取文件夹路径输入

**返回值**:
- `Ok(PathBuf)` - 有效的文件夹路径
- `Err(AppError)` - 输入错误或路径无效

**验证**:
- 检查路径是否存在
- 验证是否为目录
- 处理路径规范化

**示例**:
```rust
use crate::utils::get_folder_path_from_user;

match get_folder_path_from_user() {
    Ok(path) => println!("选择的路径: {}", path.display()),
    Err(e) => eprintln!("路径输入错误: {}", e),
}
```

#### `sort_lra_results_file`
```rust
pub fn sort_lra_results_file(
    file_path: &Path,
    header_line: &str,
) -> Result<(), Box<dyn std::error::Error>>
```

**描述**: 按 LRA 值对结果文件进行排序

**参数**:
- `file_path: &Path` - 结果文件路径
- `header_line: &str` - 文件头部行内容

**返回值**:
- `Ok(())` - 排序成功
- `Err(Box<dyn std::error::Error>)` - 排序失败

**排序规则**: 按 LRA 值从高到低排序

**示例**:
```rust
use std::path::Path;
use crate::utils::sort_lra_results_file;

let results_file = Path::new("lra_results.txt");
let header = "文件路径 (相对) - LRA 数值 (LU)";

match sort_lra_results_file(results_file, header) {
    Ok(()) => println!("结果文件已排序"),
    Err(e) => eprintln!("排序失败: {}", e),
}
```

## 📊 数据结构

### `ProcessingStats`
```rust
pub struct ProcessingStats {
    pub successful: usize,
    pub failed: usize,
    pub error_messages: Vec<String>,
}
```

**描述**: 处理统计信息结构体

**字段**:
- `successful: usize` - 成功处理的文件数量
- `failed: usize` - 处理失败的文件数量
- `error_messages: Vec<String>` - 错误信息列表

## 🔢 常量定义

### 音频格式支持
```rust
pub const SUPPORTED_EXTENSIONS: [&str; 10] = [
    "wav", "mp3", "m4a", "flac", "aac", 
    "ogg", "opus", "wma", "aiff", "alac"
];
```

### 默认配置
```rust
// 结果文件头部
const RESULTS_HEADER: &str = "文件路径 (相对) - LRA 数值 (LU)";

// FFmpeg 命令参数
const FFMPEG_ARGS: &[&str] = &[
    "-filter_complex", "ebur128",
    "-f", "null",
    "-hide_banner",
    "-loglevel", "info"
];
```

## 🔄 类型别名

```rust
// 文件处理结果类型
pub type FileProcessResult = Result<(String, f64), ProcessFileError>;

// 文件列表类型
pub type FileList = Vec<(PathBuf, String)>;

// 处理结果列表类型
pub type ProcessResults = Vec<FileProcessResult>;
```

## 📝 使用示例

### 完整的 API 使用流程

```rust
use std::path::Path;
use crate::{
    audio::{check_ffmpeg_availability, scan_audio_files},
    processor::{process_files_parallel, analyze_results, display_processing_stats},
    utils::{get_folder_path_from_user, sort_lra_results_file},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 检查环境
    check_ffmpeg_availability()?;
    
    // 2. 获取用户输入
    let base_path = get_folder_path_from_user()?;
    
    // 3. 扫描文件
    let files = scan_audio_files(&base_path, None);
    
    // 4. 并行处理
    let results = process_files_parallel(files);
    
    // 5. 分析结果
    let (stats, successful_results) = analyze_results(results);
    
    // 6. 显示统计
    display_processing_stats(&stats);
    
    // 7. 保存和排序结果
    let results_file = base_path.join("lra_results.txt");
    // ... 写入文件逻辑 ...
    sort_lra_results_file(&results_file, "文件路径 (相对) - LRA 数值 (LU)")?;
    
    Ok(())
}
```

---

*本 API 参考文档随代码更新而维护。如有疑问或发现错误，请创建 GitHub Issue。*
