# superbook-pdf

スキャンされた書籍PDFを高品質なデジタル書籍に変換するRustツール。

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)

## 特徴

- **高品質画像抽出**: ImageMagick連携による300-600 DPI画像抽出
- **傾き補正**: 自動スキュー検出と補正
- **マージン最適化**: 統一マージン検出とトリミング
- **AI超解像**: RealESRGAN 2x/4xアップスケーリング
- **日本語OCR**: YomiToku AI-OCRによる検索可能PDF生成
- **ページ番号認識**: ローマ数字対応、オフセット自動補正
- **メモリ効率**: ストリーミング処理で1-3GB RAM使用

## インストール

### ビルド要件

- Rust 1.75以上
- ImageMagick 7.x
- Ghostscript
- Python 3.10以上 (AI機能用)

```bash
# ビルド
cargo build --release

# バイナリは target/release/superbook-pdf に生成
```

### Python AI環境セットアップ

```bash
cd ai_bridge
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate
pip install -r requirements.txt
```

## 使い方

### 基本的な変換

```bash
# 単一PDF変換
superbook-pdf convert input.pdf -o output/

# ディレクトリ一括変換
superbook-pdf convert input_dir/ -o output/

# プレビュー (ドライラン)
superbook-pdf convert input.pdf -o output/ --dry-run
```

### オプション

```bash
superbook-pdf convert input.pdf -o output/ \
    --dpi 300 \           # 出力DPI (default: 300)
    --deskew \            # 傾き補正有効
    --margin-trim 1.0 \   # マージントリム % (default: 0.5)
    --upscale \           # AI超解像有効
    --ocr \               # OCR有効 (YomiToku)
    --gpu \               # GPU使用
    -v                    # 詳細出力
```

### システム情報

```bash
superbook-pdf info
```

## アーキテクチャ

```
PDF入力 → 画像抽出 → 傾き補正 → マージン検出 → AI超解像 → OCR → PDF出力
              ↓
          (ImageMagick)   (imageproc)   (統計分析)  (RealESRGAN) (YomiToku)
```

### モジュール構成

| モジュール | 機能 |
|-----------|------|
| `cli` | CLIインターフェース (clap) |
| `pdf_reader` | PDF読み込み (lopdf) |
| `pdf_writer` | PDF書き込み (printpdf) |
| `image_extract` | 画像抽出 (ImageMagick) |
| `deskew` | 傾き補正 (imageproc) |
| `margin` | マージン検出 |
| `page_number` | ページ番号認識 |
| `ai_bridge` | Python AI連携 |
| `realesrgan` | AI超解像 |
| `yomitoku` | 日本語OCR |

## 開発

```bash
# テスト実行
cargo test

# 特定モジュールのテスト
cargo test pdf_reader::

# ベンチマーク
cargo bench

# コード品質チェック
cargo clippy -- -D warnings
cargo fmt -- --check
```

## ライセンス

AGPL-3.0

Original Author: 登 大遊 (Daiyuu Nobori)

## 関連リンク

- [オリジナルリポジトリ](https://github.com/dnobori/DN_SuperBook_PDF_Converter)
- [開発ロードマップ (Issue #19)](https://github.com/clearclown/DN_SuperBook_PDF_Converter_Linux/issues/19)
- [RealESRGAN](https://github.com/xinntao/Real-ESRGAN)
- [YomiToku](https://github.com/kotaro-kinoshita/yomitoku)
