# Vocal2Midi

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md)

Vocal2Midi は、ボーカル録音から歌詞付きの MIDI、USTX、TextGrid、および編集用の補助成果物を生成するツールです。Fluent デスクトップ GUI、ローカル Web UI、バッチ slice + ASR CLI、ONNX 優先の推論パイプラインを含みます。

## プロジェクトの状態

- 現在の実行系は ONNX 優先です。
- 段階的な Rust 書き換えは `rewrite-in-rust/` 以下で進行中です。現在のユーザー向け
  アプリケーションはまだ Python 主導であり、`verified` 済みの Rust unit も、
  明示的な互換性確認とロールバック確認を経てから `promoted` されます。
- Windows では利用可能な場合、ONNX モデルに DirectML を使用します。
- Windows の Qwen3-ASR は、プロジェクト内の ONNX エンコーダー + GGUF/`llama.cpp` デコーダー経路を使用します。
- Linux と macOS の Qwen3-ASR は、公式の `qwen-asr` Transformers バックエンドを使用します。
- Linux と macOS では、Qwen 以外の ONNX モデルに標準の ONNX Runtime CPU 実行を使用します。
- モデルアセットは `experiments/` または設定された別のローカルパスに置く前提です。
- コードベースの整理中のため、一部に古い関数名や互換用分岐が残っています。

## ユーザー向け

### インストール

推奨の開発環境は Python 3.12 と `uv` を使用します。

```bash
uv python install 3.12
uv python pin 3.12
uv sync
```

プラットフォーム用ヘルパーも利用できます。

```bash
./install.sh
./run.sh
```

Windows ポータブル環境のセットアップ:

```bat
install.bat
run.bat
```

Linux 固有のセットアップ手順は [docs/linux.md](docs/linux.md) にあります。上流版 Qwen3-ASR 単体のセットアップ手順は [docs/qwen-linux.md](docs/qwen-linux.md) にあります。

### モデルのダウンロード

モデルダウンロードの計画を表示します。

```bash
uv run python download_models.py --list
```

不足しているモデルアセットをダウンロードします。

```bash
uv run python download_models.py
```

必要に応じて Qwen3-ASR の取得元を明示します。

```bash
uv run python download_models.py --qwen-source modelscope
uv run python download_models.py --qwen-source huggingface
```

既定のモデルパス:

| コンポーネント | 既定パス |
| --- | --- |
| GAME | `experiments/GAME-1.0.3-medium-onnx` |
| HubertFA | `experiments/1218_hfa_model_new_dict` |
| Linux/macOS/Web の Qwen3-ASR | `experiments/Qwen3-ASR-1.7B` |
| Windows デスクトップの Qwen3-ASR | `experiments/Qwen3-ASR-1.7B-dml` |
| 日本語 mora ASR | `experiments/romajiASR` |
| RMVPE | `experiments/RMVPE/rmvpe.onnx` |

モデルパスは、デスクトップ GUI の設定パネルまたは Web UI の設定ページで変更できます。

### デスクトップ GUI

デスクトップ GUI を起動します。

```bash
uv run python app_fluent.py
```

デスクトップ GUI は主な対話型ワークフローです。モデルパス、実行デバイス、スライス設定、言語と歌詞モード、任意の参照歌詞を指定し、MIDI/USTX/デバッグ成果物をエクスポートできます。

### Web UI

ローカル Web バックエンドを起動します。

```bash
uv run python web_server.py
```

次を開きます。

```text
http://localhost:5000
```

必要な場合はカスタムポートを指定します。

```bash
V2M_WEB_PORT=5001 uv run python web_server.py
```

Web API の契約は [docs/web-api.md](docs/web-api.md) に記載されています。

### バッチ CLI

フォルダ単位の slice + ASR 処理を実行します。

```bash
uv run python scripts/slice_asr_cli.py <input_dir> <output_dir> \
  --asr-model experiments/Qwen3-ASR-1.7B \
  --device cpu \
  --language zh
```

