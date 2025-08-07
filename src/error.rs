//! 错误处理模块 (Error Handling Module)
//!
//! 本模块定义了应用程序中使用的各种错误类型和相关的处理逻辑。
//! 采用分层错误处理策略，将不同类型的错误进行分类管理，
//! 以提供更好的错误信息和调试体验。
//!
//! ## 设计理念
//!
//! - **错误分类**: 将错误按照来源和性质进行分类（I/O、FFmpeg、路径等）
//! - **上下文保留**: 保留错误发生时的上下文信息，便于调试
//! - **用户友好**: 提供清晰的中文错误信息，帮助用户理解问题
//! - **错误链**: 支持错误链追踪，保留原始错误信息

use std::fmt;

/// 文件处理错误结构体 (File Processing Error)
///
/// 专门用于封装在处理音频文件过程中发生的错误信息。
/// 这种错误通常是可恢复的，即单个文件失败不应影响其他文件的处理。
///
/// ## 使用场景
/// - 音频文件格式不支持或损坏
/// - FFmpeg 分析失败
/// - 文件读取权限问题
/// - LRA 值解析失败
#[derive(Debug, Clone)]
pub struct ProcessFileError {
    /// 出错的文件路径（相对路径，用于显示）
    pub file_path: String,
    /// 错误描述信息（详细的错误原因）
    pub message: String,
    /// 错误类型分类（用于统计和分析）
    pub error_type: FileErrorType,
}

/// 文件处理错误类型分类 (File Error Type Classification)
///
/// 用于对文件处理错误进行分类，便于统计分析和针对性处理
#[derive(Debug, Clone, PartialEq)]
pub enum FileErrorType {
    /// FFmpeg 执行失败（如格式不支持、文件损坏）
    FfmpegExecution,
    /// LRA 值解析失败（FFmpeg 输出格式异常）
    LraParsingFailed,
    /// 文件访问失败（权限、文件不存在等）
    FileAccess,
    /// 其他未分类错误
    Other,
}

impl ProcessFileError {
    /// 创建新的文件处理错误
    ///
    /// # 参数
    /// - `file_path` - 出错的文件路径
    /// - `message` - 错误描述信息
    /// - `error_type` - 错误类型分类
    pub fn new(file_path: String, message: String, error_type: FileErrorType) -> Self {
        Self {
            file_path,
            message,
            error_type,
        }
    }

    /// 创建 FFmpeg 执行错误
    pub fn ffmpeg_error(file_path: String, message: String) -> Self {
        Self::new(file_path, message, FileErrorType::FfmpegExecution)
    }

    /// 创建 LRA 解析错误
    pub fn lra_parsing_error(file_path: String, message: String) -> Self {
        Self::new(file_path, message, FileErrorType::LraParsingFailed)
    }

    /// 创建文件访问错误
    pub fn file_access_error(file_path: String, message: String) -> Self {
        Self::new(file_path, message, FileErrorType::FileAccess)
    }

    /// 获取错误类型的中文描述
    pub fn error_type_description(&self) -> &'static str {
        match self.error_type {
            FileErrorType::FfmpegExecution => "FFmpeg 执行失败",
            FileErrorType::LraParsingFailed => "LRA 值解析失败",
            FileErrorType::FileAccess => "文件访问失败",
            FileErrorType::Other => "其他错误",
        }
    }
}

impl fmt::Display for ProcessFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "文件 '{}' 处理失败 [{}]: {}",
            self.file_path,
            self.error_type_description(),
            self.message
        )
    }
}

impl std::error::Error for ProcessFileError {}

/// 应用程序的主要错误类型 (Main Application Error Types)
///
/// 这是应用程序的顶层错误类型，用于处理不同类别的系统级错误。
/// 与 `ProcessFileError` 不同，这些错误通常是不可恢复的，
/// 会导致程序终止或需要用户干预。
///
/// ## 错误分类说明
/// - `Io`: 系统 I/O 操作失败，如文件读写、网络访问等
/// - `FileProcessing`: 单个文件处理失败的汇总（用于批量处理场景）
/// - `Ffmpeg`: FFmpeg 环境问题，如未安装、版本不兼容等
/// - `Path`: 路径相关问题，如路径不存在、权限不足等
/// - `Configuration`: 配置相关错误，如参数无效、配置文件格式错误等
#[derive(Debug)]
pub enum AppError {
    /// 输入/输出错误 - 系统级 I/O 操作失败
    ///
    /// 包括文件读写失败、权限不足、磁盘空间不足等
    Io(std::io::Error),

    /// 文件处理错误 - 单个文件处理失败
    ///
    /// 用于将文件级错误提升为应用级错误（在某些严格模式下）
    FileProcessing(ProcessFileError),

    /// FFmpeg 相关错误 - FFmpeg 环境或执行问题
    ///
    /// 包括 FFmpeg 未安装、版本不兼容、执行失败等系统级问题
    Ffmpeg(String),

    /// 路径相关错误 - 路径验证和访问问题
    ///
    /// 包括路径不存在、不是目录、权限不足等
    Path(String),

    /// 配置错误 - 程序配置和参数问题
    ///
    /// 包括无效参数、配置文件错误等
    Configuration(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "输入/输出错误: {err}"),
            AppError::FileProcessing(err) => write!(f, "{err}"),
            AppError::Ffmpeg(msg) => write!(f, "FFmpeg 错误: {msg}"),
            AppError::Path(msg) => write!(f, "路径错误: {msg}"),
            AppError::Configuration(msg) => write!(f, "配置错误: {msg}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            AppError::FileProcessing(err) => Some(err),
            AppError::Ffmpeg(_) | AppError::Path(_) | AppError::Configuration(_) => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<ProcessFileError> for AppError {
    fn from(err: ProcessFileError) -> Self {
        AppError::FileProcessing(err)
    }
}
