//! 音频处理模块 (Audio Processing Module)
//!
//! 本模块是 LRA 计算器的核心，负责音频文件的发现、验证和分析。
//! 主要功能包括递归目录扫描、音频格式识别、FFmpeg 集成和 LRA 值计算。
//!
//! ## 核心功能
//!
//! - **文件扫描**: 递归遍历目录树，识别支持的音频文件格式
//! - **格式支持**: 支持主流的无损和有损音频格式
//! - **LRA 计算**: 基于 FFmpeg 的 ebur128 滤波器进行精确计算
//! - **环境检查**: 验证 FFmpeg 的可用性和版本兼容性
//!
//! ## 设计原则
//!
//! - **性能优先**: 使用高效的文件遍历和正则表达式匹配
//! - **错误隔离**: 单个文件的错误不影响整体处理流程
//! - **标准兼容**: 严格遵循 EBU R128 标准进行 LRA 计算
//! - **扩展性**: 易于添加新的音频格式支持

use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;
use walkdir::WalkDir;

use crate::error::AppError;

/// 支持的音频文件扩展名列表 (Supported Audio File Extensions)
///
/// 本列表包含了程序支持的所有音频格式，分为无损和有损两大类。
/// 选择这些格式是基于以下考虑：
///
/// ## 无损格式 (Lossless Formats)
/// - `wav`: PCM 未压缩格式，音质最高，广泛支持
/// - `flac`: 开源无损压缩，压缩率好，元数据丰富
/// - `aiff`: 苹果音频格式，Mac 平台常用
/// - `alac`: 苹果无损压缩，iTunes 生态系统标准
///
/// ## 有损格式 (Lossy Formats)
/// - `mp3`: 最普及的音频格式，兼容性最好
/// - `m4a`/`aac`: 高效音频编码，移动设备和流媒体首选
/// - `ogg`: 开源音频格式，游戏和开源软件常用
/// - `opus`: 现代低延迟编解码器，语音和音乐兼顾
/// - `wma`: Windows 媒体音频，Windows 平台常用
///
/// ## 注意事项
/// - 所有扩展名都使用小写形式进行匹配
/// - 文件扫描时会自动转换为小写进行比较
/// - 添加新格式时需要确保 FFmpeg 支持该格式的 LRA 分析
pub const SUPPORTED_EXTENSIONS: [&str; 10] = [
    "wav", "mp3", "m4a", "flac", "aac", "ogg", "opus", "wma", "aiff", "alac",
];

/// 扫描指定目录中的音频文件 (Scan Audio Files in Directory)
///
/// 递归遍历指定目录及其所有子目录，查找所有支持格式的音频文件。
/// 这个函数是整个处理流程的起点，负责构建待处理文件的完整列表。
///
/// ## 扫描策略
/// - **递归遍历**: 使用 `walkdir` 库进行深度优先遍历
/// - **格式过滤**: 只保留扩展名在支持列表中的文件
/// - **路径处理**: 生成相对路径用于显示，保留绝对路径用于处理
/// - **排除机制**: 可以排除特定文件（如结果文件）避免重复处理
///
/// ## 性能考虑
/// - 使用迭代器链式操作，避免中间集合的创建
/// - 文件类型检查在遍历过程中进行，减少系统调用
/// - 扩展名比较使用小写转换，确保大小写不敏感
///
/// # 参数
/// - `base_path` - 要扫描的根目录路径
/// - `exclude_file` - 要排除的文件路径（通常是结果文件，避免处理自己）
///
/// # 返回值
/// 返回包含文件信息的元组向量：
/// - `PathBuf` - 文件的完整绝对路径（用于实际处理）
/// - `String` - 相对于基础路径的显示路径（用于用户界面）
///
/// # 示例
/// ```rust
/// use std::path::Path;
/// let files = scan_audio_files(Path::new("/music"), None);
/// for (full_path, display_path) in files {
///     println!("发现文件: {} -> {}", display_path, full_path.display());
/// }
/// ```
pub fn scan_audio_files(
    base_path: &Path,
    exclude_file: Option<&Path>,
) -> Vec<(PathBuf, String)> {
    let mut files_to_process = Vec::new();

    // 使用 WalkDir 进行递归目录遍历
    // 这里使用函数式编程风格，通过链式调用提高代码可读性
    for entry_result in WalkDir::new(base_path)
        .into_iter()
        .filter_map(Result::ok)  // 忽略无法访问的目录项（权限问题等）
        .filter(|e| e.file_type().is_file())  // 只处理文件，跳过目录和符号链接
    {
        let current_file_path = entry_result.path().to_path_buf();

        // 排除指定文件（通常是结果文件，避免处理自己生成的文件）
        if let Some(exclude) = exclude_file {
            if current_file_path == exclude {
                continue;
            }
        }

        // 检查文件扩展名是否在支持列表中
        // 使用 Option 链式调用优雅地处理可能的 None 值
        if let Some(extension) = extract_file_extension(&current_file_path) {
            if is_supported_audio_format(&extension) {
                // 生成用户友好的相对路径显示
                let display_path_str = generate_display_path(&current_file_path, base_path);
                files_to_process.push((current_file_path, display_path_str));
            }
        }
    }

    files_to_process
}

