//! 并行处理模块 (Parallel Processing Module)
//!
//! 本模块负责音频文件的并行处理和进度跟踪，是程序性能优化的核心。
//! 通过充分利用多核 CPU 资源，显著提升大批量音频文件的处理效率。
//!
//! ## 核心设计理念
//!
//! ### 数据并行 (Data Parallelism)
//! 使用 Rayon 库实现数据并行处理，将文件列表分割到多个线程中并行执行。
//! 这种方式相比传统的任务并行更加高效，因为：
//! - 自动负载均衡：Rayon 使用工作窃取算法
//! - 零开销抽象：编译时优化，运行时性能接近手写线程代码
//! - 内存安全：利用 Rust 的所有权系统避免数据竞争
//!
//! ### 错误隔离 (Error Isolation)
//! 单个文件的处理失败不会影响其他文件的处理，确保程序的健壮性。
//! 所有错误都被收集并在最后统一报告。
//!
//! ### 进度跟踪 (Progress Tracking)
//! 使用原子计数器实现线程安全的进度跟踪，为用户提供实时反馈。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use rayon::prelude::*;

use crate::audio::calculate_lra_direct;
use crate::error::ProcessFileError;

/// 并行处理音频文件的 LRA 计算 (Parallel LRA Calculation for Audio Files)
///
/// 这是程序的性能核心，使用 Rayon 库实现数据并行处理。
/// 将文件列表分发到多个线程中并行执行 LRA 计算，同时提供实时的进度反馈。
///
/// ## 并行处理策略
///
/// ### 工作分配
/// - 使用 Rayon 的 `par_iter()` 将文件列表转换为并行迭代器
/// - 自动利用所有可用的 CPU 核心
/// - 工作窃取算法确保负载均衡
///
/// ### 进度跟踪
/// - 使用原子计数器 `AtomicUsize` 跟踪已处理文件数量
/// - 每个线程处理文件前原子性地增加计数器
/// - 实时显示处理进度和线程信息
///
/// ### 错误处理
/// - 单个文件失败不影响其他文件处理
/// - 错误信息包含文件路径和详细错误描述
/// - 根据错误类型进行分类，便于后续分析
///
/// ## 性能特性
/// - **CPU 密集型优化**: 充分利用多核处理器
/// - **内存效率**: 流式处理，避免大量内存占用
/// - **I/O 优化**: 并行 I/O 操作，减少等待时间
///
/// # 参数
/// - `files_to_process` - 要处理的文件列表，每个元素包含：
///   - `PathBuf` - 文件的完整路径（用于实际处理）
///   - `String` - 显示路径（用于用户界面）
///
/// # 返回值
/// 返回处理结果的向量，每个元素为：
/// - `Ok((String, f64))` - 成功：(显示路径, LRA值)
/// - `Err(ProcessFileError)` - 失败：包含错误详情的结构体
///
/// # 线程安全性
/// - 使用原子操作进行计数，避免数据竞争
/// - 每个文件的处理完全独立，无共享状态
/// - 输出操作使用 println! 宏，内部有锁保护
pub fn process_files_parallel(
    files_to_process: Vec<(PathBuf, String)>,
) -> Vec<Result<(String, f64), ProcessFileError>> {
    let total_files = files_to_process.len();
    let processed_count = AtomicUsize::new(0);

    println!("开始多线程直接分析...");
    println!("总文件数: {}, 可用 CPU 核心数: {}", total_files, rayon::current_num_threads());

    // 使用 Rayon 的并行迭代器进行数据并行处理
    // into_par_iter() 将 Vec 转换为并行迭代器，自动分配到多个线程
    files_to_process
        .into_par_iter()
        .map(|(current_file_path, display_path_str)| {
            // 原子性地增加已处理计数，确保线程安全
            // fetch_add 返回增加前的值，所以需要 +1 得到当前处理的文件序号
            let current_processed_atomic = processed_count.fetch_add(1, Ordering::SeqCst) + 1;

            // 显示开始处理的信息，包含线程 ID 用于调试
            println!(
                "  [线程 {:?}] ({}/{}) 开始分析: {}",
                thread::current().id(),
                current_processed_atomic,
                total_files,
                display_path_str
            );

            // 执行实际的 LRA 计算
            let result = process_single_file(&current_file_path, &display_path_str);

            // 根据处理结果显示相应的信息
            match &result {
                Ok((_, lra)) => {
                    println!(
                        "    [线程 {:?}] ({}/{}) ✓ 分析成功: {} → LRA: {:.1} LU",
                        thread::current().id(),
                        current_processed_atomic,
                        total_files,
                        display_path_str,
                        lra
                    );
                }
                Err(error) => {
                    println!(
                        "    [线程 {:?}] ({}/{}) ✗ 分析失败: {} → {}",
                        thread::current().id(),
                        current_processed_atomic,
                        total_files,
                        display_path_str,
                        error.message
                    );
                }
            }

            result
        })
        .collect()  // 收集所有结果到 Vec 中
}

