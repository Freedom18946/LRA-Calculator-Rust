# 快速开始指南 (Quick Start Guide)

欢迎使用 LRA 音频响度范围计算器！本指南将帮助您在 5 分钟内快速上手使用这个工具。

## 🎯 目标用户

本工具适合以下用户：
- **音频工程师** - 需要分析音频动态范围的专业人士
- **音乐制作人** - 希望了解音乐作品响度特性的创作者
- **音频研究人员** - 进行音频质量分析和研究的学者
- **音频爱好者** - 对音频技术感兴趣的发烧友

## ⚡ 5分钟快速体验

### 第一步：环境准备

确保您的系统已安装以下软件：

1. **Rust 编程语言环境**
   ```bash
   # 检查 Rust 是否已安装
   rustc --version
   cargo --version
   
   # 如果未安装，请访问 https://rustup.rs/ 安装
   ```

2. **FFmpeg 音频处理工具**
   ```bash
   # 检查 FFmpeg 是否已安装
   ffmpeg -version
   
   # macOS 安装方法
   brew install ffmpeg
   
   # Ubuntu/Debian 安装方法
   sudo apt update && sudo apt install ffmpeg
   
   # Windows 安装方法
   # 请从 https://ffmpeg.org/download.html 下载并添加到 PATH
   ```

### 第二步：获取和构建项目

```bash
# 1. 克隆项目（如果从 Git 仓库获取）
git clone <repository-url>
cd LRA-Calculator-Rust

# 或者如果您已有项目文件，直接进入项目目录
cd LRA-Calculator-Rust

# 2. 构建发布版本（推荐，性能更好）
cargo build --release

# 3. 验证构建成功
ls -la target/release/LRA-Calculator-Rust
```

### 第三步：准备测试音频文件

为了快速体验，您需要准备一些音频文件：

```bash
# 创建测试目录结构
mkdir -p ~/test-audio/album1
mkdir -p ~/test-audio/album2

# 将您的音频文件复制到测试目录
# 支持的格式：.wav, .mp3, .flac, .m4a, .aac, .ogg, .opus, .wma, .aiff, .alac
```

### 第四步：运行程序

```bash
# 运行 LRA 计算器
./target/release/LRA-Calculator-Rust
```

程序启动后，您将看到类似以下的界面：

```
欢迎使用音频 LRA 计算器（高性能版 - 直接分析）！
当前时间: 2025-08-07 14:30:00
✓ FFmpeg 检测成功
请输入要递归处理的音乐顶层文件夹路径: 
```

### 第五步：输入路径并查看结果

1. **输入音频文件夹路径**：
   ```
   请输入要递归处理的音乐顶层文件夹路径: /Users/yourname/test-audio
   ```

2. **观察处理过程**：
   ```
   正在递归扫描文件夹: /Users/yourname/test-audio
   扫描完成，找到 15 个音频文件待处理。
   开始多线程直接分析...
     [线程 ThreadId(2)] (1/15) 直接分析: album1/track01.mp3
     [线程 ThreadId(3)] (2/15) 直接分析: album1/track02.flac
     [线程 ThreadId(4)] (3/15) 直接分析: album2/song01.wav
   ```

3. **查看分析结果**：
   ```
   处理完成！
   成功处理: 15 个文件
   处理失败: 0 个文件
   结果写入完成。
   结果文件 /Users/yourname/test-audio/lra_results.txt 已成功排序。
   ```

### 第六步：理解结果文件

程序会在您指定的目录中生成 `lra_results.txt` 文件：

```
文件路径 (相对) - LRA 数值 (LU)
classical/symphony_no_5.wav - 18.5
rock/stairway_to_heaven.flac - 14.2
jazz/take_five.mp3 - 12.8
pop/single.m4a - 8.1
podcast/episode_01.mp3 - 5.5
```

**LRA 值解读**：
- **高 LRA (15+ LU)**: 动态范围大，音量变化丰富（古典音乐、爵士乐）
- **中等 LRA (8-15 LU)**: 适中的动态范围（摇滚、流行音乐）
- **低 LRA (0-8 LU)**: 动态范围小，音量较为平稳（现代流行、播客）

## 🎉 恭喜！

您已经成功完成了第一次 LRA 分析！现在您可以：

1. **分析更多音频文件** - 尝试不同类型的音频内容
2. **比较不同格式** - 观察有损和无损格式的 LRA 差异
3. **深入学习** - 查看 [用户手册](./user-guide.md) 了解更多功能
4. **了解技术细节** - 阅读 [LRA 算法说明](./lra-algorithm.md)

## 🔧 常见问题快速解决

### 问题：FFmpeg 未找到
```bash
错误: 未找到 FFmpeg，请确保已安装并添加到 PATH 环境变量中
```
**解决方案**：请按照第一步的说明安装 FFmpeg。

### 问题：没有找到音频文件
```bash
在指定路径下没有找到支持的音频文件。
```
**解决方案**：确保目录中包含支持的音频格式文件。

### 问题：权限错误
```bash
权限被拒绝 (os error 13)
```
**解决方案**：确保对目标目录有读写权限。

## 📚 下一步

- 📖 [用户手册](./user-guide.md) - 详细的功能说明
- 🏗️ [架构设计](./architecture.md) - 了解程序工作原理
- ❓ [常见问题](./faq.md) - 更多问题解答
- 🚀 [性能基准](./benchmarks.md) - 性能测试结果

---

*如有问题，请查看 [常见问题](./faq.md) 或创建 GitHub Issue。*
