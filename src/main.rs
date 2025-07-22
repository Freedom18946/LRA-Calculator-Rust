use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;
use regex::Regex;
use walkdir::WalkDir;
// use tempfile::Builder as TempFileBuilder; // <--- 不再需要 tempfile
use chrono::Local;

const SUPPORTED_EXTENSIONS: [&str; 10] = [
    "wav", "mp3", "m4a", "flac", "aac", "ogg", "opus", "wma", "aiff", "alac",
];

#[derive(Debug)]
struct ProcessFileError {
    file_path: String,
    message: String,
}

impl std::fmt::Display for ProcessFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "File '{}': {}", self.file_path, self.message)
    }
}
impl std::error::Error for ProcessFileError {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("欢迎使用音频 LRA 计算器（高性能版 - 直接分析）！");
    println!("当前时间: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));

    let base_folder_path = get_folder_path_from_user()?;
    println!("正在递归扫描文件夹: {}", base_folder_path.display());

    let results_file_path = base_folder_path.join("lra_results.txt");
    let header_line = "文件路径 (相对) - LRA 数值 (LU)";

    let mut files_to_process: Vec<(PathBuf, String)> = Vec::new();
    for entry_result in WalkDir::new(&base_folder_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let current_file_path = entry_result.path().to_path_buf();
        if current_file_path == results_file_path {
            continue;
        }
        if let Some(extension) = current_file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
        {
            if SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
                let display_path_str = current_file_path
                    .strip_prefix(&base_folder_path)
                    .unwrap_or(&current_file_path)
                    .to_string_lossy()
                    .into_owned();
                files_to_process.push((current_file_path, display_path_str));
            }
        }
    }

    if files_to_process.is_empty() {
        println!("在指定路径下没有找到支持的音频文件。");
        let mut writer = BufWriter::new(File::create(&results_file_path)?);
        writeln!(writer, "{}", header_line)?;
        writer.flush()?;
        return Ok(());
    }
    println!(
        "扫描完成，找到 {} 个音频文件待处理。",
        files_to_process.len()
    );
    println!("开始多线程直接分析...");

    let total_files = files_to_process.len();
    let processed_count = AtomicUsize::new(0);

    let processing_results: Vec<Result<(String, f64), ProcessFileError>> = files_to_process
        .into_par_iter()
        .map(|(current_file_path, display_path_str)| {
            let current_processed_atomic = processed_count.fetch_add(1, Ordering::SeqCst) + 1;
            println!(
                "  [线程 {:?}] ({}/{}) 直接分析: {}",
                std::thread::current().id(),
                current_processed_atomic,
                total_files,
                display_path_str
            );

            // 直接调用 calculate_lra，传入原始文件路径
            // 不再有转换步骤和临时文件
            match calculate_lra_direct(&current_file_path) {
                // <--- 调用新命名的函数
                Ok(lra) => {
                    println!(
                        "    [线程 {:?}] ({}/{}) 分析成功: {} LRA: {:.1} LU",
                        std::thread::current().id(),
                        current_processed_atomic,
                        total_files,
                        display_path_str,
                        lra
                    );
                    Ok((display_path_str, lra))
                }
                Err(e) => {
                    let err_msg = format!("分析失败: {}", e);
                    Err(ProcessFileError {
                        file_path: display_path_str,
                        message: err_msg,
                    })
                }
            }
        })
        .collect();

    println!("\n并行分析阶段完成。");

    let mut writer = BufWriter::new(File::create(&results_file_path)?);
    writeln!(writer, "{}", header_line)?;
    let mut actual_successes = 0;
    let mut actual_failures = 0;
    let mut error_messages_collected: Vec<String> = Vec::new();

    for result in processing_results {
        match result {
            Ok((path_str, lra)) => {
                writeln!(writer, "{} - {:.1}", path_str, lra)?;
                actual_successes += 1;
            }
            Err(e) => {
                error_messages_collected.push(format!("文件 '{}': {}", e.file_path, e.message));
                actual_failures += 1;
            }
        }
    }
    writer.flush()?;

    println!("结果写入完成。");
    println!("成功处理 {} 个文件。", actual_successes);
    if actual_failures > 0 {
        println!("{} 个文件处理失败。详情如下:", actual_failures);
        for err_msg in error_messages_collected {
            eprintln!("  - {}", err_msg);
        }
    }