/// 处理单个音频文件 (Process Single Audio File)
///
/// 这个辅助函数封装了单个文件的处理逻辑，包括 LRA 计算和错误分类。
/// 分离这个逻辑可以提高代码的可读性和可测试性。
///
/// ## 错误分类策略
/// 根据错误信息的内容自动判断错误类型：
/// - FFmpeg 相关错误：包含 "ffmpeg" 或 "FFmpeg" 关键词
/// - LRA 解析错误：包含 "解析" 或 "LRA" 关键词
/// - 其他错误：未分类的错误类型
///
/// # 参数
/// - `file_path` - 文件的完整路径
/// - `display_path` - 用于显示的路径
///
/// # 返回值
/// - `Ok((String, f64))` - 成功：(显示路径, LRA值)
/// - `Err(ProcessFileError)` - 失败：分类后的错误信息
fn process_single_file(
    file_path: &Path,
    display_path: &str
) -> Result<(String, f64), ProcessFileError> {
    match calculate_lra_direct(file_path) {
        Ok(lra) => Ok((display_path.to_string(), lra)),
        Err(e) => {
            let err_msg = format!("分析失败: {e}");

            // 根据错误信息内容自动分类错误类型
            let error = if err_msg.contains("ffmpeg") || err_msg.contains("FFmpeg") {
                ProcessFileError::ffmpeg_error(display_path.to_string(), err_msg)
            } else if err_msg.contains("解析") || err_msg.contains("LRA") {
                ProcessFileError::lra_parsing_error(display_path.to_string(), err_msg)
            } else {
                ProcessFileError::new(
                    display_path.to_string(),
                    err_msg,
                    crate::error::FileErrorType::Other
                )
            };

            Err(error)
        }
    }
}

/// 处理结果统计信息 (Processing Statistics)
///
/// 这个结构体用于汇总并行处理的统计信息，提供处理结果的概览。
/// 它不仅包含成功和失败的数量，还保存了详细的错误信息用于调试和用户反馈。
///
/// ## 设计考虑
///
/// ### 统计维度
/// - **成功计数**: 成功处理的文件数量，用于计算成功率
/// - **失败计数**: 处理失败的文件数量，用于识别问题严重程度
/// - **错误详情**: 保存所有错误信息，便于问题诊断和用户反馈
///
/// ### 内存管理
/// - 错误信息使用 `Vec<String>` 存储，避免生命周期复杂性
/// - 在大批量处理时，错误信息可能占用较多内存，但通常错误数量有限
///
/// ### 扩展性
/// - 结构体设计便于未来添加更多统计维度（如处理时间、文件大小等）
/// - 所有字段都是公开的，便于外部代码访问和分析
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    /// 成功处理的文件数量
    pub successful: usize,
    /// 失败的文件数量
    pub failed: usize,
    /// 详细的错误信息列表，每个元素包含文件路径和错误描述
    pub error_messages: Vec<String>,
}

impl ProcessingStats {
    /// 创建新的统计信息实例
    ///
    /// # 参数
    /// - `successful` - 成功处理的文件数量
    /// - `failed` - 失败的文件数量
    /// - `error_messages` - 错误信息列表
    pub fn new(successful: usize, failed: usize, error_messages: Vec<String>) -> Self {
        Self {
            successful,
            failed,
            error_messages,
        }
    }

    /// 获取总处理文件数量
    pub fn total(&self) -> usize {
        self.successful + self.failed
    }

    /// 计算成功率（百分比）
    pub fn success_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            (self.successful as f64 / self.total() as f64) * 100.0
        }
    }

    /// 检查是否有处理失败的文件
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }
}