/// 提取文件扩展名并转换为小写 (Extract File Extension in Lowercase)
///
/// 这是一个辅助函数，用于安全地提取文件扩展名并转换为小写。
/// 分离这个逻辑可以提高代码的可测试性和可读性。
///
/// # 参数
/// - `file_path` - 文件路径
///
/// # 返回值
/// - `Some(String)` - 小写的文件扩展名
/// - `None` - 文件没有扩展名或扩展名包含非 UTF-8 字符
pub fn extract_file_extension(file_path: &Path) -> Option<String> {
    file_path
        .extension()
        .and_then(|ext| ext.to_str())  // 转换为字符串，处理非 UTF-8 情况
        .map(|s| s.to_lowercase())     // 转换为小写，确保大小写不敏感匹配
}

/// 检查是否为支持的音频格式 (Check if Supported Audio Format)
///
/// 检查给定的扩展名是否在支持的音频格式列表中。
/// 这个函数封装了格式检查逻辑，便于未来扩展和修改。
///
/// # 参数
/// - `extension` - 文件扩展名（应该已经是小写）
///
/// # 返回值
/// - `true` - 支持的音频格式
/// - `false` - 不支持的格式
pub fn is_supported_audio_format(extension: &str) -> bool {
    SUPPORTED_EXTENSIONS.contains(&extension)
}

/// 生成用于显示的相对路径 (Generate Display Path)
///
/// 生成相对于基础路径的显示路径，用于用户界面显示。
/// 如果无法生成相对路径（理论上不应该发生），则使用完整路径。
///
/// # 参数
/// - `file_path` - 文件的完整路径
/// - `base_path` - 基础路径
///
/// # 返回值
/// - 相对路径的字符串表示
fn generate_display_path(file_path: &Path, base_path: &Path) -> String {
    file_path
        .strip_prefix(base_path)
        .unwrap_or(file_path)  // 如果无法生成相对路径，使用完整路径
        .to_string_lossy()     // 处理非 UTF-8 路径字符
        .into_owned()          // 转换为拥有的字符串
}

