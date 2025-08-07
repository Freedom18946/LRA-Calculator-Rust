//! 实用工具模块 (Utility Module)
//!
//! 本模块提供了程序运行所需的各种辅助功能，包括用户交互、文件操作、
//! 数据处理等。这些工具函数被设计为可重用、可测试的独立组件。
//!
//! ## 核心功能
//!
//! ### 用户交互 (User Interaction)
//! - 路径输入和验证：安全地获取用户输入的文件夹路径
//! - 输入验证：确保路径存在、可访问且为目录
//! - 错误处理：提供友好的错误信息和重试机制
//!
//! ### 文件操作 (File Operations)
//! - 结果文件排序：按 LRA 值对结果进行排序
//! - 文件格式处理：解析和格式化结果文件
//! - 错误恢复：处理文件操作中的各种异常情况
//!
//! ### 数据处理 (Data Processing)
//! - 字符串解析：从文本中提取数值数据
//! - 排序算法：高效的数据排序实现
//! - 格式化输出：生成用户友好的文件格式
//!
//! ## 设计原则
//!
//! - **健壮性**: 所有函数都有完善的错误处理
//! - **用户友好**: 提供清晰的中文提示和错误信息
//! - **可测试性**: 函数设计便于单元测试
//! - **性能优化**: 使用高效的算法和数据结构

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::error::AppError;

