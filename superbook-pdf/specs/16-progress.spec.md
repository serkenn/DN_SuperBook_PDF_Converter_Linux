# 16-progress.spec.md - 進捗追跡モジュール仕様

## 概要

PDF処理の進捗状況を追跡・表示するモジュール。C#版の`ProgressTracker.cs`を参考に、Rustで再実装する。

## 目的

- 処理ステージごとの進捗を視覚的に表示
- ファイル単位・ページ単位の進捗を追跡
- サマリー出力でユーザーに結果を明示

## 設計

### ProcessingStage (列挙型)

| ステージ | 英語名 | 説明 |
|----------|--------|------|
| Initializing | 初期化中 | セットアップ処理 |
| Extracting | 抽出中 | PDF→画像抽出 |
| Deskewing | 傾き補正中 | Deskew処理 |
| Normalizing | 正規化中 | 内部解像度正規化 |
| ColorCorrecting | 色補正中 | グローバルカラー補正 |
| Cropping | クロップ中 | マージン検出・クロップ |
| Upscaling | AI高画質化中 | RealESRGAN処理 |
| Finalizing | 最終処理中 | 出力リサイズ |
| WritingPdf | PDF生成中 | PDF書き込み |
| OCR | 文字認識中 | YomiToku OCR |
| Completed | 完了 | 処理完了 |

### ProgressTracker (構造体)

```rust
pub struct ProgressTracker {
    /// 現在処理中のファイル番号 (1から開始)
    pub current_file: usize,
    /// 総ファイル数
    pub total_files: usize,
    /// 現在のファイル名
    pub current_filename: String,
    /// 現在の処理ステージ
    pub current_stage: ProcessingStage,
    /// 現在のページ番号 (1から開始)
    pub current_page: usize,
    /// 総ページ数
    pub total_pages: usize,
    /// 現在処理中のアイテム名
    pub current_item: String,
    /// 開始時刻
    pub start_time: Instant,
    /// 出力モード (quiet/normal/verbose)
    pub output_mode: OutputMode,
}
```

### OutputMode (列挙型)

| モード | 説明 |
|--------|------|
| Quiet | 出力なし |
| Normal | 通常出力 (ステージ表示) |
| Verbose | 詳細出力 (ページ単位進捗) |
| VeryVerbose | 超詳細 (全アイテム表示) |

## API

### 基本操作

| 関数 | 説明 |
|------|------|
| `new(total_files, output_mode)` | 新規作成 |
| `start_file(file_number, filename)` | ファイル処理開始 |
| `set_stage(stage, total_pages)` | ステージ変更 |
| `update_page(page_number, item_name)` | ページ進捗更新 |
| `complete_file()` | ファイル処理完了 |
| `print_summary(ok, skip, error)` | サマリー出力 |

### 進捗バー

```
[========================================] 100%
[====================--------------------]  50%
[------------------------------------]   0%
```

### 出力例

```
================================================================================
[ファイル 1/3] sample.pdf
================================================================================
ステージ: Extracting (抽出中)
ページ進捗: [====================--------------------]  50% (50/100)
現在の処理: page_0050.png
--------------------------------------------------------------------------------
```

## テストケース

| TC ID | テスト内容 |
|-------|----------|
| PROG-001 | ProgressTracker新規作成 |
| PROG-002 | start_file()でファイル開始 |
| PROG-003 | set_stage()でステージ変更 |
| PROG-004 | update_page()で進捗更新 |
| PROG-005 | complete_file()で完了マーク |
| PROG-006 | ProcessingStage名称取得 |
| PROG-007 | ProcessingStage日本語説明取得 |
| PROG-008 | プログレスバー構築 |
| PROG-009 | サマリー出力 |
| PROG-010 | OutputMode::Quiet動作確認 |
| PROG-011 | OutputMode::Verbose動作確認 |
| PROG-012 | 経過時間計算 |

## 実装ステータス

| 機能 | 状態 | 備考 |
|------|------|------|
| ProcessingStage列挙型 | ✅ | 11ステージ実装 |
| ProgressTracker構造体 | ✅ | スレッドセーフ |
| プログレスバー表示 | ✅ | 40文字幅 |
| サマリー出力 | ✅ | print_summary() |
| OutputMode対応 | ✅ | Quiet/Normal/Verbose/VeryVerbose |
| テスト | ✅ | 21テスト実装 |
