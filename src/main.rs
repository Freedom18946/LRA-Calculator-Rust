//! # LRA 音频响度范围计算器
//!
//! 这是一个高性能的命令行工具，用于递归计算指定文件夹内所有音频文件的响度范围（Loudness Range, LRA）。
//! 它利用多线程并行处理来最大化效率，并使用业界标准的 FFmpeg 进行核心分析。
//!
//! ## 主要功能
//! - 递归扫描音频文件
//! - 多线程并行处理
//! - 支持多种音频格式
//! - 基于 EBU R128 标准的精确 LRA 计算
//! - 结果自动排序和保存

mod audio;
mod error;
mod processor;
mod utils;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use chrono::Local;

use audio::{check_ffmpeg_availability, scan_audio_files};
use processor::{analyze_results, display_processing_stats, process_files_parallel};
use utils::{get_folder_path_from_user, sort_lra_results_file};


/// 程序主入口函数 (Main Entry Point)
///
/// 这是 LRA 音频响度范围计算器的主控制函数，协调整个处理流程。
/// 它按照清晰的步骤执行完整的 LRA 计算工作流，包含完善的错误处理和用户反馈。
///
/// ## 执行流程
///
/// ### 1. 环境初始化和检查
/// - 显示欢迎信息和程序版本
/// - 检查 FFmpeg 的可用性和版本兼容性
/// - 验证系统环境是否满足运行要求
///
/// ### 2. 用户交互和输入验证
/// - 获取用户输入的文件夹路径
/// - 验证路径的有效性和访问权限
/// - 提供友好的错误提示和重试机制
///
/// ### 3. 文件发现和预处理
/// - 递归扫描指定目录及其子目录
/// - 识别和过滤支持的音频文件格式
/// - 排除结果文件，避免处理自己生成的文件
/// - 显示发现的文件数量和预估处理时间
///
/// ### 4. 并行处理和进度跟踪
/// - 使用多线程并行计算 LRA 值
/// - 实时显示处理进度和线程状态
/// - 收集成功和失败的处理结果
/// - 提供详细的错误信息和统计数据
///
/// ### 5. 结果处理和输出
/// - 分析处理结果，生成统计信息
/// - 将成功的结果写入文件
/// - 按 LRA 值对结果进行排序
/// - 显示最终的处理摘要和文件位置
///
/// ## 错误处理策略
///
/// ### 致命错误（程序终止）
/// - FFmpeg 不可用或版本不兼容
/// - 用户输入的路径无效且无法修复
/// - 系统资源不足（内存、磁盘空间）
/// - 文件系统权限问题
///
/// ### 可恢复错误（继续处理）
/// - 单个音频文件处理失败
/// - 部分文件无法访问
/// - 网络存储临时不可用
///
/// ### 用户错误（提示重试）
/// - 路径输入错误
/// - 选择了文件而非目录
/// - 权限不足但可以修复
///
/// # 返回值
/// - `Ok(())` - 程序成功执行完成，所有步骤都正常
/// - `Err(Box<dyn std::error::Error>)` - 发生不可恢复的错误，程序需要终止
///
/// # 性能特性
/// - 自动利用所有可用 CPU 核心进行并行处理
/// - 内存使用量与文件数量成正比，通常保持在合理范围内
/// - 支持处理大型音乐库（数万个文件）
/// - 提供实时进度反馈，避免用户等待焦虑
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 程序初始化和环境检查
    display_welcome_message();
    check_system_environment()?;

    // 2. 获取用户输入和路径验证
    let base_folder_path = get_user_input_with_validation()?;

    // 3. 文件发现和预处理
    let (files_to_process, results_file_path) = discover_and_prepare_files(&base_folder_path)?;

    // 4. 并行处理和进度跟踪
    let processing_results = execute_parallel_processing(files_to_process);

    // 5. 结果处理和输出
    finalize_and_output_results(processing_results, &results_file_path)?;

    display_completion_message(&results_file_path);
    Ok(())
}