/// 直接计算音频文件的 LRA 值 (Calculate LRA Value Directly)
///
/// 这是程序的核心函数，使用 FFmpeg 的 ebur128 滤波器直接分析音频文件，
/// 计算符合 EBU R128 标准的响度范围（Loudness Range）值。
///
/// ## 技术实现
///
/// ### FFmpeg 命令构建
/// 使用以下 FFmpeg 命令进行分析：
/// ```bash
/// ffmpeg -i <input_file> -filter_complex ebur128 -f null -hide_banner -loglevel info -
/// ```
///
/// ### 参数说明
/// - `-i <input_file>`: 指定输入音频文件
/// - `-filter_complex ebur128`: 使用 EBU R128 标准的响度分析滤波器
/// - `-f null`: 输出格式为 null（不生成实际输出文件）
/// - `-hide_banner`: 隐藏 FFmpeg 版本信息，减少输出噪音
/// - `-loglevel info`: 设置日志级别为 info，确保 ebur128 输出可见
/// - `-`: 输出到标准输出（实际上被丢弃）
///
/// ### LRA 值解析
/// ebur128 滤波器会在 stderr 中输出分析结果，格式类似：
/// ```
/// [Parsed_ebur128_0 @ 0x...] Summary:
/// [Parsed_ebur128_0 @ 0x...] Integrated loudness: -23.0 LUFS
/// [Parsed_ebur128_0 @ 0x...] LRA: 12.3 LU
/// ```
///
/// 我们使用正则表达式提取 "LRA: X.X LU" 中的数值部分。
///
/// # 参数
/// - `audio_file_path` - 要分析的音频文件路径
///
/// # 返回值
/// - `Ok(f64)` - 计算得到的 LRA 值（单位：LU，Loudness Units）
/// - `Err(Box<dyn std::error::Error + Send + Sync>)` - 分析过程中的错误
///
/// # 错误情况
/// - FFmpeg 执行失败（文件不存在、格式不支持、权限问题等）
/// - 音频文件损坏或格式异常
/// - FFmpeg 输出中无法找到 LRA 值
/// - LRA 值解析失败（非数字格式）
///
/// # 性能注意事项
/// - 这个函数会阻塞直到 FFmpeg 分析完成
/// - 分析时间取决于音频文件的长度和复杂度
/// - 内存使用量相对较小，因为使用流式处理
pub fn calculate_lra_direct(
    audio_file_path: &Path,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    // 构建并执行 FFmpeg 命令
    // 使用 Command::new 创建子进程，避免 shell 注入攻击
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(audio_file_path)           // 输入文件路径
        .arg("-filter_complex")
        .arg("ebur128")                 // EBU R128 响度分析滤波器
        .arg("-f")
        .arg("null")                    // 输出格式为 null，不生成文件
        .arg("-hide_banner")            // 隐藏版本信息，减少输出噪音
        .arg("-loglevel")
        .arg("info")                    // ebur128 的输出在 info 级别
        .arg("-")                       // 输出到标准输出（被丢弃）
        .output()                       // 执行命令并等待完成
        .map_err(|e| {
            format!(
                "执行 FFmpeg 命令失败 (文件: {}): {}. 请确保 FFmpeg 已正确安装。",
                audio_file_path.display(),
                e
            )
        })?;

    // 检查 FFmpeg 命令是否成功执行
    if !output.status.success() {
        let stderr_preview = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "FFmpeg 分析文件 {} 失败 (退出码: {}). 错误信息: {}",
            audio_file_path.display(),
            output.status.code().unwrap_or(-1),
            stderr_preview.lines().take(3).collect::<Vec<_>>().join("; ")
        ).into());
    }

    // 从 stderr 中提取 LRA 值
    // FFmpeg 的 ebur128 滤波器将分析结果输出到 stderr
    let stderr_output = String::from_utf8_lossy(&output.stderr);

    // 解析 LRA 值
    parse_lra_from_ffmpeg_output(&stderr_output, audio_file_path)
}

/// 从 FFmpeg 输出中解析 LRA 值 (Parse LRA Value from FFmpeg Output)
///
/// 使用正则表达式从 FFmpeg 的 ebur128 滤波器输出中提取 LRA 值。
/// 分离这个逻辑可以提高代码的可测试性和可维护性。
///
/// ## 解析策略
/// - 使用正则表达式匹配 "LRA: X.X LU" 模式
/// - 支持整数和浮点数格式
/// - 支持负数（虽然 LRA 通常为正数）
/// - 使用 `captures_iter().last()` 获取最后一个匹配（最终结果）
///
/// # 参数
/// - `ffmpeg_output` - FFmpeg 的 stderr 输出
/// - `file_path` - 文件路径（用于错误信息）
///
/// # 返回值
/// - `Ok(f64)` - 解析得到的 LRA 值
/// - `Err(...)` - 解析失败的错误
fn parse_lra_from_ffmpeg_output(
    ffmpeg_output: &str,
    file_path: &Path
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    // 编译正则表达式匹配 LRA 值
    // 模式说明: LRA: 后跟可选空白，然后是数字（可能包含小数点和负号），最后是 LU
    let re = Regex::new(r"LRA:\s*([\d\.-]+)\s*LU")
        .map_err(|e| format!("正则表达式编译失败: {}", e))?;

    // 查找所有匹配项，取最后一个（通常是最终的汇总结果）
    if let Some(caps) = re.captures_iter(ffmpeg_output).last() {
        if let Some(lra_match) = caps.get(1) {
            let lra_str = lra_match.as_str();
            return lra_str.parse::<f64>().map_err(|e| {
                format!(
                    "解析 LRA 值 '{}' 失败 (来自文件 {}): {}",
                    lra_str,
                    file_path.display(),
                    e
                ).into()
            });
        }
    }

    // 如果没有找到 LRA 值，提供详细的错误信息
    Err(format!(
        "无法从 FFmpeg 输出中解析文件 {} 的 LRA 值。\n\
         这可能是因为：\n\
         1. 音频文件格式不支持或已损坏\n\
         2. 音频文件时长过短（需要至少几秒钟）\n\
         3. FFmpeg 版本不兼容\n\
         \n\
         FFmpeg 输出摘要: {}",
        file_path.display(),
        ffmpeg_output.lines()
            .filter(|line| !line.trim().is_empty())
            .take(5)
            .collect::<Vec<_>>()
            .join("; ")
    ).into())
}