/// 从用户输入获取要处理的文件夹路径 (Get Folder Path from User Input)
///
/// 这是程序与用户交互的核心函数，负责安全地获取用户输入的文件夹路径。
/// 它实现了一个健壮的输入循环，包含完整的验证和错误处理机制。
///
/// ## 交互流程
///
/// ### 输入提示
/// - 显示清晰的中文提示信息
/// - 说明期望的输入格式和要求
/// - 提供示例路径格式
///
/// ### 输入验证
/// 1. **非空检查**: 确保用户输入不为空
/// 2. **路径存在性**: 验证路径在文件系统中存在
/// 3. **目录检查**: 确认路径指向目录而非文件
/// 4. **权限验证**: 检查程序是否有读取权限
/// 5. **路径规范化**: 转换为绝对路径，解析符号链接
///
/// ### 错误处理
/// - 对每种错误提供具体的中文说明
/// - 给出可能的解决方案和建议
/// - 允许用户重新输入而不退出程序
///
/// ## 安全考虑
///
/// ### 路径安全
/// - 使用 `PathBuf::from()` 安全地构造路径
/// - 通过 `canonicalize()` 解析符号链接，防止路径遍历攻击
/// - 验证最终路径的有效性
///
/// ### 输入安全
/// - 使用 `trim()` 移除前后空白字符
/// - 处理各种异常输入情况
/// - 防止无限循环（通过合理的错误处理）
///
/// # 返回值
/// - `Ok(PathBuf)` - 经过验证和规范化的有效文件夹路径
/// - `Err(Box<dyn std::error::Error>)` - 不可恢复的 I/O 错误（如标准输入不可用）
///
/// # 错误处理
/// - 可恢复错误（如路径不存在）会提示用户重新输入
/// - 不可恢复错误（如 I/O 失败）会返回错误并终止函数
///
/// # 使用示例
/// ```rust
/// match get_folder_path_from_user() {
///     Ok(path) => println!("选择的路径: {}", path.display()),
///     Err(e) => eprintln!("获取路径失败: {}", e),
/// }
/// ```
pub fn get_folder_path_from_user() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 显示友好的欢迎信息和使用提示
    println!("\n📁 请选择要处理的音频文件夹");
    println!("💡 提示: 程序将递归扫描该文件夹及其所有子文件夹中的音频文件");
    println!("📝 支持的格式: WAV, MP3, FLAC, AAC, OGG, Opus, WMA, AIFF, ALAC");
    println!();

    loop {
        // 显示输入提示
        print!("请输入文件夹路径: ");
        io::stdout().flush()?;  // 确保提示信息立即显示

        // 读取用户输入
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let path_str = input.trim();

        // 检查输入是否为空
        if path_str.is_empty() {
            eprintln!("❌ 错误: 路径不能为空，请重新输入。");
            continue;
        }

        // 处理特殊输入
        if path_str == "quit" || path_str == "exit" || path_str == "q" {
            return Err("用户取消操作".into());
        }

        // 构造路径对象
        let path = PathBuf::from(path_str);

        // 验证路径的有效性
        match validate_folder_path(&path) {
            Ok(()) => {
                // 路径验证成功，尝试规范化
                match canonicalize_path(&path) {
                    Ok(canonical_path) => {
                        println!("✅ 路径验证成功: {}", canonical_path.display());
                        return Ok(canonical_path);
                    }
                    Err(e) => {
                        eprintln!("❌ 路径规范化失败: {}", e);
                        eprintln!("💡 建议: 请检查路径格式是否正确，或尝试使用绝对路径");
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ 路径验证失败: {}", e);
                eprintln!("💡 提示: 输入 'q' 或 'quit' 退出程序");
                continue;
            }
        }
    }
}

/// 规范化路径 (Canonicalize Path)
///
/// 将路径转换为绝对路径并解析所有符号链接。
/// 这个辅助函数封装了路径规范化的逻辑，提供更好的错误信息。
///
/// # 参数
/// - `path` - 要规范化的路径
///
/// # 返回值
/// - `Ok(PathBuf)` - 规范化后的绝对路径
/// - `Err(String)` - 规范化失败的详细错误信息
fn canonicalize_path(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize().map_err(|e| {
        format!(
            "无法规范化路径 '{}': {}。\n\
             可能的原因：\n\
             1. 路径包含无效字符\n\
             2. 路径中存在不可访问的符号链接\n\
             3. 文件系统权限问题\n\
             4. 路径格式不正确",
            path.display(),
            e
        )
    })
}

/// 验证文件夹路径的有效性 (Validate Folder Path)
///
/// 这个函数执行全面的路径验证，确保路径适合用于音频文件扫描。
/// 它检查路径的存在性、类型和访问权限，为后续的文件处理做好准备。
///
/// ## 验证步骤
///
/// ### 1. 存在性检查
/// 验证路径在文件系统中确实存在，避免后续的文件操作失败。
///
/// ### 2. 类型检查
/// 确认路径指向目录而非普通文件，因为程序需要递归扫描目录。
///
/// ### 3. 权限检查
/// 通过尝试读取目录内容来验证程序是否有足够的权限访问该目录。
/// 这可以提前发现权限问题，避免在处理过程中出现意外错误。
///
/// ### 4. 内容预检查（可选）
/// 可以检查目录是否包含任何文件，给用户提前反馈。
///
/// ## 错误分类
///
/// 不同类型的错误会返回不同的 `AppError::Path` 变体，包含：
/// - 路径不存在：可能是拼写错误或路径已被删除
/// - 不是目录：用户可能误选了文件而非文件夹
/// - 权限不足：需要管理员权限或文件夹被保护
///
/// # 参数
/// - `path` - 要验证的路径引用
///
/// # 返回值
/// - `Ok(())` - 路径验证通过，可以安全使用
/// - `Err(AppError::Path)` - 路径验证失败，包含详细错误信息
///
/// # 性能考虑
/// - 使用 `Path::exists()` 和 `Path::is_dir()` 进行快速检查
/// - 权限检查通过 `read_dir()` 实现，开销相对较小
/// - 避免递归遍历整个目录树，只检查顶层访问权限
pub fn validate_folder_path(path: &Path) -> Result<(), AppError> {
    // 检查路径是否存在
    if !path.exists() {
        return Err(AppError::Path(format!(
            "路径 '{}' 不存在。\n\
             请检查：\n\
             1. 路径拼写是否正确\n\
             2. 文件夹是否已被删除或移动\n\
             3. 是否使用了正确的路径分隔符",
            path.display()
        )));
    }

    // 检查是否为目录
    if !path.is_dir() {
        return Err(AppError::Path(format!(
            "路径 '{}' 不是一个目录。\n\
             请确保选择的是文件夹而不是文件。",
            path.display()
        )));
    }

    // 检查读取权限
    match std::fs::read_dir(path) {
        Ok(_) => {
            // 权限检查通过，可以进行进一步的内容检查
            Ok(())
        }
        Err(e) => {
            Err(AppError::Path(format!(
                "无法访问目录 '{}'。\n\
                 错误详情: {}\n\
                 可能的解决方案：\n\
                 1. 检查文件夹权限设置\n\
                 2. 以管理员身份运行程序\n\
                 3. 确保文件夹未被其他程序占用",
                path.display(),
                e
            )))
        }
    }
}

/// 对 LRA 结果文件进行排序 (Sort LRA Results File)
///
/// 这个函数负责读取、解析、排序和重写 LRA 结果文件。
/// 排序后的文件按照 LRA 值从高到低排列，便于用户快速识别动态范围的分布情况。
///
/// ## 处理流程
///
/// ### 1. 文件读取和解析
/// - 安全地打开和读取结果文件
/// - 跳过头部行，只处理数据行
/// - 解析每行的文件路径和 LRA 值
/// - 处理格式异常和解析错误
///
/// ### 2. 数据排序
/// - 使用高效的排序算法（通常是快速排序或归并排序）
/// - 按照 LRA 值进行降序排序（从高到低）
/// - 处理相同 LRA 值的情况（按文件路径排序）
///
/// ### 3. 文件重写
/// - 创建新的结果文件（覆盖原文件）
/// - 写入头部行
/// - 按排序顺序写入所有数据行
/// - 确保文件完整性和格式一致性
///
/// ## 错误处理策略
///
/// ### 文件操作错误
/// - 文件不存在或无法读取
/// - 磁盘空间不足或写入权限问题
/// - 文件被其他程序占用
///
/// ### 数据解析错误
/// - 行格式不符合预期
/// - LRA 值无法解析为数字
/// - 文件编码问题
///
/// ### 恢复机制
/// - 解析错误的行会被跳过并记录警告
/// - 部分数据损坏不会导致整个排序失败
/// - 提供详细的错误信息用于问题诊断
///
/// # 参数
/// - `results_file_path` - 结果文件的路径引用
/// - `header_line` - 文件头部说明行（用于重写文件时保持格式）
///
/// # 返回值
/// - `Ok(())` - 排序成功完成，文件已更新
/// - `Err(Box<dyn std::error::Error>)` - 文件操作或解析过程中发生错误
///
/// # 性能特性
/// - 时间复杂度: O(n log n)，其中 n 是文件行数
/// - 空间复杂度: O(n)，需要将所有数据加载到内存中
/// - 对于大文件（>10万行），可能需要考虑流式排序
pub fn sort_lra_results_file(
    results_file_path: &Path,
    header_line: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 正在排序结果文件: {}", results_file_path.display());

    // 读取和解析文件内容
    let entries = read_and_parse_results_file(results_file_path)?;

    // 检查是否有有效数据需要排序
    if entries.is_empty() {
        println!("📝 结果文件为空或没有有效数据，创建仅包含表头的文件。");
        write_results_file(results_file_path, header_line, &[])?;
        return Ok(());
    }

    // 对数据进行排序
    let sorted_entries = sort_entries_by_lra(entries);

    // 写入排序后的结果
    write_results_file(results_file_path, header_line, &sorted_entries)?;

    println!("✅ 排序完成，共处理 {} 个条目", sorted_entries.len());
    Ok(())
}

/// 读取和解析结果文件 (Read and Parse Results File)
///
/// 从结果文件中读取所有数据行，解析文件路径和 LRA 值。
/// 这个函数处理各种解析错误，确保部分数据损坏不会导致整个过程失败。
///
/// # 参数
/// - `file_path` - 结果文件路径
///
/// # 返回值
/// - `Ok(Vec<(String, f64)>)` - 成功解析的条目列表
/// - `Err(...)` - 文件读取错误
fn read_and_parse_results_file(
    file_path: &Path
) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut lines_iter = reader.lines();
    let mut line_number = 0;
    let mut skipped_lines = 0;

    // 跳过第一行（表头）
    if let Some(first_line) = lines_iter.next() {
        line_number += 1;
        let _ = first_line?; // 检查是否有读取错误，但不使用内容
    } else {
        // 文件为空
        return Ok(entries);
    }

    // 处理数据行
    for line_result in lines_iter {
        line_number += 1;
        let line = line_result?;

        // 跳过空行
        if line.trim().is_empty() {
            continue;
        }

        // 解析行内容
        match parse_result_line(&line) {
            Ok((path, lra)) => {
                entries.push((path, lra));
            }
            Err(e) => {
                eprintln!(
                    "⚠️  排序时警告 (第 {} 行): {}",
                    line_number, e
                );
                skipped_lines += 1;
            }
        }
    }

    if skipped_lines > 0 {
        println!(
            "📋 解析完成: 成功 {} 行，跳过 {} 行无效数据",
            entries.len(), skipped_lines
        );
    }

    Ok(entries)
}