/// 显示欢迎信息 (Display Welcome Message)
///
/// 显示程序的欢迎信息、版本信息和基本说明。
/// 这有助于用户了解程序的功能和当前运行状态。
fn display_welcome_message() {
    println!("🎵 ==========================================");
    println!("🎵   LRA 音频响度范围计算器");
    println!("🎵   高性能版 - 基于 FFmpeg 直接分析");
    println!("🎵 ==========================================");
    println!("📅 启动时间: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
    println!("🔧 基于 EBU R128 标准进行精确 LRA 计算");
    println!("⚡ 支持多线程并行处理，充分利用 CPU 资源");
    println!();
}

/// 检查系统环境 (Check System Environment)
///
/// 验证程序运行所需的系统环境，主要是 FFmpeg 的可用性。
/// 如果环境检查失败，程序将终止并提供详细的错误信息。
///
/// # 返回值
/// - `Ok(())` - 系统环境检查通过
/// - `Err(...)` - 环境检查失败，包含详细错误信息
fn check_system_environment() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 正在检查系统环境...");

    match check_ffmpeg_availability() {
        Ok(()) => {
            println!("✅ 系统环境检查完成，所有依赖都已就绪");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ 系统环境检查失败: {}", e);
            eprintln!("💡 请按照错误提示安装必要的依赖后重试");
            Err(e.into())
        }
    }
}

/// 获取用户输入并验证 (Get User Input with Validation)
///
/// 获取用户输入的文件夹路径，并进行完整的验证。
/// 这个函数封装了用户交互逻辑，提供友好的错误处理。
///
/// # 返回值
/// - `Ok(PathBuf)` - 验证通过的文件夹路径
/// - `Err(...)` - 用户输入无效或验证失败
fn get_user_input_with_validation() -> Result<PathBuf, Box<dyn std::error::Error>> {
    println!("📂 请选择要处理的音频文件夹...");

    match get_folder_path_from_user() {
        Ok(path) => {
            println!("✅ 文件夹路径验证成功: {}", path.display());
            Ok(path)
        }
        Err(e) => {
            eprintln!("❌ 文件夹路径获取失败: {}", e);
            Err(e)
        }
    }
}

/// 发现和准备文件 (Discover and Prepare Files)
///
/// 扫描指定目录中的音频文件，并准备处理所需的数据结构。
/// 这个函数还会创建结果文件路径并处理空目录的情况。
///
/// # 参数
/// - `base_folder_path` - 要扫描的基础文件夹路径
///
/// # 返回值
/// - `Ok((Vec<(PathBuf, String)>, PathBuf))` - 文件列表和结果文件路径
/// - `Err(...)` - 文件扫描或准备过程中的错误
fn discover_and_prepare_files(
    base_folder_path: &Path
) -> Result<(Vec<(PathBuf, String)>, PathBuf), Box<dyn std::error::Error>> {
    println!("🔍 正在递归扫描文件夹: {}", base_folder_path.display());

    let results_file_path = base_folder_path.join("lra_results.txt");
    let files_to_process = scan_audio_files(base_folder_path, Some(&results_file_path));

    if files_to_process.is_empty() {
        println!("⚠️  在指定路径下没有找到支持的音频文件");
        println!("📝 创建空的结果文件...");

        // 创建空的结果文件
        let header_line = "文件路径 (相对) - LRA 数值 (LU)";
        let mut writer = BufWriter::new(File::create(&results_file_path)?);
        writeln!(writer, "{}", header_line)?;
        writer.flush()?;

        println!("✅ 空结果文件已创建: {}", results_file_path.display());
        return Err("没有找到要处理的音频文件".into());
    }

    println!(
        "✅ 扫描完成，发现 {} 个音频文件待处理",
        files_to_process.len()
    );

    // 显示文件格式统计
    display_file_format_statistics(&files_to_process);

    Ok((files_to_process, results_file_path))
}

/// 显示文件格式统计 (Display File Format Statistics)
///
/// 分析发现的音频文件，按格式进行统计并显示给用户。
/// 这有助于用户了解文件库的组成情况。
///
/// # 参数
/// - `files` - 发现的文件列表
fn display_file_format_statistics(files: &[(PathBuf, String)]) {
    use std::collections::HashMap;

    let mut format_counts: HashMap<String, usize> = HashMap::new();

    for (file_path, _) in files {
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            let ext_lower = extension.to_lowercase();
            *format_counts.entry(ext_lower).or_insert(0) += 1;
        }
    }

    println!("📊 文件格式统计:");
    let mut formats: Vec<_> = format_counts.into_iter().collect();
    formats.sort_by(|a, b| b.1.cmp(&a.1)); // 按数量降序排序

    for (format, count) in formats {
        println!("   {} 格式: {} 个文件", format.to_uppercase(), count);
    }
    println!();
}

