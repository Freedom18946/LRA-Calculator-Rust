//! LRA 计算器性能基准测试 (LRA Calculator Performance Benchmarks)
//! 
//! 本文件包含了 LRA 计算器各个组件的性能基准测试，
//! 用于监控和优化程序的性能表现。

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs::File;
use std::path::Path;
use tempfile::TempDir;

use lra_calculator_rust::audio::{scan_audio_files, extract_file_extension, is_supported_audio_format};
use lra_calculator_rust::processor::{analyze_results, ProcessingStats};
use lra_calculator_rust::utils::{parse_result_line, sort_entries_by_lra};
use lra_calculator_rust::error::{ProcessFileError, FileErrorType};

/// 基准测试：文件扫描性能
/// 
/// 测试在不同数量的文件下，文件扫描功能的性能表现。
fn benchmark_file_scanning(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_scanning");
    
    // 测试不同数量的文件
    for file_count in [100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("scan_audio_files", file_count),
            file_count,
            |b, &file_count| {
                // 设置测试环境
                let temp_dir = TempDir::new().expect("无法创建临时目录");
                let temp_path = temp_dir.path();
                
                // 创建测试文件
                for i in 0..file_count {
                    let file_path = temp_path.join(format!("test_{:04}.mp3", i));
                    File::create(file_path).expect("无法创建测试文件");
                }
                
                // 基准测试
                b.iter(|| {
                    let files = scan_audio_files(black_box(temp_path), None);
                    black_box(files)
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：文件扩展名提取性能
/// 
/// 测试文件扩展名提取函数的性能。
fn benchmark_file_extension_extraction(c: &mut Criterion) {
    let test_paths = vec![
        "simple.mp3",
        "path/to/file.wav",
        "very/long/path/to/audio/file/with/many/directories.flac",
        "file.with.multiple.dots.m4a",
        "unicode_文件名.ogg",
    ];
    
    c.bench_function("extract_file_extension", |b| {
        b.iter(|| {
            for path_str in &test_paths {
                let path = Path::new(path_str);
                let extension = extract_file_extension(black_box(path));
                black_box(extension);
            }
        });
    });
}

/// 基准测试：音频格式支持检查性能
/// 
/// 测试音频格式支持检查函数的性能。
fn benchmark_format_support_check(c: &mut Criterion) {
    let test_extensions = vec![
        "mp3", "wav", "flac", "m4a", "aac", "ogg", "opus", "wma", "aiff", "alac",
        "txt", "doc", "pdf", "jpg", "png", "zip", "exe", "dll", "so", "dylib",
    ];
    
    c.bench_function("is_supported_audio_format", |b| {
        b.iter(|| {
            for extension in &test_extensions {
                let is_supported = is_supported_audio_format(black_box(extension));
                black_box(is_supported);
            }
        });
    });
}

/// 基准测试：结果分析性能
/// 
/// 测试处理结果分析功能在不同数据量下的性能。
fn benchmark_result_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("result_analysis");
    
    for result_count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("analyze_results", result_count),
            result_count,
            |b, &result_count| {
                // 创建测试数据
                let mut test_results = Vec::new();
                
                for i in 0..result_count {
                    if i % 10 == 0 {
                        // 10% 的失败率
                        test_results.push(Err(ProcessFileError::new(
                            format!("file_{}.mp3", i),
                            "模拟错误".to_string(),
                            FileErrorType::Other,
                        )));
                    } else {
                        test_results.push(Ok((
                            format!("file_{}.mp3", i),
                            (i as f64) * 0.1 + 5.0, // 模拟 LRA 值
                        )));
                    }
                }
                
                // 基准测试
                b.iter(|| {
                    let (stats, successful) = analyze_results(black_box(test_results.clone()));
                    black_box((stats, successful))
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：结果行解析性能
/// 
/// 测试结果文件行解析功能的性能。
fn benchmark_result_line_parsing(c: &mut Criterion) {
    let test_lines = vec![
        "simple.mp3 - 12.5",
        "path/to/long/file/name.wav - 8.3",
        "unicode_文件名_with_spaces.flac - 15.7",
        "file.with.multiple.dots.m4a - 20.1",
        "very/very/very/long/path/to/audio/file/in/deep/directory/structure.ogg - 6.9",
    ];
    
    c.bench_function("parse_result_line", |b| {
        b.iter(|| {
            for line in &test_lines {
                let result = parse_result_line(black_box(line));
                black_box(result);
            }
        });
    });
}

/// 基准测试：条目排序性能
/// 
/// 测试结果条目排序功能在不同数据量下的性能。
fn benchmark_entry_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("entry_sorting");
    
    for entry_count in [100, 500, 1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("sort_entries_by_lra", entry_count),
            entry_count,
            |b, &entry_count| {
                // 创建测试数据（随机 LRA 值）
                let mut entries = Vec::new();
                for i in 0..entry_count {
                    let lra = (i as f64 * 7.0) % 25.0; // 生成 0-25 范围的 LRA 值
                    entries.push((format!("file_{:04}.mp3", i), lra));
                }
                
                // 基准测试
                b.iter(|| {
                    let sorted = sort_entries_by_lra(black_box(entries.clone()));
                    black_box(sorted)
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：ProcessingStats 创建和方法调用
/// 
/// 测试 ProcessingStats 结构体的性能。
fn benchmark_processing_stats(c: &mut Criterion) {
    let error_messages: Vec<String> = (0..1000)
        .map(|i| format!("错误信息 {}", i))
        .collect();
    
    c.bench_function("processing_stats_creation", |b| {
        b.iter(|| {
            let stats = ProcessingStats::new(
                black_box(800),
                black_box(200),
                black_box(error_messages.clone()),
            );
            black_box(stats)
        });
    });
    
    let stats = ProcessingStats::new(800, 200, error_messages);
    
    c.bench_function("processing_stats_methods", |b| {
        b.iter(|| {
            let total = stats.total();
            let success_rate = stats.success_rate();
            let has_failures = stats.has_failures();
            black_box((total, success_rate, has_failures))
        });
    });
}

/// 基准测试：内存使用模式
/// 
/// 测试程序在处理大量数据时的内存分配模式。
fn benchmark_memory_usage(c: &mut Criterion) {
    c.bench_function("large_data_processing", |b| {
        b.iter(|| {
            // 模拟处理大量文件的内存使用模式
            let file_count = 10000;
            let mut file_paths = Vec::with_capacity(file_count);
            
            // 创建文件路径列表
            for i in 0..file_count {
                file_paths.push((
                    std::path::PathBuf::from(format!("/path/to/file_{:05}.mp3", i)),
                    format!("file_{:05}.mp3", i),
                ));
            }
            
            // 模拟处理结果
            let mut results = Vec::with_capacity(file_count);
            for (_, display_path) in &file_paths {
                if display_path.ends_with("0.mp3") {
                    // 模拟 10% 的失败率
                    results.push(Err(ProcessFileError::new(
                        display_path.clone(),
                        "模拟错误".to_string(),
                        FileErrorType::Other,
                    )));
                } else {
                    results.push(Ok((display_path.clone(), 12.5)));
                }
            }
            
            // 分析结果
            let (stats, successful) = analyze_results(black_box(results));
            black_box((stats, successful, file_paths))
        });
    });
}

// 定义基准测试组
criterion_group!(
    benches,
    benchmark_file_scanning,
    benchmark_file_extension_extraction,
    benchmark_format_support_check,
    benchmark_result_analysis,
    benchmark_result_line_parsing,
    benchmark_entry_sorting,
    benchmark_processing_stats,
    benchmark_memory_usage
);

criterion_main!(benches);
