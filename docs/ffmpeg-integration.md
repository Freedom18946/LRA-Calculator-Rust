# FFmpeg 集成 (FFmpeg Integration)

本文档详细介绍了 LRA 计算器与 FFmpeg 的集成方式、技术细节和最佳实践。

## 📋 目录

1. [集成概述](#集成概述)
2. [FFmpeg 要求](#ffmpeg-要求)
3. [命令行接口](#命令行接口)
4. [输出解析](#输出解析)
5. [错误处理](#错误处理)
6. [性能优化](#性能优化)
7. [故障排除](#故障排除)

## 🎯 集成概述

### 为什么选择 FFmpeg？

FFmpeg 是我们选择的音频分析引擎，原因包括：

- **标准兼容**: 完全符合 EBU R128 标准的 ebur128 滤波器
- **格式支持**: 支持几乎所有音频格式
- **性能优异**: 高度优化的 C 代码实现
- **跨平台**: 支持 Windows、macOS、Linux
- **开源免费**: 无许可费用和使用限制
- **持续维护**: 活跃的开发社区和定期更新

### 集成架构

```
Rust 程序 ←→ FFmpeg 进程 ←→ 音频文件
    ↑              ↑
    └── 命令行调用 ──┘
    ↑              ↑
    └── 输出解析 ←──┘
```

## 📦 FFmpeg 要求

### 最低版本要求

- **FFmpeg 版本**: 4.0 或更高
- **编译选项**: 必须包含 `ebur128` 滤波器
- **推荐版本**: 4.4+ 或 5.0+（更好的性能和稳定性）

### 功能检查

程序启动时会自动检查 FFmpeg 的可用性：

```rust
pub fn check_ffmpeg_availability() -> Result<(), AppError> {
    match Command::new("ffmpeg").arg("-version").output() {
        Ok(output) => {
            if output.status.success() {
                // 提取版本信息
                let version_info = extract_ffmpeg_version(&output.stdout);
                println!("✓ FFmpeg 检测成功{}", version_info);
                Ok(())
            } else {
                Err(AppError::Ffmpeg("FFmpeg 存在但无法正常运行".to_string()))
            }
        }
        Err(_) => Err(AppError::Ffmpeg("未找到 FFmpeg".to_string())),
    }
}
```

### 安装验证

```bash
# 检查 FFmpeg 是否安装
ffmpeg -version

# 检查 ebur128 滤波器是否可用
ffmpeg -filters | grep ebur128

# 预期输出
# T.. ebur128          EBU R128 scanner.
```

## 🖥️ 命令行接口

### 基本命令结构

```bash
ffmpeg -i <input_file> -filter_complex ebur128 -f null -hide_banner -loglevel info -
```

### 参数详解

| 参数 | 作用 | 说明 |
|------|------|------|
| `-i <input_file>` | 输入文件 | 指定要分析的音频文件路径 |
| `-filter_complex ebur128` | 滤波器 | 使用 EBU R128 响度分析滤波器 |
| `-f null` | 输出格式 | 不生成实际输出文件，只进行分析 |
| `-hide_banner` | 隐藏横幅 | 减少输出噪音，只显示关键信息 |
| `-loglevel info` | 日志级别 | 确保 ebur128 的分析结果可见 |
| `-` | 输出目标 | 输出到标准输出（实际被丢弃） |

### 高级选项

```bash
# 指定分析时长（仅分析前 60 秒）
ffmpeg -i input.wav -t 60 -filter_complex ebur128 -f null -

# 指定起始时间（从第 30 秒开始分析）
ffmpeg -i input.wav -ss 30 -filter_complex ebur128 -f null -

# 多声道音频处理
ffmpeg -i input.wav -filter_complex "ebur128=peak=true" -f null -
```

## 📊 输出解析

### 标准输出格式

FFmpeg 的 ebur128 滤波器将分析结果输出到 stderr：

```
[Parsed_ebur128_0 @ 0x7f8b8c000000] Summary:
[Parsed_ebur128_0 @ 0x7f8b8c000000] Integrated loudness: -23.0 LUFS
[Parsed_ebur128_0 @ 0x7f8b8c000000] LRA: 12.3 LU
[Parsed_ebur128_0 @ 0x7f8b8c000000] LRA low: -33.2 LUFS
[Parsed_ebur128_0 @ 0x7f8b8c000000] LRA high: -20.9 LUFS
[Parsed_ebur128_0 @ 0x7f8b8c000000] Sample peak: -1.2 dBFS
```

### 解析实现

```rust
fn parse_lra_from_ffmpeg_output(
    ffmpeg_output: &str, 
    file_path: &Path
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    // 编译正则表达式匹配 LRA 值
    let re = Regex::new(r"LRA:\s*([\d\.-]+)\s*LU")?;
    
    // 查找所有匹配项，取最后一个（最终结果）
    if let Some(caps) = re.captures_iter(ffmpeg_output).last() {
        if let Some(lra_match) = caps.get(1) {
            let lra_str = lra_match.as_str();
            return lra_str.parse::<f64>().map_err(|e| {
                format!("解析 LRA 值 '{}' 失败: {}", lra_str, e).into()
            });
        }
    }
    
    Err(format!("无法从 FFmpeg 输出中解析 LRA 值").into())
}
```

### 正则表达式模式

- **模式**: `LRA:\s*([\d\.-]+)\s*LU`
- **解释**: 
  - `LRA:` - 字面匹配 "LRA:"
  - `\s*` - 可选的空白字符
  - `([\d\.-]+)` - 捕获组：数字、点、负号
  - `\s*LU` - 可选空白 + "LU"

## ⚠️ 错误处理

### 常见错误类型

#### 1. FFmpeg 执行失败
```rust
// 错误示例
"FFmpeg 分析文件 test.mp3 失败 (退出码: 1)"

// 可能原因
- 文件格式不支持
- 文件损坏或不完整
- 权限不足
- FFmpeg 版本过旧
```

#### 2. LRA 值解析失败
```rust
// 错误示例
"无法从 FFmpeg 输出中解析文件 test.wav 的 LRA 值"

// 可能原因
- 音频时长过短（< 3 秒）
- 音频内容异常（全静音）
- FFmpeg 输出格式变化
- 编码问题
```

#### 3. 文件访问错误
```rust
// 错误示例
"执行 FFmpeg 命令失败: 权限被拒绝"

// 可能原因
- 文件被其他程序占用
- 网络存储连接问题
- 文件系统权限限制
```

### 错误恢复策略

```rust
pub fn calculate_lra_with_retry(
    audio_file_path: &Path,
    max_retries: usize
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match calculate_lra_direct(audio_file_path) {
            Ok(lra) => return Ok(lra),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    // 短暂等待后重试
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

## 🚀 性能优化

### 并行处理优化

```rust
// 避免 FFmpeg 进程过多
const MAX_CONCURRENT_FFMPEG: usize = num_cpus::get();

// 使用信号量控制并发
let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_FFMPEG));

files.par_iter().map(|file| {
    let _permit = semaphore.acquire();
    calculate_lra_direct(file)
}).collect()
```

### 内存使用优化

- **流式处理**: FFmpeg 使用流式处理，内存使用稳定
- **输出缓冲**: 限制 stderr 输出缓冲区大小
- **进程清理**: 确保 FFmpeg 进程正确终止

### I/O 优化

```rust
// 使用 BufReader 处理大量输出
let mut reader = BufReader::new(process.stderr.take().unwrap());
let mut output = String::new();
reader.read_to_string(&mut output)?;
```

## 🔧 故障排除

### 诊断步骤

1. **验证 FFmpeg 安装**
   ```bash
   which ffmpeg
   ffmpeg -version
   ```

2. **测试 ebur128 滤波器**
   ```bash
   ffmpeg -f lavfi -i "sine=frequency=1000:duration=5" -filter_complex ebur128 -f null -
   ```

3. **检查文件权限**
   ```bash
   ls -la /path/to/audio/file
   ```

4. **手动运行 FFmpeg 命令**
   ```bash
   ffmpeg -i test.mp3 -filter_complex ebur128 -f null - 2>&1 | grep LRA
   ```

### 常见问题解决

#### 问题：FFmpeg 未找到
```bash
# 解决方案 1: 添加到 PATH
export PATH="/usr/local/bin:$PATH"

# 解决方案 2: 创建符号链接
sudo ln -s /opt/ffmpeg/bin/ffmpeg /usr/local/bin/ffmpeg

# 解决方案 3: 指定完整路径
FFMPEG_PATH="/opt/ffmpeg/bin/ffmpeg" cargo run
```

#### 问题：权限被拒绝
```bash
# 检查文件权限
ls -la audio_file.mp3

# 修复权限
chmod 644 audio_file.mp3

# 检查目录权限
chmod 755 /path/to/audio/directory
```

#### 问题：输出解析失败
```bash
# 调试 FFmpeg 输出
ffmpeg -i test.mp3 -filter_complex ebur128 -f null - 2> debug.log
cat debug.log | grep -E "(LRA|Summary|Error)"
```

### 调试模式

```rust
#[cfg(debug_assertions)]
fn debug_ffmpeg_output(output: &str, file_path: &Path) {
    eprintln!("=== FFmpeg Debug Output for {} ===", file_path.display());
    eprintln!("{}", output);
    eprintln!("=== End Debug Output ===");
}
```

## 📈 监控和日志

### 性能监控

```rust
use std::time::Instant;

let start = Instant::now();
let lra = calculate_lra_direct(file_path)?;
let duration = start.elapsed();

if duration > Duration::from_secs(30) {
    eprintln!("警告: 文件 {} 处理时间过长: {:?}", 
              file_path.display(), duration);
}
```

### 错误统计

```rust
struct FfmpegStats {
    total_calls: usize,
    successful_calls: usize,
    failed_calls: usize,
    average_duration: Duration,
}
```

## 🔮 未来改进

### 计划中的优化

1. **FFmpeg 库集成**: 直接使用 libavformat/libavcodec
2. **缓存机制**: 缓存已分析文件的结果
3. **增量分析**: 只分析文件的变化部分
4. **自适应并发**: 根据系统负载调整并发数

### 实验性功能

- **GPU 加速**: 使用 FFmpeg 的 GPU 滤波器
- **网络分析**: 支持网络音频流分析
- **实时分析**: 支持实时音频流 LRA 计算

---

*本文档基于 FFmpeg 4.4+ 版本编写。不同版本的 FFmpeg 可能有细微差异。*
