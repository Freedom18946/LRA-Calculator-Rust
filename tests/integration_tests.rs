//! 集成测试 (Integration Tests)
//! 
//! 本文件包含了 LRA 计算器的集成测试，验证整个系统的端到端功能。
//! 这些测试模拟真实的使用场景，确保各个模块协同工作正常。

use std::fs::{self, File};
use std::path::Path;
use tempfile::TempDir;

// 导入被测试的模块
// 注意：Rust 中连字符会被转换为下划线
use lra_calculator_rust::audio::{scan_audio_files, calculate_lra_direct, check_ffmpeg_availability};
use lra_calculator_rust::processor::{process_files_parallel, analyze_results};
use lra_calculator_rust::utils::{validate_folder_path, sort_lra_results_file};
use lra_calculator_rust::error::{AppError, ProcessFileError};

/// 测试 FFmpeg 环境检查功能
/// 
/// 验证程序能够正确检测 FFmpeg 的可用性。
/// 这是所有其他测试的前提条件。
#[test]
fn test_ffmpeg_availability() {
    match check_ffmpeg_availability() {
        Ok(()) => {
            println!("✅ FFmpeg 可用，可以进行后续测试");
        }
        Err(e) => {
            panic!("❌ FFmpeg 不可用，无法进行测试: {}", e);
        }
    }
}

/// 测试文件夹路径验证功能
/// 
/// 验证路径验证函数能够正确处理各种路径情况。
#[test]
fn test_folder_path_validation() {
    // 测试有效路径（当前目录）
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    assert!(validate_folder_path(&current_dir).is_ok());
    
    // 测试无效路径
    let invalid_path = Path::new("/this/path/should/not/exist/12345");
    assert!(validate_folder_path(invalid_path).is_err());
    
    // 测试文件而非目录（使用 Cargo.toml 作为测试文件）
    let file_path = Path::new("Cargo.toml");
    if file_path.exists() {
        assert!(validate_folder_path(file_path).is_err());
    }
}

/// 测试音频文件扫描功能
/// 
/// 创建临时目录结构，测试文件扫描的准确性。
#[test]
fn test_audio_file_scanning() {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let temp_path = temp_dir.path();
    
    // 创建测试目录结构
    let subdir = temp_path.join("subdir");
    fs::create_dir(&subdir).expect("无法创建子目录");
    
    // 创建测试文件（空文件用于测试扫描功能）
    let test_files = vec![
        temp_path.join("test1.mp3"),
        temp_path.join("test2.wav"),
        subdir.join("test3.flac"),
        temp_path.join("not_audio.txt"),  // 非音频文件
        temp_path.join("test4.m4a"),
    ];
    
    for file_path in &test_files {
        File::create(file_path).expect("无法创建测试文件");
    }
    
    // 扫描音频文件
    let found_files = scan_audio_files(temp_path, None);
    
    // 验证结果
    assert_eq!(found_files.len(), 4); // 应该找到 4 个音频文件
    
    // 验证找到的文件包含预期的音频文件
    let found_names: Vec<String> = found_files.iter()
        .map(|(_, display_path)| display_path.clone())
        .collect();
    
    assert!(found_names.iter().any(|name| name.contains("test1.mp3")));
    assert!(found_names.iter().any(|name| name.contains("test2.wav")));
    assert!(found_names.iter().any(|name| name.contains("test3.flac")));
    assert!(found_names.iter().any(|name| name.contains("test4.m4a")));
    
    // 确保非音频文件被排除
    assert!(!found_names.iter().any(|name| name.contains("not_audio.txt")));
}

/// 测试排除文件功能
/// 
/// 验证文件扫描时能够正确排除指定的文件。
#[test]
fn test_file_exclusion() {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let temp_path = temp_dir.path();
    
    // 创建测试文件
    let audio_file = temp_path.join("audio.mp3");
    let exclude_file = temp_path.join("lra_results.txt");
    
    File::create(&audio_file).expect("无法创建音频文件");
    File::create(&exclude_file).expect("无法创建排除文件");
    
    // 不排除任何文件的扫描
    let files_without_exclusion = scan_audio_files(temp_path, None);
    assert_eq!(files_without_exclusion.len(), 1);
    
    // 排除结果文件的扫描
    let files_with_exclusion = scan_audio_files(temp_path, Some(&exclude_file));
    assert_eq!(files_with_exclusion.len(), 1); // 应该还是 1 个，因为排除的不是音频文件
    
    // 如果排除文件也是音频格式
    let audio_exclude = temp_path.join("exclude.mp3");
    File::create(&audio_exclude).expect("无法创建排除的音频文件");
    
    let files_excluding_audio = scan_audio_files(temp_path, Some(&audio_exclude));
    assert_eq!(files_excluding_audio.len(), 1); // 应该只找到一个文件
}

