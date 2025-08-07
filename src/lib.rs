//! LRA 音频响度范围计算器库 (LRA Audio Loudness Range Calculator Library)
//! 
//! 这是一个基于 Rust 的高性能音频响度范围（LRA）计算库，
//! 使用 FFmpeg 的 ebur128 滤波器进行精确的 EBU R128 标准分析。
//! 
//! ## 核心功能
//! 
//! - **音频文件扫描**: 递归扫描目录，识别支持的音频格式
//! - **并行处理**: 利用多核 CPU 进行高效的并行 LRA 计算
//! - **格式支持**: 支持 WAV、MP3、FLAC、AAC、OGG 等主流音频格式
//! - **错误处理**: 完善的错误分类和处理机制
//! - **结果管理**: 自动排序和格式化输出结果
//! 
//! ## 使用示例
//! 
//! ```rust,no_run
//! use lra_calculator_rust::audio::{scan_audio_files, check_ffmpeg_availability};
//! use lra_calculator_rust::processor::process_files_parallel;
//! use std::path::Path;
//! 
//! // 检查 FFmpeg 环境
//! check_ffmpeg_availability().expect("FFmpeg 不可用");
//! 
//! // 扫描音频文件
//! let audio_path = Path::new("/path/to/audio/files");
//! let files = scan_audio_files(audio_path, None);
//! 
//! // 并行处理
//! let results = process_files_parallel(files);
//! println!("处理了 {} 个文件", results.len());
//! ```
//! 
//! ## 模块结构
//! 
//! - [`audio`] - 音频文件处理和 LRA 计算核心功能
//! - [`processor`] - 并行处理和进度跟踪
//! - [`error`] - 错误类型定义和处理
//! - [`utils`] - 通用工具函数和辅助功能

pub mod audio;
pub mod error;
pub mod processor;
pub mod utils;

// 重新导出常用类型和函数，方便使用
pub use audio::{
    scan_audio_files, calculate_lra_direct, check_ffmpeg_availability,
    extract_file_extension, is_supported_audio_format, SUPPORTED_EXTENSIONS
};
pub use processor::{process_files_parallel, analyze_results, display_processing_stats, ProcessingStats};
pub use error::{AppError, ProcessFileError, FileErrorType};
pub use utils::{
    validate_folder_path, sort_lra_results_file, get_folder_path_from_user,
    parse_result_line, sort_entries_by_lra
};

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 获取库的完整版本信息
/// 
/// # 返回值
/// 包含库名称和版本号的字符串
/// 
/// # 示例
/// ```
/// use lra_calculator_rust::get_version_info;
/// println!("{}", get_version_info());
/// ```
pub fn get_version_info() -> String {
    format!("{} v{}", NAME, VERSION)
}

/// 检查库的运行环境
/// 
/// 验证所有必要的依赖和环境配置是否正确。
/// 这是一个便利函数，组合了多个环境检查。
/// 
/// # 返回值
/// - `Ok(())` - 环境检查通过
/// - `Err(AppError)` - 环境检查失败
/// 
/// # 示例
/// ```rust,no_run
/// use lra_calculator_rust::check_environment;
/// 
/// match check_environment() {
///     Ok(()) => println!("环境检查通过"),
///     Err(e) => eprintln!("环境检查失败: {}", e),
/// }
/// ```
pub fn check_environment() -> Result<(), AppError> {
    // 检查 FFmpeg 可用性
    check_ffmpeg_availability()?;
    
    // 未来可以添加更多环境检查
    // 例如：检查系统资源、权限等
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let version_info = get_version_info();
        assert!(version_info.contains("LRA-Calculator-Rust"));
        assert!(version_info.contains("v"));
    }

    #[test]
    fn test_constants() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
        assert_eq!(NAME, "LRA-Calculator-Rust");
    }

    #[test]
    fn test_supported_extensions_export() {
        // 测试重新导出的常量是否可用
        assert!(SUPPORTED_EXTENSIONS.contains(&"mp3"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"wav"));
        assert_eq!(SUPPORTED_EXTENSIONS.len(), 10);
    }

    #[test]
    fn test_environment_check() {
        // 这个测试依赖于 FFmpeg 的可用性
        // 在 CI 环境中可能需要跳过
        match check_environment() {
            Ok(()) => println!("✅ 环境检查通过"),
            Err(e) => println!("⚠️ 环境检查失败: {} (这在没有 FFmpeg 的环境中是正常的)", e),
        }
    }
}
