# Vocal2Midi

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md)

Vocal2Midi 可以把人声音频转换为带歌词对齐信息的 MIDI、USTX、TextGrid 以及辅助编辑产物。项目包含 Fluent 桌面 GUI、本地 Web UI、批量 slice + ASR CLI，以及以 ONNX 为主的推理流水线。

## 项目状态

- 当前运行时以 ONNX 为主。
- 渐进式 Rust 重写正在 `rewrite-in-rust/` 下进行。当前面向用户的应用仍由
  Python 主导；已 `verified` 的 Rust 单元只有在明确的兼容性和回滚检查后才会
  `promoted`。
- Windows 在可用时会对 ONNX 模型使用 DirectML。
- Windows Qwen3-ASR 使用项目内置的 ONNX encoder + GGUF/`llama.cpp` decoder 路径。
- Linux 和 macOS Qwen3-ASR 使用官方 `qwen-asr` Transformers 后端。
- Linux 和 macOS 对非 Qwen ONNX 模型使用标准 ONNX Runtime CPU 执行。
- 模型资源预期放在 `experiments/` 下，或放在用户配置的其他本地路径。
- 代码库仍在清理中，因此还保留了一些历史函数名和兼容分支。

## 使用者

### 安装

推荐开发环境使用 Python 3.12 和 `uv`：

```bash
uv python install 3.12
uv python pin 3.12
uv sync
```

也可以使用平台辅助脚本：

```bash
./install.sh
./run.sh
```

Windows 便携环境：

```bat
install.bat
run.bat
```

Linux 专用安装说明见 [docs/linux.md](docs/linux.md)。官方上游 Qwen3-ASR 独立安装说明见 [docs/qwen-linux.md](docs/qwen-linux.md)。

### 下载模型

查看模型下载计划：

```bash
uv run python download_models.py --list
```

下载缺失的模型资源：

```bash
uv run python download_models.py
```

需要时可以明确选择 Qwen3-ASR 来源：

```bash
uv run python download_models.py --qwen-source modelscope
uv run python download_models.py --qwen-source huggingface
```

默认模型路径：

| 组件 | 默认路径 |
| --- | --- |
| GAME | `experiments/GAME-1.0.3-medium-onnx` |
| HubertFA | `experiments/1218_hfa_model_new_dict` |
| Linux/macOS/Web 的 Qwen3-ASR | `experiments/Qwen3-ASR-1.7B` |
| Windows 桌面端的 Qwen3-ASR | `experiments/Qwen3-ASR-1.7B-dml` |
| 日文 mora ASR | `experiments/romajiASR` |
| RMVPE | `experiments/RMVPE/rmvpe.onnx` |

你可以在桌面 GUI 设置面板或 Web UI 设置页中修改模型路径。

### 桌面 GUI

启动桌面 GUI：

```bash
uv run python app_fluent.py
```

桌面 GUI 是主要的交互式工作流。它可以选择模型路径、运行设备、切片设置、语言和歌词模式，提供可选参考歌词，并导出 MIDI/USTX/调试产物。

### Web UI

启动本地 Web 后端：

```bash
uv run python web_server.py
```

然后打开：

```text
http://localhost:5000
```

需要自定义端口时：

```bash
V2M_WEB_PORT=5001 uv run python web_server.py
```

Web API 契约见 [docs/web-api.md](docs/web-api.md)。

### 批量 CLI

运行按文件夹处理的 slice + ASR 流程：

```bash
uv run python scripts/slice_asr_cli.py <input_dir> <output_dir> \
  --asr-model experiments/Qwen3-ASR-1.7B \
  --device cpu \
  --language zh
```

在 Windows 桌面环境中，如果对应模型目录可用，使用 Windows Qwen 路径和 DirectML 设备：

```bash
uv run python scripts/slice_asr_cli.py <input_dir> <output_dir> \
  --asr-model experiments/Qwen3-ASR-1.7B-dml \
  --device dml \
  --language zh
```

常用选项：

```text
--no-slice              bypass slicing and send the whole file to ASR
--asr-batch-size        ASR batch size
--file-batch-size       number of audio files per batch
--rmvpe-model           enable RMVPE-assisted smart slicing
--rmvpe-batch-size      RMVPE batch size
--keep-model            keep the ASR runtime alive across the batch
--keep-rmvpe            keep the RMVPE runtime alive across the batch
--save-json             save slice timing and ASR outputs as JSON
--no-recursive          scan only the top level
--no-skip-existing      force reprocessing of existing outputs
```

### 输出

根据所选工作流，Vocal2Midi 可以导出：

- `.mid`
- `.ustx`
- `.txt`
- `.csv`
- `TextGrid`
- chunk `.wav` files
- `.lab`
- ASR matching logs

## 开发者

### 架构

预期依赖方向是：

```text
gui -> application -> inference
web -> application -> inference
```

主要区域：

- `application/`：应用层配置、校验和任务入口
- `gui/`：PyQt5 + qfluentwidgets 桌面 UI
- `web_server.py`：Flask + SocketIO 本地 Web 后端
- `inference/`：ASR、对齐、音高提取、切片、量化、导出
- `scripts/`：模型/源码维护和批量 CLI 辅助脚本
- `tests/`：自动化测试

架构说明见 [docs/architecture.md](docs/architecture.md)。开发工作流和文档规则见 [docs/contributing.md](docs/contributing.md)。

### Rust Workspace

Rust 迁移工作位于 [rewrite-in-rust/](rewrite-in-rust/)。Cargo workspace 有意嵌套在
[rewrite-in-rust/rust/](rewrite-in-rust/rust/) 下，这样 Rust library units
可以在不启动桌面 GUI、Web 后端或完整模型流水线的情况下独立测试。

常用 Rust 检查：

```bash
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
```

Rust workspace README 记录了 MSRV、crate 边界、JSON bridge 契约和 migration-owner 规则：
[rewrite-in-rust/rust/README.md](rewrite-in-rust/rust/README.md)。

### 测试

运行聚焦的 Web/API 测试套件：

```bash
uv run pytest tests/test_web_api.py
```

运行全部自动化测试：

```bash
uv run pytest
```

手动集成测试。先启动 `web_server.py`，然后运行：

```bash
uv run python tests/test_api_integration.py
```

### 源码镜像

vendored 第三方源码镜像位于 `third_party/` 下。使用以下命令刷新并审计：

```bash
uv run python scripts/vendor_sources.py --force
uv run python scripts/vendor_native_sources.py --force
uv run python scripts/audit_vendored_sources.py
```

## 文档

- Linux 安装: [docs/linux.md](docs/linux.md)
- Qwen3-ASR Linux 说明: [docs/qwen-linux.md](docs/qwen-linux.md)
- 架构: [docs/architecture.md](docs/architecture.md)
- 开发指南: [docs/contributing.md](docs/contributing.md)
- Web API 契约: [docs/web-api.md](docs/web-api.md)
- 第三方致谢: [ACKNOWLEDGEMENTS.md](ACKNOWLEDGEMENTS.md)

## 许可证

Vocal2Midi 以 Apache License 2.0 分发。见 [LICENSE](LICENSE)。

第三方组件、vendored code、模型资源、词典和嵌入材料可能带有各自原始的许可证、通知或署名要求。这些通知仍适用于对应材料。
