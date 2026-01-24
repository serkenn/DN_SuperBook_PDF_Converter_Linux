# 縦書き検出モジュール仕様書

## 概要

日本語書籍の縦書き（top-to-bottom）と横書き（left-to-right）を自動検出するモジュール。
二値化画像の行構造を分析し、縦書き確率を0.0〜1.0で返す。

## TC ID一覧

| TC ID | テスト項目 | 優先度 |
|-------|-----------|--------|
| VD-001 | 縦書き確率計算 | 高 |
| VD-002 | 横書き確率計算 | 高 |
| VD-003 | 混合レイアウト判定 | 中 |
| VD-004 | 空白画像ハンドリング | 高 |
| VD-005 | ブロック分割処理 | 高 |
| VD-006 | 交差数カウント | 高 |
| VD-007 | Welford法による統計計算 | 中 |
| VD-008 | 行厚/行間比率計算 | 中 |
| VD-009 | スコア合成 | 高 |
| VD-010 | 画像回転処理 | 高 |
| VD-011 | 4960×7016標準解像度 | 中 |
| VD-012 | 書籍全体の縦書き判定 | 高 |

## アルゴリズム

### 1. 縦書き確率計算 (IsPaperVerticalWriting_GetProbability)

```
入力: 二値画像 (CV_8UC1相当)
出力: 縦書き確率 (0.0-1.0)

手順:
1. 横方向スキャン → horizontalScore
2. 画像を90度時計回りに回転
3. 回転画像で同じスキャン → verticalScore
4. verticalProbability = verticalScore / (horizontalScore + verticalScore + 1e-9)
5. [0.0, 1.0]にクランプ
```

### 2. 線形スコア計算 (ComputeLinearScore)

画像を横一列ずつ走査し、行構造の「らしさ」を評価。

```
入力: 二値画像
出力: スコア (0.0-1.0)

手順:
1. 画像を4ブロックに横分割（段組み対策）
2. 各ブロックで:
   a. 各行の黒画素塊数（交差数）をカウント
   b. Welford法で平均・分散を計算
   c. 行厚と行間を抽出
   d. 3指標を合成:
      - 変動係数 × 0.4
      - ゼロ行比率 × 0.2
      - 行間比率 × 0.4
3. 4ブロックの平均を返す
```

### 3. 交差数カウント

1行あたりの黒画素塊（連続した黒ピクセル群）の数をカウント。
横書きテキストは多くの交差を持ち、空白行は交差0。

### 4. 行厚/行間抽出

- 交差数が閾値以上の連続行 → 行厚
- 交差数が閾値未満の連続行 → 行間
- 中央値を使用して外れ値の影響を抑制

### 5. スコア重み付け

| 指標 | 重み | 説明 |
|------|------|------|
| 変動係数 | 0.4 | 行ごとの交差数のばらつき |
| ゼロ行比率 | 0.2 | 完全な空白行の割合 |
| 行間比率 | 0.4 | 行間の広さ |

## 公開API

```rust
/// 縦書き検出オプション
pub struct VerticalDetectOptions {
    /// 黒とみなす閾値 (0-255, デフォルト: 128)
    pub black_threshold: u8,
    /// ブロック分割数 (デフォルト: 4)
    pub block_count: u32,
}

/// 縦書き検出結果
pub struct VerticalDetectResult {
    /// 縦書き確率 (0.0-1.0)
    pub vertical_probability: f64,
    /// 横書きスコア
    pub horizontal_score: f64,
    /// 縦書きスコア
    pub vertical_score: f64,
    /// 判定結果
    pub is_vertical: bool,
}

/// 単一画像の縦書き確率を計算
pub fn detect_vertical_probability(
    image: &image::GrayImage,
    options: &VerticalDetectOptions,
) -> Result<VerticalDetectResult, VerticalDetectError>;

/// 複数ページの縦書き判定（書籍全体）
pub fn detect_book_vertical_writing(
    images: &[image::GrayImage],
    options: &VerticalDetectOptions,
) -> Result<BookVerticalResult, VerticalDetectError>;
```

## 依存関係

- `image` クレート (GrayImage, ImageBuffer)
- `imageproc` クレート (回転処理)

## 参照

- C#実装: `SuperPdfUtil.cs` 行4940-5118
- 関数: `IsPaperVerticalWriting_GetProbability`, `ComputeLinearScore`