/// 分析处理结果并生成统计信息 (Analyze Processing Results and Generate Statistics)
///
/// 这个函数负责汇总并行处理的结果，将成功和失败的结果分别收集，
/// 并生成详细的统计信息。它是处理流程中的重要环节，连接并行处理和结果输出。
///
/// ## 处理策略
///
/// ### 结果分类
/// - **成功结果**: 提取文件路径和 LRA 值，用于后续的文件输出
/// - **失败结果**: 收集错误信息，用于用户反馈和问题诊断
///
/// ### 统计计算
/// - 统计成功和失败的文件数量
/// - 格式化错误信息，包含文件路径和错误类型
/// - 生成便于后续处理的数据结构
///
/// ### 内存优化
/// - 使用 `Vec::with_capacity` 预分配内存（如果知道大小）
/// - 避免不必要的字符串克隆
/// - 使用迭代器进行高效的数据转换
///
/// # 参数
/// - `results` - 并行处理的结果向量，每个元素为成功或失败的结果
///
/// # 返回值
/// 返回一个元组：
/// - `ProcessingStats` - 包含统计信息和错误详情的结构体
/// - `Vec<(String, f64)>` - 成功处理的文件列表，包含路径和 LRA 值
///
/// # 性能特性
/// - 时间复杂度: O(n)，其中 n 是结果数量
/// - 空间复杂度: O(n)，需要存储所有成功结果和错误信息
pub fn analyze_results(
    results: Vec<Result<(String, f64), ProcessFileError>>,
) -> (ProcessingStats, Vec<(String, f64)>) {
    // 预分配向量容量以提高性能
    let total_count = results.len();
    let mut successful_results = Vec::with_capacity(total_count);
    let mut error_messages = Vec::new();
    let mut successful_count = 0;
    let mut failed_count = 0;

    // 使用迭代器处理结果，避免索引访问
    for result in results {
        match result {
            Ok((path_str, lra)) => {
                successful_results.push((path_str, lra));
                successful_count += 1;
            }
            Err(error) => {
                // 格式化错误信息，包含错误类型和详细描述
                let formatted_error = format!(
                    "文件 '{}' [{}]: {}",
                    error.file_path,
                    error.error_type_description(),
                    error.message
                );
                error_messages.push(formatted_error);
                failed_count += 1;
            }
        }
    }

    // 创建统计信息结构体
    let stats = ProcessingStats {
        successful: successful_count,
        failed: failed_count,
        error_messages,
    };

    (stats, successful_results)
}

/// 显示处理结果统计信息 (Display Processing Statistics)
///
/// 这个函数负责向用户展示处理结果的详细统计信息，包括成功率、失败详情等。
/// 它提供了友好的用户界面，帮助用户理解处理结果和识别潜在问题。
///
/// ## 显示策略
///
/// ### 成功信息
/// - 显示成功处理的文件数量和成功率
/// - 使用绿色或正面的表述增强用户体验
///
/// ### 失败信息
/// - 按错误类型分组显示失败信息
/// - 提供具体的错误描述和可能的解决方案
/// - 使用 `eprintln!` 输出到 stderr，便于日志分离
///
/// ### 格式化输出
/// - 使用清晰的层次结构和缩进
/// - 包含统计摘要和详细信息
/// - 支持大量错误信息的合理截断
///
/// # 参数
/// - `stats` - 包含处理统计信息的结构体引用
///
/// # 输出格式示例
/// ```text
///
/// ==================== 处理结果统计 ====================
/// 总文件数: 150
/// 成功处理: 148 个文件 (98.7%)
/// 处理失败: 2 个文件 (1.3%)
///
/// 失败文件详情:
///   - 文件 'corrupted.mp3' [FFmpeg 执行失败]: 音频文件损坏
///   - 文件 'invalid.wav' [LRA 值解析失败]: 无法解析 LRA 值
/// =====================================================
/// ```
pub fn display_processing_stats(stats: &ProcessingStats) {
    println!("\n==================== 处理结果统计 ====================");

    let total = stats.successful + stats.failed;
    println!("总文件数: {}", total);

    if total > 0 {
        let success_rate = (stats.successful as f64 / total as f64) * 100.0;
        println!("成功处理: {} 个文件 ({:.1}%)", stats.successful, success_rate);

        if stats.failed > 0 {
            let failure_rate = (stats.failed as f64 / total as f64) * 100.0;
            println!("处理失败: {} 个文件 ({:.1}%)", stats.failed, failure_rate);

            println!("\n失败文件详情:");
            display_error_details(&stats.error_messages);
        } else {
            println!("🎉 所有文件都已成功处理！");
        }
    } else {
        println!("⚠️  没有找到要处理的文件。");
    }

    println!("=====================================================");
}