/// 解析单行结果数据 (Parse Single Result Line)
///
/// 解析格式为 "文件路径 - LRA值" 的单行数据。
///
/// # 参数
/// - `line` - 要解析的行内容
///
/// # 返回值
/// - `Ok((String, f64))` - 解析成功的文件路径和 LRA 值
/// - `Err(String)` - 解析失败的错误信息
pub fn parse_result_line(line: &str) -> Result<(String, f64), String> {
    match line.rsplit_once(" - ") {
        Some((path_part, lra_str_part)) => {
            let lra_str = lra_str_part.trim();
            match lra_str.parse::<f64>() {
                Ok(lra_value) => {
                    // 验证 LRA 值的合理性
                    if lra_value.is_finite() && lra_value >= 0.0 {
                        Ok((path_part.to_string(), lra_value))
                    } else {
                        Err(format!(
                            "LRA 值 '{}' 超出合理范围 (应为非负有限数)",
                            lra_str
                        ))
                    }
                }
                Err(e) => Err(format!(
                    "无法解析 LRA 值 '{}': {}",
                    lra_str, e
                ))
            }
        }
        None => Err(format!(
            "行格式不正确: '{}' (期望格式: '文件路径 - LRA值')",
            line
        ))
    }
}

/// 对条目按 LRA 值排序 (Sort Entries by LRA Value)
///
/// 使用稳定排序算法按 LRA 值降序排列，LRA 值相同时按文件路径排序。
///
/// # 参数
/// - `mut entries` - 要排序的条目列表
///
/// # 返回值
/// - 排序后的条目列表
pub fn sort_entries_by_lra(mut entries: Vec<(String, f64)>) -> Vec<(String, f64)> {
    entries.sort_by(|a, b| {
        // 首先按 LRA 值降序排序
        match b.1.total_cmp(&a.1) {
            std::cmp::Ordering::Equal => {
                // LRA 值相同时，按文件路径升序排序
                a.0.cmp(&b.0)
            }
            other => other,
        }
    });
    entries
}