    if actual_successes > 0 {
        match sort_lra_results_file(&results_file_path, header_line) {
            Ok(_) => println!("结果文件 {} 已成功排序。", results_file_path.display()),
            Err(e) => eprintln!(
                "错误：排序结果文件 {} 失败: {}",
                results_file_path.display(),
                e
            ),
        }
    } else {
        println!("没有成功处理的文件，跳过排序。");
    }

    println!(
        "所有操作完成！结果已保存于: {}",
        results_file_path.display()
    );
    println!("结束时间: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
    Ok(())
}

fn get_folder_path_from_user() -> Result<PathBuf, Box<dyn std::error::Error>> {
    loop {
        print!("请输入要递归处理的音乐顶层文件夹路径: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let path_str = input.trim();
        if path_str.is_empty() {
            eprintln!("错误: 路径不能为空，请重新输入。");
            continue;
        }
        let path = PathBuf::from(path_str);
        if path.is_dir() {
            match path.canonicalize() {
                Ok(canonical_path) => return Ok(canonical_path),
                Err(e) => eprintln!(
                    "错误: 无法规范化路径 '{}': {}. 请确保路径有效且程序有权限访问。",
                    path.display(),
                    e
                ),
            }
        } else {
            eprintln!(
                "错误: \"{}\" 不是一个有效的文件夹路径或文件夹不存在，请重新输入。",
                path.display()
            );
        }
    }
}

// 移除了 convert_to_flac 函数

// calculate_lra 函数被重命名并修改为直接处理原始音频文件
fn calculate_lra_direct(
    audio_file_path: &Path,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(audio_file_path) // <--- 直接使用原始音频文件路径
        .arg("-filter_complex")
        .arg("ebur128")
        .arg("-f")
        .arg("null")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("info") // ebur128 输出在 info 级别
        .arg("-")
        .output()?;

    let stderr_output = String::from_utf8_lossy(&output.stderr);

    let re = Regex::new(r"LRA:\s*([\d\.-]+)\s*LU")?;
    if let Some(caps) = re.captures_iter(&stderr_output).last() {
        if let Some(lra_str) = caps.get(1) {
            return lra_str.as_str().parse::<f64>().map_err(|e| {
                format!(
                    "解析LRA值 '{}' (来自文件 {}) 失败: {}",
                    lra_str.as_str(),
                    audio_file_path.display(),
                    e
                )
                .into()
            });
        }
    }
    Err(format!(
        "无法从 ffmpeg 输出中为文件 {} 解析 LRA 值. stderr: {}",
        audio_file_path.display(),
        stderr_output.trim()
    )
    .into())
}

fn sort_lra_results_file(
    results_file_path: &Path,
    header_line: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // (此函数与上一版本相同，无需修改)
    println!("\n正在排序结果文件: {}", results_file_path.display());
    let file = File::open(results_file_path)?;
    let reader = BufReader::new(file);
    let mut entries: Vec<(String, f64)> = Vec::new();
    let mut lines_iter = reader.lines();

    if lines_iter.next().is_none() {
        println!("结果文件为空或只有表头，无需排序。");
        let mut writer = BufWriter::new(File::create(results_file_path)?);
        writeln!(writer, "{}", header_line)?;
        writer.flush()?;
        return Ok(());
    }

    for line_result in lines_iter {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }
        match line.rsplit_once(" - ") {
            Some((path_part, lra_str_part)) => match lra_str_part.trim().parse::<f64>() {
                Ok(lra_value) => entries.push((path_part.to_string(), lra_value)),
                Err(e) => eprintln!(
                    "排序时警告: 无法解析行 '{}' 中的LRA值 '{}': {}",
                    line, lra_str_part, e
                ),
            },
            None => eprintln!("排序时警告: 行 '{}' 格式不符合预期。将被忽略。", line),
        }
    }

    entries.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));

    let mut writer = BufWriter::new(File::create(results_file_path)?);
    writeln!(writer, "{}", header_line)?;
    for (path_str, lra) in entries {
        writeln!(writer, "{} - {:.1}", path_str, lra)?;
    }
    writer.flush()?;
    Ok(())
}