/// 测试结果分析功能
/// 
/// 验证处理结果的分析和统计功能。
#[test]
fn test_result_analysis() {
    // 创建模拟的处理结果
    let mock_results = vec![
        Ok(("file1.mp3".to_string(), 12.5)),
        Ok(("file2.wav".to_string(), 8.3)),
        Err(ProcessFileError::ffmpeg_error(
            "file3.flac".to_string(),
            "模拟的 FFmpeg 错误".to_string()
        )),
        Ok(("file4.m4a".to_string(), 15.7)),
        Err(ProcessFileError::lra_parsing_error(
            "file5.mp3".to_string(),
            "模拟的解析错误".to_string()
        )),
    ];
    
    // 分析结果
    let (stats, successful_results) = analyze_results(mock_results);
    
    // 验证统计信息
    assert_eq!(stats.successful, 3);
    assert_eq!(stats.failed, 2);
    assert_eq!(stats.error_messages.len(), 2);
    
    // 验证成功结果
    assert_eq!(successful_results.len(), 3);
    assert_eq!(successful_results[0].0, "file1.mp3");
    assert_eq!(successful_results[0].1, 12.5);
    
    // 验证错误信息包含预期内容
    assert!(stats.error_messages.iter().any(|msg| msg.contains("file3.flac")));
    assert!(stats.error_messages.iter().any(|msg| msg.contains("file5.mp3")));
}

/// 测试结果文件排序功能
/// 
/// 验证结果文件的排序功能是否正常工作。
#[test]
fn test_result_file_sorting() {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let results_file = temp_dir.path().join("test_results.txt");
    
    // 创建测试结果文件
    let test_content = r#"文件路径 (相对) - LRA 数值 (LU)
file1.mp3 - 8.5
file2.wav - 15.2
file3.flac - 12.1
file4.m4a - 20.0
file5.ogg - 5.3"#;
    
    fs::write(&results_file, test_content).expect("无法写入测试文件");
    
    // 执行排序
    let header_line = "文件路径 (相对) - LRA 数值 (LU)";
    sort_lra_results_file(&results_file, header_line).expect("排序失败");
    
    // 读取排序后的内容
    let sorted_content = fs::read_to_string(&results_file).expect("无法读取排序后的文件");
    let lines: Vec<&str> = sorted_content.lines().collect();
    
    // 验证排序结果（应该按 LRA 值降序排列）
    assert_eq!(lines.len(), 6); // 包括表头
    assert_eq!(lines[0], header_line);
    assert!(lines[1].contains("file4.m4a - 20.0"));
    assert!(lines[2].contains("file2.wav - 15.2"));
    assert!(lines[3].contains("file3.flac - 12.1"));
    assert!(lines[4].contains("file1.mp3 - 8.5"));
    assert!(lines[5].contains("file5.ogg - 5.3"));
}

/// 测试空结果文件的处理
/// 
/// 验证程序能够正确处理空的或只有表头的结果文件。
#[test]
fn test_empty_result_file_handling() {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let results_file = temp_dir.path().join("empty_results.txt");
    
    // 创建只有表头的文件
    let header_line = "文件路径 (相对) - LRA 数值 (LU)";
    fs::write(&results_file, header_line).expect("无法写入测试文件");
    
    // 执行排序（应该不会出错）
    let result = sort_lra_results_file(&results_file, header_line);
    assert!(result.is_ok());
    
    // 验证文件内容保持不变
    let content = fs::read_to_string(&results_file).expect("无法读取文件");
    assert_eq!(content.trim(), header_line);
}

/// 测试错误处理的健壮性
/// 
/// 验证程序在遇到各种错误情况时的处理能力。
#[test]
fn test_error_handling_robustness() {
    // 测试不存在的路径
    let non_existent = Path::new("/this/path/does/not/exist");
    let validation_result = validate_folder_path(non_existent);
    assert!(validation_result.is_err());
    
    if let Err(AppError::Path(msg)) = validation_result {
        assert!(msg.contains("不存在"));
    } else {
        panic!("期望得到 AppError::Path 错误");
    }
    
    // 测试空文件列表的并行处理
    let empty_files = vec![];
    let empty_results = process_files_parallel(empty_files);
    assert!(empty_results.is_empty());
    
    // 测试空结果的分析
    let (empty_stats, empty_successful) = analyze_results(vec![]);
    assert_eq!(empty_stats.successful, 0);
    assert_eq!(empty_stats.failed, 0);
    assert!(empty_successful.is_empty());
}

/// 性能基准测试（简单版本）
/// 
/// 测试程序在处理大量文件时的性能表现。
/// 注意：这个测试创建的是空文件，不会进行实际的 LRA 计算。
#[test]
#[ignore] // 默认忽略，因为这是一个较慢的测试
fn test_performance_with_many_files() {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let temp_path = temp_dir.path();
    
    // 创建大量测试文件
    const FILE_COUNT: usize = 1000;
    for i in 0..FILE_COUNT {
        let file_path = temp_path.join(format!("test_{:04}.mp3", i));
        File::create(file_path).expect("无法创建测试文件");
    }
    
    // 测试文件扫描性能
    let start_time = std::time::Instant::now();
    let found_files = scan_audio_files(temp_path, None);
    let scan_duration = start_time.elapsed();
    
    assert_eq!(found_files.len(), FILE_COUNT);
    println!("扫描 {} 个文件耗时: {:?}", FILE_COUNT, scan_duration);
    
    // 验证扫描时间在合理范围内（应该在几毫秒内完成）
    assert!(scan_duration.as_millis() < 1000, "文件扫描耗时过长");
}