/// 写入结果文件 (Write Results File)
///
/// 将排序后的结果写入文件，包含表头和所有数据行。
///
/// # 参数
/// - `file_path` - 输出文件路径
/// - `header_line` - 表头行内容
/// - `entries` - 要写入的数据条目
///
/// # 返回值
/// - `Ok(())` - 写入成功
/// - `Err(...)` - 写入失败
fn write_results_file(
    file_path: &Path,
    header_line: &str,
    entries: &[(String, f64)]
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = BufWriter::new(File::create(file_path)?);

    // 写入表头
    writeln!(writer, "{}", header_line)?;

    // 写入数据行
    for (path_str, lra) in entries {
        writeln!(writer, "{} - {:.1}", path_str, lra)?;
    }

    // 确保数据写入磁盘
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// 测试文件夹路径验证功能
    #[test]
    fn test_validate_folder_path() {
        // 测试当前目录（应该有效）
        let current_dir = std::env::current_dir().expect("无法获取当前目录");
        assert!(validate_folder_path(&current_dir).is_ok());

        // 测试不存在的路径
        let non_existent = Path::new("/this/path/should/not/exist/12345");
        let result = validate_folder_path(non_existent);
        assert!(result.is_err());

        if let Err(AppError::Path(msg)) = result {
            assert!(msg.contains("不存在"));
        } else {
            panic!("期望得到 AppError::Path 错误");
        }

        // 测试文件而非目录（使用 Cargo.toml）
        let file_path = Path::new("Cargo.toml");
        if file_path.exists() {
            let result = validate_folder_path(file_path);
            assert!(result.is_err());

            if let Err(AppError::Path(msg)) = result {
                assert!(msg.contains("不是一个目录"));
            }
        }
    }

    /// 测试路径规范化功能
    #[test]
    fn test_canonicalize_path() {
        // 测试当前目录
        let current_dir = Path::new(".");
        let result = canonicalize_path(current_dir);
        assert!(result.is_ok());

        // 测试不存在的路径
        let non_existent = Path::new("/this/path/does/not/exist");
        let result = canonicalize_path(non_existent);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无法规范化路径"));
    }

    /// 测试结果行解析功能
    #[test]
    fn test_parse_result_line() {
        // 测试正常格式的行
        let normal_line = "music/song.mp3 - 12.5";
        let result = parse_result_line(normal_line);
        assert!(result.is_ok());
        let (path, lra) = result.unwrap();
        assert_eq!(path, "music/song.mp3");
        assert_eq!(lra, 12.5);

        // 测试带空格的行
        let spaced_line = "  music/song with spaces.wav  -  8.3  ";
        let result = parse_result_line(spaced_line);
        assert!(result.is_ok());
        let (path, lra) = result.unwrap();
        assert_eq!(path, "  music/song with spaces.wav ");
        assert_eq!(lra, 8.3);

        // 测试格式错误的行
        let invalid_line = "invalid format";
        let result = parse_result_line(invalid_line);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("行格式不正确"));

        // 测试无效的 LRA 值
        let invalid_lra = "music/song.mp3 - not_a_number";
        let result = parse_result_line(invalid_lra);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无法解析 LRA 值"));

        // 测试负数 LRA 值（应该被拒绝）
        let negative_lra = "music/song.mp3 - -5.0";
        let result = parse_result_line(negative_lra);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("超出合理范围"));

        // 测试无穷大值
        let infinite_lra = "music/song.mp3 - inf";
        let result = parse_result_line(infinite_lra);
        assert!(result.is_err());
    }

    /// 测试条目排序功能
    #[test]
    fn test_sort_entries_by_lra() {
        let entries = vec![
            ("file1.mp3".to_string(), 8.5),
            ("file2.wav".to_string(), 15.2),
            ("file3.flac".to_string(), 12.1),
            ("file4.m4a".to_string(), 15.2), // 相同的 LRA 值
            ("file5.ogg".to_string(), 5.3),
        ];

        let sorted = sort_entries_by_lra(entries);

        // 验证按 LRA 值降序排列
        assert_eq!(sorted[0].1, 15.2);
        assert_eq!(sorted[1].1, 15.2);
        assert_eq!(sorted[2].1, 12.1);
        assert_eq!(sorted[3].1, 8.5);
        assert_eq!(sorted[4].1, 5.3);

        // 验证相同 LRA 值时按文件名排序
        assert!(sorted[0].0 < sorted[1].0); // file2.wav < file4.m4a
    }

    /// 测试结果文件写入功能
    #[test]
    fn test_write_results_file() {
        let temp_dir = TempDir::new().expect("无法创建临时目录");
        let test_file = temp_dir.path().join("test_results.txt");

        let header = "文件路径 (相对) - LRA 数值 (LU)";
        let entries = vec![
            ("file1.mp3".to_string(), 12.5),
            ("file2.wav".to_string(), 8.3),
            ("file3.flac".to_string(), 15.7),
        ];

        // 写入文件
        let result = write_results_file(&test_file, header, &entries);
        assert!(result.is_ok());

        // 验证文件内容
        let content = fs::read_to_string(&test_file).expect("无法读取文件");
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 4); // 表头 + 3 个数据行
        assert_eq!(lines[0], header);
        assert_eq!(lines[1], "file1.mp3 - 12.5");
        assert_eq!(lines[2], "file2.wav - 8.3");
        assert_eq!(lines[3], "file3.flac - 15.7");
    }

    /// 测试读取和解析结果文件功能
    #[test]
    fn test_read_and_parse_results_file() {
        let temp_dir = TempDir::new().expect("无法创建临时目录");
        let test_file = temp_dir.path().join("test_results.txt");

        // 创建测试文件内容
        let content = r#"文件路径 (相对) - LRA 数值 (LU)
file1.mp3 - 12.5
file2.wav - 8.3

file3.flac - 15.7
invalid line format
file4.m4a - not_a_number
file5.ogg - 9.1"#;

        fs::write(&test_file, content).expect("无法写入测试文件");

        // 读取和解析文件
        let result = read_and_parse_results_file(&test_file);
        assert!(result.is_ok());

        let entries = result.unwrap();
        assert_eq!(entries.len(), 4); // 应该成功解析 4 个有效条目

        // 验证解析的条目
        assert_eq!(entries[0], ("file1.mp3".to_string(), 12.5));
        assert_eq!(entries[1], ("file2.wav".to_string(), 8.3));
        assert_eq!(entries[2], ("file3.flac".to_string(), 15.7));
        assert_eq!(entries[3], ("file5.ogg".to_string(), 9.1));
    }

    /// 测试完整的结果文件排序功能
    #[test]
    fn test_sort_lra_results_file() {
        let temp_dir = TempDir::new().expect("无法创建临时目录");
        let results_file = temp_dir.path().join("test_results.txt");

        // 创建未排序的测试文件
        let content = r#"文件路径 (相对) - LRA 数值 (LU)
file1.mp3 - 8.5
file2.wav - 15.2
file3.flac - 12.1
file4.m4a - 20.0
file5.ogg - 5.3"#;

        fs::write(&results_file, content).expect("无法写入测试文件");

        // 执行排序
        let header_line = "文件路径 (相对) - LRA 数值 (LU)";
        let result = sort_lra_results_file(&results_file, header_line);
        assert!(result.is_ok());

        // 验证排序结果
        let sorted_content = fs::read_to_string(&results_file).expect("无法读取排序后的文件");
        let lines: Vec<&str> = sorted_content.lines().collect();

        assert_eq!(lines.len(), 6); // 表头 + 5 个数据行
        assert_eq!(lines[0], header_line);
        assert!(lines[1].contains("file4.m4a - 20.0"));
        assert!(lines[2].contains("file2.wav - 15.2"));
        assert!(lines[3].contains("file3.flac - 12.1"));
        assert!(lines[4].contains("file1.mp3 - 8.5"));
        assert!(lines[5].contains("file5.ogg - 5.3"));
    }

    /// 测试空结果文件的排序
    #[test]
    fn test_sort_empty_results_file() {
        let temp_dir = TempDir::new().expect("无法创建临时目录");
        let results_file = temp_dir.path().join("empty_results.txt");

        // 创建只有表头的文件
        let header_line = "文件路径 (相对) - LRA 数值 (LU)";
        fs::write(&results_file, header_line).expect("无法写入测试文件");

        // 执行排序
        let result = sort_lra_results_file(&results_file, header_line);
        assert!(result.is_ok());

        // 验证文件内容保持不变
        let content = fs::read_to_string(&results_file).expect("无法读取文件");
        assert_eq!(content.trim(), header_line);
    }

    /// 测试不存在文件的排序处理
    #[test]
    fn test_sort_nonexistent_file() {
        let non_existent_file = Path::new("/this/file/does/not/exist.txt");
        let header_line = "文件路径 (相对) - LRA 数值 (LU)";

        let result = sort_lra_results_file(non_existent_file, header_line);
        assert!(result.is_err());
    }
}
