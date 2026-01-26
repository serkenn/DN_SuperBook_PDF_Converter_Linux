# superbook-pdf

[![Crates.io](https://img.shields.io/crates/v/superbook-pdf.svg)](https://crates.io/crates/superbook-pdf)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

> **Fork of [dnobori/DN_SuperBook_PDF_Converter](https://github.com/dnobori/DN_SuperBook_PDF_Converter)**
>
> Rust で完全リライトしたスキャン書籍 PDF 高品質化ツール

**オリジナル著者:** 登 大遊 (Daiyuu Nobori)
**Rust リライト:** clearclown
**ライセンス:** AGPL v3.0

---

## 概要

superbook-pdf は、スキャンした書籍の PDF を高品質化するツールです。傾き補正、AI 超解像、裏写り除去、OCR など、プロフェッショナルレベルの PDF 最適化を自動で行います。

## Before / After

![Before and After comparison](doc_img/ba.png)

| | Before (左) | After (右) |
|---|---|---|
| **解像度** | 1242×2048 px | 2363×3508 px |
| **ファイルサイズ** | 981 KB | 1.6 MB |
| **品質** | ぼやけ、低コントラスト | 鮮明、高コントラスト |

> RealESRGAN による AI 超解像で、文字のエッジが鮮明になり、読みやすさが大幅に向上

---

## 特徴

| 機能 | 説明 |
|------|------|
| **Rust 実装** | C# 版を完全リライト。メモリ効率 10 倍改善 (10-30GB → 1-3GB) |
| **シングルバイナリ** | 依存関係最小化。コンテナ不要で直接実行可能 |
| **GPU 高速化** | NVIDIA CUDA 対応 RealESRGAN で AI 超解像を高速実行 |
| **Web UI** | ブラウザからドラッグ&ドロップで PDF 変換 |
| **WebSocket** | リアルタイム進捗表示・ページプレビュー |
| **日本語 OCR** | YomiToku による高精度日本語文字認識 |
| **TDD 開発** | 1,260+ テストで品質担保 |

---

## システム要件

### 必須

- **OS:** Linux (Ubuntu 20.04+, Debian 11+, Fedora 38+)
- **RAM:** 4GB 以上 (8GB 推奨)
- **Rust:** 1.75 以上
- **Poppler:** pdftoppm コマンド

### GPU 高速化 (推奨)

| 要件 | 説明 |
|------|------|
| **GPU** | NVIDIA GeForce GTX 1060 以上 |
| **VRAM** | 4GB 以上 (6GB+ 推奨) |
| **CUDA** | 11.8 以上 |
| **ドライバ** | 525.0 以上 |

> **Note:** GPU がなくても動作しますが、AI 超解像 (RealESRGAN) は CPU フォールバックとなり、大幅に遅くなります。

---

## インストール

### 方法 1: crates.io から (推奨)

```bash
cargo install superbook-pdf --features web
```

### 方法 2: ソースからビルド

```bash
git clone https://github.com/clearclown/DN_SuperBook_PDF_Converter_Linux.git
cd DN_SuperBook_PDF_Converter_Linux/superbook-pdf
cargo build --release --features web
```

### 依存ツールのインストール

#### Ubuntu / Debian

```bash
# 必須: PDF 画像抽出
sudo apt install poppler-utils

# 推奨: Python 仮想環境
sudo apt install python3-venv python3-pip
```

#### Fedora / RHEL

```bash
sudo dnf install poppler-utils python3-pip
```

#### Arch Linux

```bash
sudo pacman -S poppler python-pip
```

### AI 機能のセットアップ (オプション)

```bash
cd superbook-pdf/ai_bridge

# 仮想環境作成
python3 -m venv .venv
source .venv/bin/activate

# RealESRGAN (AI 超解像)
pip install realesrgan

# YomiToku (日本語 OCR)
pip install yomitoku

# 環境変数設定 (実行時に必要)
export SUPERBOOK_VENV=$(pwd)/.venv
```

---

## 使い方

### CLI - 基本

```bash
# シンプルな変換
superbook-pdf convert input.pdf -o output/

# 高品質変換 (AI 超解像 + OCR)
superbook-pdf convert input.pdf -o output/ --advanced --ocr

# GPU 使用を明示
superbook-pdf convert input.pdf -o output/ --gpu true --advanced
```

### CLI - オプション一覧

```bash
superbook-pdf convert [OPTIONS] <INPUT>

引数:
  <INPUT>              入力 PDF ファイル

オプション:
  -o, --output <DIR>   出力ディレクトリ [デフォルト: ./output]
  -d, --dpi <DPI>      抽出 DPI [デフォルト: 300]
  --max-pages <N>      処理ページ数制限 (デバッグ用)
  --gpu <BOOL>         GPU 使用 [デフォルト: true]
  --advanced           高度な処理を有効化 (カラー補正、オフセット整列)
  --ocr                OCR を有効化 (YomiToku)
  --save-debug         中間ファイルを保存
  -v, --verbose        詳細出力 (-vvv で最大)
  -h, --help           ヘルプ表示
```

### Web UI

```bash
# サーバー起動
superbook-pdf serve --port 8080

# ブラウザで開く
open http://localhost:8080
```

Web UI の機能:
- ドラッグ&ドロップで PDF アップロード
- リアルタイム進捗バー
- ページプレビュー (WebSocket)
- 複数ジョブの並列処理
- 結果のダウンロード

---

## 処理パイプライン

superbook-pdf は以下の 10 ステップで PDF を高品質化します:

```
┌─────────────────────────────────────────────────────────────────────┐
│  1. PDF 画像抽出    │ pdftoppm で 300 DPI 抽出                      │
├─────────────────────────────────────────────────────────────────────┤
│  2. マージントリム  │ 0.5% の余白を除去                             │
├─────────────────────────────────────────────────────────────────────┤
│  3. AI 超解像       │ RealESRGAN で 2x アップスケール (GPU)         │
├─────────────────────────────────────────────────────────────────────┤
│  4. 解像度正規化    │ 4960×7016 に統一 (内部処理用)                 │
├─────────────────────────────────────────────────────────────────────┤
│  5. 傾き補正        │ 大津二値化 + Hough 変換で自動検出・補正       │
├─────────────────────────────────────────────────────────────────────┤
│  6. カラー補正      │ 裏写り除去、紙色の白化                        │
├─────────────────────────────────────────────────────────────────────┤
│  7. グループクロップ│ Tukey fence で外れ値除去、統一クロップ        │
├─────────────────────────────────────────────────────────────────────┤
│  8. オフセット整列  │ ページ番号位置に基づく整列                    │
├─────────────────────────────────────────────────────────────────────┤
│  9. 最終リサイズ    │ 3508 px 高さに統一                            │
├─────────────────────────────────────────────────────────────────────┤
│ 10. PDF 生成 + OCR  │ メタデータ同期、YomiToku OCR (オプション)     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## GPU 設定ガイド

### NVIDIA GPU の確認

```bash
# GPU 情報確認
nvidia-smi

# CUDA バージョン確認
nvcc --version
```

### PyTorch GPU 版のインストール

```bash
source ai_bridge/.venv/bin/activate

# CUDA 11.8 の場合
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu118

# CUDA 12.1 の場合
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121
```

### GPU 動作確認

```bash
python3 -c "import torch; print(f'CUDA available: {torch.cuda.is_available()}')"
```

### トラブルシューティング

| 問題 | 解決策 |
|------|--------|
| `CUDA out of memory` | `--max-pages 10` で分割処理、または VRAM 6GB+ の GPU を使用 |
| `RealESRGAN not found` | `SUPERBOOK_VENV` 環境変数を設定 |
| GPU が使用されない | `nvidia-smi` で GPU 使用率を確認、PyTorch GPU 版をインストール |

---

## 設定ファイル

`superbook.toml` で設定をカスタマイズ:

```toml
[pipeline]
dpi = 300                    # 抽出解像度
deskew = true                # 傾き補正
upscale = true               # AI 超解像
gpu = true                   # GPU 使用

[advanced]
internal_resolution = true   # 内部解像度正規化
color_correction = true      # カラー補正
offset_alignment = true      # オフセット整列

[output]
final_height = 3508          # 出力高さ (px)
jpeg_quality = 95            # JPEG 品質 (PDF 埋め込み時)

[ocr]
enabled = false              # OCR 有効化
language = "ja"              # 言語
```

---

## API リファレンス

### REST API

| エンドポイント | メソッド | 説明 |
|---------------|---------|------|
| `/api/jobs` | POST | ジョブ作成 (multipart/form-data) |
| `/api/jobs/{id}` | GET | ジョブ状態取得 |
| `/api/jobs/{id}/result` | GET | 結果 PDF ダウンロード |
| `/api/jobs/{id}` | DELETE | ジョブキャンセル |
| `/api/stats` | GET | サーバー統計 |
| `/ws` | WebSocket | リアルタイム通知 |

### WebSocket メッセージ形式

```json
// 進捗通知
{"type": "progress", "job_id": "...", "current": 5, "total": 10, "step": "Deskewing"}

// ページプレビュー
{"type": "page_preview", "job_id": "...", "page": 1, "preview": "base64...", "stage": "deskewed"}

// 完了通知
{"type": "completed", "job_id": "...", "elapsed": 45.2, "pages": 100}

// エラー通知
{"type": "error", "job_id": "...", "message": "..."}
```

### ジョブ作成例

```bash
curl -X POST http://localhost:8080/api/jobs \
  -F "file=@input.pdf" \
  -F "options={\"dpi\":300,\"advanced\":true,\"ocr\":true}"
```

---

## ベンチマーク

### 環境

- CPU: AMD Ryzen 9 5900X
- RAM: 32GB
- GPU: NVIDIA RTX 3080 (10GB VRAM)
- SSD: NVMe

### 結果

| ページ数 | C# 版 (メモリ) | Rust 版 (メモリ) | 速度比 |
|---------|---------------|-----------------|-------|
| 100     | 45s (12GB)    | 42s (1.8GB)     | 1.07x |
| 300     | 180s (28GB)   | 125s (2.4GB)    | 1.44x |
| 500     | OOM           | 210s (3.1GB)    | ∞     |

> メモリ効率が 10 倍以上改善され、大量ページの処理が可能に

---

## 開発

```bash
cd superbook-pdf

# テスト実行
cargo test

# Web 機能付きテスト
cargo test --features web

# ベンチマーク
cargo bench

# フォーマット & Lint
cargo fmt && cargo clippy

# ドキュメント生成
cargo doc --open
```

---

## レガシー C# 版

オリジナルの C# 実装は `legacy-csharp` ブランチで保存されています:

```bash
git checkout legacy-csharp
```

---

## ライセンス

AGPL v3.0 - [LICENSE](LICENSE)

このソフトウェアを使用したサービスを提供する場合、ソースコードの公開が必要です。

---

## 謝辞

- **登 大遊 (Daiyuu Nobori)** - オリジナル実装、PDF 処理アルゴリズム
- **[RealESRGAN](https://github.com/xinntao/Real-ESRGAN)** - AI 画像超解像
- **[YomiToku](https://github.com/kotaro-kinoshita/yomitoku)** - 日本語 AI OCR
- **[Poppler](https://poppler.freedesktop.org/)** - PDF レンダリング

---

## 関連リンク

- [オリジナルリポジトリ](https://github.com/dnobori/DN_SuperBook_PDF_Converter)
- [Rust ドキュメント](https://docs.rs/superbook-pdf)
- [Issue Tracker](https://github.com/clearclown/DN_SuperBook_PDF_Converter_Linux/issues)