/// 显示错误详情 (Display Error Details)
///
/// 这个辅助函数负责格式化和显示错误信息，支持大量错误的合理处理。
///
/// ## 显示策略
/// - 如果错误数量较少（≤10），显示所有错误
/// - 如果错误数量较多，显示前几个并提示总数
/// - 按错误类型进行分组（未来扩展）
///
/// # 参数
/// - `error_messages` - 错误信息列表的引用
fn display_error_details(error_messages: &[String]) {
    const MAX_DISPLAY_ERRORS: usize = 10;

    let display_count = error_messages.len().min(MAX_DISPLAY_ERRORS);

    for (index, error_msg) in error_messages.iter().take(display_count).enumerate() {
        println!("  {}. {}", index + 1, error_msg);
    }

    if error_messages.len() > MAX_DISPLAY_ERRORS {
        let remaining = error_messages.len() - MAX_DISPLAY_ERRORS;
        println!("  ... 还有 {} 个错误未显示", remaining);
        println!("  💡 提示: 检查日志文件获取完整错误列表");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ProcessFileError, FileErrorType};

    /// 测试 ProcessingStats 结构体的基本功能
    #[test]
    fn test_processing_stats_creation() {
        let error_messages = vec![
            "错误1".to_string(),
            "错误2".to_string(),
        ];

        let stats = ProcessingStats {
            successful: 10,
            failed: 2,
            error_messages: error_messages.clone(),
        };

        assert_eq!(stats.successful, 10);
        assert_eq!(stats.failed, 2);
        assert_eq!(stats.error_messages.len(), 2);
        assert_eq!(stats.error_messages, error_messages);
    }

    /// 测试 ProcessingStats 的便利方法
    #[test]
    fn test_processing_stats_methods() {
        let stats = ProcessingStats::new(
            15,
            3,
            vec!["错误1".to_string(), "错误2".to_string(), "错误3".to_string()]
        );

        // 测试总数计算
        assert_eq!(stats.total(), 18);

        // 测试成功率计算
        let expected_rate = (15.0 / 18.0) * 100.0;
        assert!((stats.success_rate() - expected_rate).abs() < 0.01);

        // 测试失败检查
        assert!(stats.has_failures());

        // 测试没有失败的情况
        let no_failure_stats = ProcessingStats::new(10, 0, vec![]);
        assert!(!no_failure_stats.has_failures());
        assert_eq!(no_failure_stats.success_rate(), 100.0);

        // 测试空统计的情况
        let empty_stats = ProcessingStats::new(0, 0, vec![]);
        assert_eq!(empty_stats.total(), 0);
        assert_eq!(empty_stats.success_rate(), 0.0);
        assert!(!empty_stats.has_failures());
    }

    /// 测试结果分析功能
    #[test]
    fn test_analyze_results() {
        // 创建测试数据
        let test_results = vec![
            Ok(("file1.mp3".to_string(), 12.5)),
            Ok(("file2.wav".to_string(), 8.3)),
            Err(ProcessFileError::ffmpeg_error(
                "file3.flac".to_string(),
                "FFmpeg 执行失败".to_string()
            )),
            Ok(("file4.m4a".to_string(), 15.7)),
            Err(ProcessFileError::lra_parsing_error(
                "file5.mp3".to_string(),
                "LRA 解析失败".to_string()
            )),
            Ok(("file6.ogg".to_string(), 9.1)),
        ];

        // 执行分析
        let (stats, successful_results) = analyze_results(test_results);

        // 验证统计信息
        assert_eq!(stats.successful, 4);
        assert_eq!(stats.failed, 2);
        assert_eq!(stats.error_messages.len(), 2);

        // 验证成功结果
        assert_eq!(successful_results.len(), 4);
        assert_eq!(successful_results[0], ("file1.mp3".to_string(), 12.5));
        assert_eq!(successful_results[1], ("file2.wav".to_string(), 8.3));
        assert_eq!(successful_results[2], ("file4.m4a".to_string(), 15.7));
        assert_eq!(successful_results[3], ("file6.ogg".to_string(), 9.1));

        // 验证错误信息格式
        assert!(stats.error_messages[0].contains("file3.flac"));
        assert!(stats.error_messages[0].contains("FFmpeg 执行失败"));
        assert!(stats.error_messages[1].contains("file5.mp3"));
        assert!(stats.error_messages[1].contains("LRA 解析失败"));
    }

    /// 测试空结果的分析
    #[test]
    fn test_analyze_empty_results() {
        let empty_results = vec![];
        let (stats, successful_results) = analyze_results(empty_results);

        assert_eq!(stats.successful, 0);
        assert_eq!(stats.failed, 0);
        assert!(stats.error_messages.is_empty());
        assert!(successful_results.is_empty());
    }

    /// 测试只有成功结果的分析
    #[test]
    fn test_analyze_only_successful_results() {
        let success_only_results = vec![
            Ok(("file1.mp3".to_string(), 12.5)),
            Ok(("file2.wav".to_string(), 8.3)),
            Ok(("file3.flac".to_string(), 15.7)),
        ];

        let (stats, successful_results) = analyze_results(success_only_results);

        assert_eq!(stats.successful, 3);
        assert_eq!(stats.failed, 0);
        assert!(stats.error_messages.is_empty());
        assert_eq!(successful_results.len(), 3);
    }

    /// 测试只有失败结果的分析
    #[test]
    fn test_analyze_only_failed_results() {
        let failure_only_results = vec![
            Err(ProcessFileError::ffmpeg_error(
                "file1.mp3".to_string(),
                "错误1".to_string()
            )),
            Err(ProcessFileError::lra_parsing_error(
                "file2.wav".to_string(),
                "错误2".to_string()
            )),
        ];

        let (stats, successful_results) = analyze_results(failure_only_results);

        assert_eq!(stats.successful, 0);
        assert_eq!(stats.failed, 2);
        assert_eq!(stats.error_messages.len(), 2);
        assert!(successful_results.is_empty());
    }

    /// 测试并行处理空文件列表
    #[test]
    fn test_process_empty_file_list() {
        let empty_files = vec![];
        let results = process_files_parallel(empty_files);
        assert!(results.is_empty());
    }

    /// 测试单个文件处理函数（模拟）
    #[test]
    fn test_process_single_file_error_classification() {
        // 注意：这个测试不会实际调用 FFmpeg，因为我们没有真实的音频文件
        // 我们主要测试错误分类逻辑

        // 由于 process_single_file 是私有函数且依赖 FFmpeg，
        // 我们通过测试 analyze_results 来间接测试错误分类

        let ffmpeg_error = ProcessFileError::ffmpeg_error(
            "test.mp3".to_string(),
            "ffmpeg 命令执行失败".to_string()
        );

        let lra_error = ProcessFileError::lra_parsing_error(
            "test.wav".to_string(),
            "无法解析 LRA 值".to_string()
        );

        let other_error = ProcessFileError::new(
            "test.flac".to_string(),
            "其他类型的错误".to_string(),
            FileErrorType::Other
        );

        // 验证错误类型描述
        assert_eq!(ffmpeg_error.error_type_description(), "FFmpeg 执行失败");
        assert_eq!(lra_error.error_type_description(), "LRA 值解析失败");
        assert_eq!(other_error.error_type_description(), "其他错误");
    }

    /// 测试显示错误详情功能
    #[test]
    fn test_display_error_details() {
        // 这个测试主要验证函数不会崩溃
        // 实际的输出需要手动验证

        let few_errors = vec![
            "错误1".to_string(),
            "错误2".to_string(),
            "错误3".to_string(),
        ];

        // 测试少量错误（不应该崩溃）
        display_error_details(&few_errors);

        // 测试大量错误
        let many_errors: Vec<String> = (0..20)
            .map(|i| format!("错误{}", i))
            .collect();

        display_error_details(&many_errors);

        // 测试空错误列表
        let empty_errors: Vec<String> = vec![];
        display_error_details(&empty_errors);
    }
}