/// 验证 FFmpeg 是否可用 (Verify FFmpeg Availability)
///
/// 这个函数在程序启动时被调用，用于验证 FFmpeg 是否正确安装并可用。
/// 这是一个关键的环境检查，因为整个 LRA 计算依赖于 FFmpeg。
///
/// ## 检查策略
/// 1. **存在性检查**: 尝试执行 `ffmpeg -version` 命令
/// 2. **功能性检查**: 验证命令是否成功执行
/// 3. **版本信息**: 可选地提取版本信息用于兼容性检查
///
/// ## 错误处理
/// - 如果 FFmpeg 不存在，返回安装指导信息
/// - 如果 FFmpeg 存在但无法运行，返回权限或损坏提示
/// - 成功时显示确认信息并返回 Ok
///
/// # 返回值
/// - `Ok(())` - FFmpeg 可用且功能正常
/// - `Err(AppError::Ffmpeg)` - FFmpeg 不可用或存在问题
///
/// # 使用场景
/// 通常在 main 函数开始时调用，如果失败则终止程序执行
pub fn check_ffmpeg_availability() -> Result<(), AppError> {
    match Command::new("ffmpeg").arg("-version").output() {
        Ok(output) => {
            if output.status.success() {
                // 可选：提取版本信息进行更详细的检查
                let version_info = extract_ffmpeg_version(&output.stdout);
                println!("✓ FFmpeg 检测成功{}", version_info);
                Ok(())
            } else {
                Err(AppError::Ffmpeg(
                    "FFmpeg 存在但无法正常运行。可能的原因：\n\
                     1. FFmpeg 文件损坏或不完整\n\
                     2. 缺少必要的系统依赖库\n\
                     3. 权限不足\n\
                     请尝试重新安装 FFmpeg。".to_string(),
                ))
            }
        }
        Err(_) => Err(AppError::Ffmpeg(
            "未找到 FFmpeg，请确保已安装并添加到 PATH 环境变量中。\n\
             \n\
             安装方法：\n\
             • macOS: brew install ffmpeg\n\
             • Ubuntu/Debian: sudo apt install ffmpeg\n\
             • Windows: choco install ffmpeg 或从官网下载\n\
             • 其他系统: 请访问 https://ffmpeg.org/download.html".to_string(),
        )),
    }
}

