# 常见问题 (Frequently Asked Questions)

本文档收集了用户在使用 LRA 音频响度范围计算器时遇到的常见问题和解决方案。

## 📋 目录

1. [安装和环境问题](#安装和环境问题)
2. [使用和操作问题](#使用和操作问题)
3. [结果和数据问题](#结果和数据问题)
4. [性能和优化问题](#性能和优化问题)
5. [技术和原理问题](#技术和原理问题)

## 🛠️ 安装和环境问题

### Q1: 提示 "FFmpeg 未找到" 错误怎么办？

**错误信息**:
```
错误: 未找到 FFmpeg，请确保已安装并添加到 PATH 环境变量中
```

**解决方案**:

1. **检查 FFmpeg 是否已安装**:
   ```bash
   # Linux/macOS
   which ffmpeg
   ffmpeg -version
   
   # Windows
   where ffmpeg
   ffmpeg -version
   ```

2. **安装 FFmpeg**:
   ```bash
   # macOS
   brew install ffmpeg
   
   # Ubuntu/Debian
   sudo apt update && sudo apt install ffmpeg
   
   # Windows (使用 Chocolatey)
   choco install ffmpeg
   ```

3. **手动添加到 PATH**:
   - Windows: 将 FFmpeg 的 bin 目录添加到系统环境变量 PATH 中
   - Linux/macOS: 在 ~/.bashrc 或 ~/.zshrc 中添加 `export PATH="/path/to/ffmpeg/bin:$PATH"`

### Q2: Rust 编译失败怎么办？

**常见编译错误**:

1. **链接器错误**:
   ```bash
   # Ubuntu/Debian
   sudo apt install build-essential
   
   # CentOS/RHEL
   sudo yum groupinstall "Development Tools"
   ```

2. **OpenSSL 相关错误**:
   ```bash
   # Ubuntu/Debian
   sudo apt install pkg-config libssl-dev
   
   # CentOS/RHEL
   sudo yum install openssl-devel
   ```

3. **Rust 版本过旧**:
   ```bash
   rustup update
   ```

### Q3: 在 Windows 上运行出现编码问题？

**问题**: 中文路径或文件名显示乱码

**解决方案**:
1. 确保 Windows 系统区域设置支持 UTF-8
2. 使用 Windows Terminal 或支持 UTF-8 的终端
3. 设置环境变量: `set RUST_LOG=debug`

## 🎯 使用和操作问题

### Q4: 程序处理速度很慢怎么办？

**可能原因和解决方案**:

1. **硬件资源不足**:
   - 检查 CPU 使用率: `top` (Linux/macOS) 或任务管理器 (Windows)
   - 确保有足够的内存
   - 考虑使用 SSD 硬盘

2. **网络存储延迟**:
   - 将文件复制到本地硬盘处理
   - 使用有线网络连接
   - 检查网络存储的性能

3. **文件数量过多**:
   - 分批处理大型音乐库
   - 使用 `nice` 命令降低进程优先级

### Q5: 某些音频文件被跳过了？

**检查清单**:

1. **文件格式支持**:
   - 确认文件扩展名在支持列表中
   - 支持的格式: wav, mp3, m4a, flac, aac, ogg, opus, wma, aiff, alac

2. **文件权限**:
   ```bash
   # 检查文件权限
   ls -la /path/to/audio/file
   
   # 修复权限
   chmod 644 /path/to/audio/file
   ```

3. **文件完整性**:
   ```bash
   # 使用 FFmpeg 检查文件
   ffmpeg -v error -i "audio_file.mp3" -f null -
   ```

### Q6: 如何处理包含空格的路径？

**解决方案**:
- 程序会自动处理包含空格的路径
- 如果遇到问题，可以使用引号包围路径:
  ```
  请输入路径: "/Users/username/My Music/Collection"
  ```

### Q7: 可以中断正在运行的处理吗？

**中断方法**:
- 使用 `Ctrl+C` (Linux/macOS) 或 `Ctrl+Break` (Windows)
- 程序会安全退出，已处理的结果会保留
- 可以重新运行程序继续处理剩余文件

## 📊 结果和数据问题

### Q8: LRA 值为 0 或异常高是什么原因？

**LRA 值异常的可能原因**:

1. **LRA = 0**:
   - 音频内容完全静音
   - 音频被极度压缩（响度战争）
   - 单声道或立体声问题

2. **LRA 异常高 (>25 LU)**:
   - 音频包含长时间静音段
   - 录音质量问题
   - 文件损坏

**验证方法**:
```bash
# 手动检查音频文件
ffmpeg -i "suspicious_file.mp3" -filter_complex ebur128 -f null -
```

### Q9: 同一文件不同格式的 LRA 值不同？

**这是正常现象**:
- 有损压缩会轻微影响动态范围
- 不同编码器的算法差异
- 比特率和质量设置的影响

**典型差异范围**:
- 无损 vs 高质量有损: ±0.5 LU
- 无损 vs 低质量有损: ±1-2 LU

### Q10: 结果文件格式可以自定义吗？

**当前格式**:
```
文件路径 (相对) - LRA 数值 (LU)
```

**自定义处理**:
```bash
# 转换为 CSV 格式
awk -F' - ' 'NR>1 {print $1","$2}' lra_results.txt > results.csv

# 添加时间戳
sed "1i处理时间: $(date)" lra_results.txt > timestamped_results.txt
```

## 🚀 性能和优化问题

### Q11: 如何提高处理速度？

**优化建议**:

1. **硬件优化**:
   - 使用多核 CPU
   - 增加内存到 8GB+
   - 使用 SSD 硬盘

2. **软件优化**:
   ```bash
   # 使用发布版本
   cargo build --release
   
   # 设置高优先级
   nice -n -10 ./target/release/LRA-Calculator-Rust
   ```

3. **系统优化**:
   ```bash
   # 增加文件描述符限制
   ulimit -n 65536
   ```

### Q12: 内存使用过高怎么办？

**内存优化**:
- 程序设计为流式处理，内存使用应该稳定
- 如果内存持续增长，可能是系统问题
- 重启程序或系统

**监控内存使用**:
```bash
# Linux/macOS
top -p $(pgrep LRA-Calculator)

# Windows
任务管理器 → 详细信息 → 查找 LRA-Calculator-Rust.exe
```

## 🔬 技术和原理问题

### Q13: LRA 和其他动态范围指标有什么区别？

**主要区别**:

| 指标 | 测量方式 | 用途 |
|------|----------|------|
| LRA | EBU R128 标准，基于响度 | 广播、流媒体标准 |
| Peak-to-RMS | 峰值与均方根比值 | 传统音频工程 |
| Crest Factor | 峰值与 RMS 比值 | 信号分析 |
| Dynamic Range | 多种定义 | 通用术语 |

### Q14: 为什么选择 FFmpeg 而不是其他音频库？

**FFmpeg 优势**:
- 支持格式最广泛
- EBU R128 标准实现
- 行业标准工具
- 持续维护更新
- 跨平台兼容性好

### Q15: 程序的并行处理是如何工作的？

**并行处理机制**:
- 使用 Rust 的 Rayon 库
- 自动检测 CPU 核心数
- 每个文件独立处理
- 线程安全的进度报告
- 错误隔离（单个文件错误不影响其他文件）

### Q16: 可以修改 LRA 计算参数吗？

**当前实现**:
- 使用 FFmpeg 默认的 EBU R128 参数
- 积分时间: 400ms
- 门限: -70 LUFS

**自定义需求**:
- 需要修改源代码中的 FFmpeg 参数
- 不建议修改，以保持标准兼容性

## 🆘 获取更多帮助

### 报告问题

如果您的问题不在此列表中，请：

1. **检查现有 Issues**: 搜索 GitHub Issues 查看是否已有相关问题
2. **创建新 Issue**: 提供以下信息
   - 操作系统和版本
   - Rust 和 FFmpeg 版本
   - 错误信息的完整输出
   - 重现问题的步骤
   - 示例音频文件（如果可能）

### 社区支持

- 📖 查看 [用户手册](./user-guide.md) 获取详细使用说明
- 🏗️ 阅读 [架构设计](./architecture.md) 了解技术细节
- 🚀 参考 [性能基准](./benchmarks.md) 了解性能表现

### 贡献改进

- 🔧 查看 [开发指南](./development.md) 了解如何贡献代码
- 📝 帮助改进文档
- 🐛 报告和修复 Bug
- 💡 提出新功能建议

---

*本 FAQ 会根据用户反馈持续更新。如果您有新的问题或建议，请创建 GitHub Issue。*