Windows デスクトップ環境で該当モデルディレクトリがある場合は、Windows 用 Qwen パスと DirectML デバイスを使用します。

```bash
uv run python scripts/slice_asr_cli.py <input_dir> <output_dir> \
  --asr-model experiments/Qwen3-ASR-1.7B-dml \
  --device dml \
  --language zh
```

主なオプション:

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

### 出力

選択したワークフローに応じて、Vocal2Midi は次をエクスポートできます。

- `.mid`
- `.ustx`
- `.txt`
- `.csv`
- `TextGrid`
- chunk `.wav` files
- `.lab`
- ASR matching logs

## 開発者向け

### アーキテクチャ

想定している依存方向は次のとおりです。

```text
gui -> application -> inference
web -> application -> inference
```

主要領域:

- `application/`: アプリケーション層の設定、検証、ジョブの入口
- `gui/`: PyQt5 + qfluentwidgets デスクトップ UI
- `web_server.py`: Flask + SocketIO ローカル Web バックエンド
- `inference/`: ASR、アラインメント、ピッチ抽出、スライス、量子化、エクスポート
- `scripts/`: モデル/ソース管理とバッチ CLI ヘルパー
- `tests/`: 自動テスト

アーキテクチャの詳細は [docs/architecture.md](docs/architecture.md) にあります。開発ワークフローとドキュメント規約は [docs/contributing.md](docs/contributing.md) にあります。

### Rust Workspace

Rust 移行作業は [rewrite-in-rust/](rewrite-in-rust/) にあります。Cargo workspace は
意図的に [rewrite-in-rust/rust/](rewrite-in-rust/rust/) の下に置かれており、
Rust library units をデスクトップ GUI、Web バックエンド、完全なモデルパイプラインを
起動せずにテストできます。

主な Rust チェック:

```bash
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-targets --all-features -- -D warnings
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features --no-deps
```

Rust workspace README には、MSRV、crate 境界、JSON bridge 契約、migration-owner ルールが記載されています:
[rewrite-in-rust/rust/README.md](rewrite-in-rust/rust/README.md)。

### テスト

Web/API の focused test suite を実行します。

```bash
uv run pytest tests/test_web_api.py
```

すべての自動テストを実行します。

```bash
uv run pytest
```

手動統合テストです。先に `web_server.py` を起動してから実行します。

```bash
uv run python tests/test_api_integration.py
```

### ソースミラー

vendored third-party source mirrors は `third_party/` 以下にあります。次のコマンドで更新と監査を行います。

```bash
uv run python scripts/vendor_sources.py --force
uv run python scripts/vendor_native_sources.py --force
uv run python scripts/audit_vendored_sources.py
```

## ドキュメント

- Linux セットアップ: [docs/linux.md](docs/linux.md)
- Qwen3-ASR Linux メモ: [docs/qwen-linux.md](docs/qwen-linux.md)
- アーキテクチャ: [docs/architecture.md](docs/architecture.md)
- 開発ガイド: [docs/contributing.md](docs/contributing.md)
- コントリビューション入口: [CONTRIBUTING.md](CONTRIBUTING.md)
- ドキュメント方針: [docs/documentation.md](docs/documentation.md)
- Web API 契約: [docs/web-api.md](docs/web-api.md)
- セキュリティポリシー: [SECURITY.md](SECURITY.md)
- サードパーティクレジット: [ACKNOWLEDGEMENTS.md](ACKNOWLEDGEMENTS.md)

## ライセンス

Vocal2Midi は Apache License 2.0 の下で配布されています。[LICENSE](LICENSE) を参照してください。

サードパーティコンポーネント、vendored code、モデルアセット、辞書、組み込み素材には、それぞれ元のライセンス、通知、または帰属要件が適用される場合があります。それらの通知は対応する素材に引き続き適用されます。