/// 从 FFmpeg 输出中提取版本信息 (Extract FFmpeg Version Information)
///
/// 解析 FFmpeg 版本输出，提取有用的版本信息用于显示。
/// 这有助于调试兼容性问题。
///
/// # 参数
/// - `version_output` - FFmpeg -version 命令的输出
///
/// # 返回值
/// - 格式化的版本信息字符串，如果解析失败则返回空字符串
fn extract_ffmpeg_version(version_output: &[u8]) -> String {
    let output_str = String::from_utf8_lossy(version_output);

    // 查找版本行，通常格式为 "ffmpeg version X.X.X ..."
    if let Some(first_line) = output_str.lines().next() {
        if first_line.starts_with("ffmpeg version") {
            // 提取版本号部分
            if let Some(version_part) = first_line.split_whitespace().nth(2) {
                return format!(" (版本: {})", version_part);
            }
        }
    }

    String::new()  // 如果无法解析版本信息，返回空字符串
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    /// 测试支持的音频格式常量
    #[test]
    fn test_supported_extensions() {
        // 验证常见格式都在支持列表中
        assert!(SUPPORTED_EXTENSIONS.contains(&"mp3"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"wav"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"flac"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"m4a"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"aac"));

        // 验证不支持的格式不在列表中
        assert!(!SUPPORTED_EXTENSIONS.contains(&"txt"));
        assert!(!SUPPORTED_EXTENSIONS.contains(&"doc"));
        assert!(!SUPPORTED_EXTENSIONS.contains(&"pdf"));

        // 验证列表长度符合预期
        assert_eq!(SUPPORTED_EXTENSIONS.len(), 10);
    }

    /// 测试文件扩展名提取功能
    #[test]
    fn test_extract_file_extension() {
        // 测试正常的文件扩展名
        assert_eq!(
            extract_file_extension(Path::new("test.mp3")),
            Some("mp3".to_string())
        );
        assert_eq!(
            extract_file_extension(Path::new("music.FLAC")),
            Some("flac".to_string())
        );

        // 测试没有扩展名的文件
        assert_eq!(
            extract_file_extension(Path::new("filename")),
            None
        );

        // 测试隐藏文件
        assert_eq!(
            extract_file_extension(Path::new(".hidden")),
            None
        );

        // 测试多个点的文件名
        assert_eq!(
            extract_file_extension(Path::new("file.name.mp3")),
            Some("mp3".to_string())
        );
    }

    /// 测试音频格式支持检查
    #[test]
    fn test_is_supported_audio_format() {
        // 测试支持的格式
        assert!(is_supported_audio_format("mp3"));
        assert!(is_supported_audio_format("wav"));
        assert!(is_supported_audio_format("flac"));

        // 测试不支持的格式
        assert!(!is_supported_audio_format("txt"));
        assert!(!is_supported_audio_format("doc"));
        assert!(!is_supported_audio_format(""));
    }

    /// 测试显示路径生成功能
    #[test]
    fn test_generate_display_path() {
        let base_path = Path::new("/music/library");
        let file_path = Path::new("/music/library/artist/album/song.mp3");

        let display_path = generate_display_path(file_path, base_path);
        assert_eq!(display_path, "artist/album/song.mp3");

        // 测试无法生成相对路径的情况
        let unrelated_path = Path::new("/other/path/file.mp3");
        let display_path2 = generate_display_path(unrelated_path, base_path);
        assert_eq!(display_path2, "/other/path/file.mp3");
    }

    /// 测试音频文件扫描功能
    #[test]
    fn test_scan_audio_files() {
        let temp_dir = TempDir::new().expect("无法创建临时目录");
        let temp_path = temp_dir.path();

        // 创建测试目录结构
        let subdir1 = temp_path.join("subdir1");
        let subdir2 = temp_path.join("subdir2");
        fs::create_dir(&subdir1).expect("无法创建子目录1");
        fs::create_dir(&subdir2).expect("无法创建子目录2");

        // 创建测试文件
        let test_files = vec![
            temp_path.join("root.mp3"),
            temp_path.join("root.wav"),
            subdir1.join("sub1.flac"),
            subdir2.join("sub2.m4a"),
            temp_path.join("not_audio.txt"),  // 非音频文件
            temp_path.join("README.md"),      // 非音频文件
        ];

        for file_path in &test_files {
            File::create(file_path).expect("无法创建测试文件");
        }

        // 扫描音频文件
        let found_files = scan_audio_files(temp_path, None);

        // 验证结果
        assert_eq!(found_files.len(), 4); // 应该找到 4 个音频文件

        // 验证找到的文件路径
        let found_paths: Vec<String> = found_files.iter()
            .map(|(_, display_path)| display_path.clone())
            .collect();

        assert!(found_paths.iter().any(|p| p.contains("root.mp3")));
        assert!(found_paths.iter().any(|p| p.contains("root.wav")));
        assert!(found_paths.iter().any(|p| p.contains("sub1.flac")));
        assert!(found_paths.iter().any(|p| p.contains("sub2.m4a")));

        // 确保非音频文件被排除
        assert!(!found_paths.iter().any(|p| p.contains("not_audio.txt")));
        assert!(!found_paths.iter().any(|p| p.contains("README.md")));
    }

    /// 测试文件排除功能
    #[test]
    fn test_scan_with_exclusion() {
        let temp_dir = TempDir::new().expect("无法创建临时目录");
        let temp_path = temp_dir.path();

        // 创建测试文件
        let audio_file1 = temp_path.join("audio1.mp3");
        let audio_file2 = temp_path.join("audio2.wav");
        let exclude_file = temp_path.join("exclude.mp3");

        File::create(&audio_file1).expect("无法创建音频文件1");
        File::create(&audio_file2).expect("无法创建音频文件2");
        File::create(&exclude_file).expect("无法创建排除文件");

        // 不排除任何文件
        let files_no_exclusion = scan_audio_files(temp_path, None);
        assert_eq!(files_no_exclusion.len(), 3);

        // 排除指定文件
        let files_with_exclusion = scan_audio_files(temp_path, Some(&exclude_file));
        assert_eq!(files_with_exclusion.len(), 2);

        // 验证排除的文件确实不在结果中
        let found_paths: Vec<String> = files_with_exclusion.iter()
            .map(|(_, display_path)| display_path.clone())
            .collect();

        assert!(!found_paths.iter().any(|p| p.contains("exclude.mp3")));
        assert!(found_paths.iter().any(|p| p.contains("audio1.mp3")));
        assert!(found_paths.iter().any(|p| p.contains("audio2.wav")));
    }

    /// 测试 FFmpeg 版本信息提取
    #[test]
    fn test_extract_ffmpeg_version() {
        // 测试正常的 FFmpeg 版本输出
        let version_output = b"ffmpeg version 4.4.2 Copyright (c) 2000-2021 the FFmpeg developers\n";
        let version_info = extract_ffmpeg_version(version_output);
        assert_eq!(version_info, " (版本: 4.4.2)");

        // 测试异常的版本输出
        let invalid_output = b"some other output\n";
        let version_info2 = extract_ffmpeg_version(invalid_output);
        assert_eq!(version_info2, "");

        // 测试空输出
        let empty_output = b"";
        let version_info3 = extract_ffmpeg_version(empty_output);
        assert_eq!(version_info3, "");
    }

    /// 测试 LRA 值解析功能
    #[test]
    fn test_parse_lra_from_ffmpeg_output() {
        let test_path = Path::new("test.mp3");

        // 测试正常的 FFmpeg 输出
        let normal_output = r#"
[Parsed_ebur128_0 @ 0x7f8b8c000000] Summary:
[Parsed_ebur128_0 @ 0x7f8b8c000000] Integrated loudness: -23.0 LUFS
[Parsed_ebur128_0 @ 0x7f8b8c000000] LRA: 12.3 LU
[Parsed_ebur128_0 @ 0x7f8b8c000000] LRA low: -33.2 LUFS
"#;

        let result = parse_lra_from_ffmpeg_output(normal_output, test_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 12.3);

        // 测试没有 LRA 值的输出
        let no_lra_output = r#"
[Parsed_ebur128_0 @ 0x7f8b8c000000] Summary:
[Parsed_ebur128_0 @ 0x7f8b8c000000] Integrated loudness: -23.0 LUFS
"#;

        let result2 = parse_lra_from_ffmpeg_output(no_lra_output, test_path);
        assert!(result2.is_err());

        // 测试多个 LRA 值（应该取最后一个）
        let multiple_lra_output = r#"
[Parsed_ebur128_0 @ 0x7f8b8c000000] LRA: 10.5 LU
[Parsed_ebur128_0 @ 0x7f8b8c000000] Summary:
[Parsed_ebur128_0 @ 0x7f8b8c000000] LRA: 15.7 LU
"#;

        let result3 = parse_lra_from_ffmpeg_output(multiple_lra_output, test_path);
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), 15.7);
    }
}