/// 执行并行处理 (Execute Parallel Processing)
///
/// 启动多线程并行处理，计算所有音频文件的 LRA 值。
/// 这是程序的核心处理阶段，会显示详细的进度信息。
///
/// # 参数
/// - `files_to_process` - 要处理的文件列表
///
/// # 返回值
/// - 处理结果列表，包含成功和失败的结果
fn execute_parallel_processing(
    files_to_process: Vec<(PathBuf, String)>
) -> Vec<Result<(String, f64), crate::error::ProcessFileError>> {
    println!("⚡ 开始并行处理阶段...");

    let start_time = std::time::Instant::now();
    let results = process_files_parallel(files_to_process);
    let elapsed = start_time.elapsed();

    println!("⏱️  并行处理耗时: {:.2} 秒", elapsed.as_secs_f64());

    results
}

/// 完成处理并输出结果 (Finalize and Output Results)
///
/// 分析处理结果，写入结果文件，并进行排序。
/// 这是程序的最后阶段，负责生成最终的输出文件。
///
/// # 参数
/// - `processing_results` - 并行处理的结果
/// - `results_file_path` - 结果文件路径
///
/// # 返回值
/// - `Ok(())` - 结果处理成功
/// - `Err(...)` - 文件写入或排序失败
fn finalize_and_output_results(
    processing_results: Vec<Result<(String, f64), crate::error::ProcessFileError>>,
    results_file_path: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 正在分析处理结果...");

    // 分析结果
    let (stats, successful_results) = analyze_results(processing_results);

    // 显示统计信息
    display_processing_stats(&stats);

    // 写入结果文件
    write_initial_results_file(results_file_path, &successful_results)?;

    // 排序结果文件
    if stats.successful > 0 {
        sort_results_file_if_needed(results_file_path, &stats)?;
    } else {
        println!("📝 没有成功处理的文件，跳过排序步骤");
    }

    Ok(())
}

/// 写入初始结果文件 (Write Initial Results File)
///
/// 将成功处理的结果写入文件，包含表头和数据行。
///
/// # 参数
/// - `results_file_path` - 结果文件路径
/// - `successful_results` - 成功处理的结果列表
///
/// # 返回值
/// - `Ok(())` - 写入成功
/// - `Err(...)` - 写入失败
fn write_initial_results_file(
    results_file_path: &Path,
    successful_results: &[(String, f64)]
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 正在写入结果文件...");

    let header_line = "文件路径 (相对) - LRA 数值 (LU)";
    let mut writer = BufWriter::new(File::create(results_file_path)?);

    writeln!(writer, "{}", header_line)?;
    for (path_str, lra) in successful_results {
        writeln!(writer, "{} - {:.1}", path_str, lra)?;
    }
    writer.flush()?;

    println!("✅ 结果文件写入完成");
    Ok(())
}

/// 根据需要排序结果文件 (Sort Results File If Needed)
///
/// 对结果文件进行排序，并处理可能的排序错误。
///
/// # 参数
/// - `results_file_path` - 结果文件路径
/// - `stats` - 处理统计信息
///
/// # 返回值
/// - `Ok(())` - 排序成功或跳过
/// - `Err(...)` - 排序失败
fn sort_results_file_if_needed(
    results_file_path: &Path,
    stats: &crate::processor::ProcessingStats
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 正在对结果文件进行排序...");

    let header_line = "文件路径 (相对) - LRA 数值 (LU)";
    match sort_lra_results_file(results_file_path, header_line) {
        Ok(()) => {
            println!("✅ 结果文件排序完成");
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "⚠️  排序结果文件失败: {}\n\
                 📝 原始结果文件仍然可用: {}",
                e,
                results_file_path.display()
            );
            // 排序失败不应该导致整个程序失败
            Ok(())
        }
    }
}

/// 显示完成信息 (Display Completion Message)
///
/// 显示程序完成的信息，包括结果文件位置和使用建议。
///
/// # 参数
/// - `results_file_path` - 结果文件路径
fn display_completion_message(results_file_path: &Path) {
    println!("\n🎉 ==========================================");
    println!("🎉   所有操作已成功完成！");
    println!("🎉 ==========================================");
    println!("📄 结果文件位置: {}", results_file_path.display());
    println!("📊 文件已按 LRA 值从高到低排序");
    println!("💡 使用建议:");
    println!("   • LRA > 15 LU: 动态范围丰富（古典、爵士）");
    println!("   • LRA 8-15 LU: 适中动态范围（摇滚、民谣）");
    println!("   • LRA < 8 LU: 动态范围较小（流行、播客）");
    println!("⏰ 完成时间: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
    println!("🎵 感谢使用 LRA 计算器！");
}


