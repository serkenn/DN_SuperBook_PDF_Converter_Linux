//! Page Number module core types
//!
//! Contains basic data structures for page number detection and offset analysis.

use std::path::{Path, PathBuf};
use thiserror::Error;

// ============================================================
// Constants
// ============================================================

/// Default search region percentage (percentage of image height)
pub const DEFAULT_SEARCH_REGION_PERCENT: f32 = 10.0;

/// Larger search region for vertical text (Japanese books)
pub const VERTICAL_SEARCH_REGION_PERCENT: f32 = 12.0;

/// Default minimum OCR confidence threshold
pub const DEFAULT_MIN_CONFIDENCE: f32 = 60.0;

/// Strict confidence threshold for high precision
pub const STRICT_MIN_CONFIDENCE: f32 = 80.0;

/// Minimum search region clamp value
pub const MIN_SEARCH_REGION: f32 = 5.0;

/// Maximum search region clamp value
pub const MAX_SEARCH_REGION: f32 = 50.0;

/// Minimum confidence clamp value
pub const MIN_CONFIDENCE_CLAMP: f32 = 0.0;

/// Maximum confidence clamp value
pub const MAX_CONFIDENCE_CLAMP: f32 = 100.0;

// ============================================================
// Error Types
// ============================================================

/// Page number detection error types
#[derive(Debug, Error)]
pub enum PageNumberError {
    #[error("Image not found: {0}")]
    ImageNotFound(PathBuf),

    #[error("OCR failed: {0}")]
    OcrFailed(String),

    #[error("No page numbers detected")]
    NoPageNumbersDetected,

    #[error("Inconsistent page numbers")]
    InconsistentPageNumbers,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PageNumberError>;

// ============================================================
// Core Data Structures
// ============================================================

/// Page number position types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageNumberPosition {
    /// Bottom center
    BottomCenter,
    /// Bottom outside (odd: right, even: left)
    BottomOutside,
    /// Bottom inside
    BottomInside,
    /// Top center
    TopCenter,
    /// Top outside
    TopOutside,
}

/// Page number rectangle
#[derive(Debug, Clone, Copy)]
pub struct PageNumberRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Detected page number
#[derive(Debug, Clone)]
pub struct DetectedPageNumber {
    /// Page index (0-indexed)
    pub page_index: usize,
    /// Detected number
    pub number: Option<i32>,
    /// Detection position
    pub position: PageNumberRect,
    /// OCR confidence
    pub confidence: f32,
    /// Raw OCR text
    pub raw_text: String,
}

/// Page number analysis result
#[derive(Debug, Clone)]
pub struct PageNumberAnalysis {
    /// Detection results for each page
    pub detections: Vec<DetectedPageNumber>,
    /// Detected position pattern
    pub position_pattern: PageNumberPosition,
    /// Odd page X offset (pixels)
    pub odd_page_offset_x: i32,
    /// Even page X offset
    pub even_page_offset_x: i32,
    /// Overall detection confidence
    pub overall_confidence: f32,
    /// Missing page numbers
    pub missing_pages: Vec<usize>,
    /// Duplicate page numbers
    pub duplicate_pages: Vec<i32>,
}

/// Offset correction result
#[derive(Debug, Clone)]
pub struct OffsetCorrection {
    /// Per-page horizontal offset
    pub page_offsets: Vec<(usize, i32)>,
    /// Recommended unified offset
    pub unified_offset: i32,
}

// ============================================================
// Options
// ============================================================

/// Page number detection options
#[derive(Debug, Clone)]
pub struct PageNumberOptions {
    /// Search region (percentage of image height to search)
    pub search_region_percent: f32,
    /// OCR language
    pub ocr_language: String,
    /// Minimum confidence threshold
    pub min_confidence: f32,
    /// Detect numbers only
    pub numbers_only: bool,
    /// Position hint
    pub position_hint: Option<PageNumberPosition>,
}

impl Default for PageNumberOptions {
    fn default() -> Self {
        Self {
            search_region_percent: DEFAULT_SEARCH_REGION_PERCENT,
            ocr_language: "jpn+eng".to_string(),
            min_confidence: DEFAULT_MIN_CONFIDENCE,
            numbers_only: true,
            position_hint: None,
        }
    }
}

impl PageNumberOptions {
    /// Create a new options builder
    pub fn builder() -> PageNumberOptionsBuilder {
        PageNumberOptionsBuilder::default()
    }

    /// Create options for Japanese documents
    pub fn japanese() -> Self {
        Self {
            ocr_language: "jpn".to_string(),
            search_region_percent: VERTICAL_SEARCH_REGION_PERCENT,
            ..Default::default()
        }
    }

    /// Create options for English documents
    pub fn english() -> Self {
        Self {
            ocr_language: "eng".to_string(),
            ..Default::default()
        }
    }

    /// Create options with high confidence threshold
    pub fn strict() -> Self {
        Self {
            min_confidence: STRICT_MIN_CONFIDENCE,
            ..Default::default()
        }
    }
}

/// Builder for PageNumberOptions
#[derive(Debug, Default)]
pub struct PageNumberOptionsBuilder {
    options: PageNumberOptions,
}

impl PageNumberOptionsBuilder {
    /// Set search region (percentage of image height, clamped to 5-50)
    #[must_use]
    pub fn search_region_percent(mut self, percent: f32) -> Self {
        self.options.search_region_percent = percent.clamp(MIN_SEARCH_REGION, MAX_SEARCH_REGION);
        self
    }

    /// Set OCR language
    #[must_use]
    pub fn ocr_language(mut self, lang: impl Into<String>) -> Self {
        self.options.ocr_language = lang.into();
        self
    }

    /// Set minimum confidence threshold (clamped to 0-100)
    #[must_use]
    pub fn min_confidence(mut self, confidence: f32) -> Self {
        self.options.min_confidence = confidence.clamp(MIN_CONFIDENCE_CLAMP, MAX_CONFIDENCE_CLAMP);
        self
    }

    /// Set whether to detect numbers only
    #[must_use]
    pub fn numbers_only(mut self, only: bool) -> Self {
        self.options.numbers_only = only;
        self
    }

    /// Set position hint
    #[must_use]
    pub fn position_hint(mut self, position: PageNumberPosition) -> Self {
        self.options.position_hint = Some(position);
        self
    }

    /// Build the options
    #[must_use]
    pub fn build(self) -> PageNumberOptions {
        self.options
    }
}

// ============================================================
// Detector Trait
// ============================================================

/// Page number detector trait
pub trait PageNumberDetector {
    /// Detect page number from single image
    fn detect_single(
        image_path: &Path,
        page_index: usize,
        options: &PageNumberOptions,
    ) -> Result<DetectedPageNumber>;

    /// Analyze multiple images
    fn analyze_batch(images: &[PathBuf], options: &PageNumberOptions)
        -> Result<PageNumberAnalysis>;

    /// Calculate offset correction
    fn calculate_offset(
        analysis: &PageNumberAnalysis,
        image_width: u32,
    ) -> Result<OffsetCorrection>;

    /// Validate page order
    fn validate_order(analysis: &PageNumberAnalysis) -> Result<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_number_options_default() {
        let opts = PageNumberOptions::default();
        assert_eq!(opts.search_region_percent, 10.0);
        assert_eq!(opts.min_confidence, 60.0);
        assert!(opts.numbers_only);
    }

    #[test]
    fn test_page_number_options_japanese() {
        let opts = PageNumberOptions::japanese();
        assert_eq!(opts.ocr_language, "jpn");
        assert_eq!(opts.search_region_percent, 12.0);
    }

    #[test]
    fn test_page_number_options_english() {
        let opts = PageNumberOptions::english();
        assert_eq!(opts.ocr_language, "eng");
    }

    #[test]
    fn test_page_number_options_strict() {
        let opts = PageNumberOptions::strict();
        assert_eq!(opts.min_confidence, 80.0);
    }

    #[test]
    fn test_page_number_options_builder() {
        let opts = PageNumberOptions::builder()
            .search_region_percent(15.0)
            .ocr_language("fra")
            .min_confidence(75.0)
            .numbers_only(false)
            .position_hint(PageNumberPosition::BottomCenter)
            .build();

        assert_eq!(opts.search_region_percent, 15.0);
        assert_eq!(opts.ocr_language, "fra");
        assert_eq!(opts.min_confidence, 75.0);
        assert!(!opts.numbers_only);
        assert!(matches!(
            opts.position_hint,
            Some(PageNumberPosition::BottomCenter)
        ));
    }

    #[test]
    fn test_builder_clamping() {
        // Search region clamped to 5-50
        let opts = PageNumberOptions::builder()
            .search_region_percent(100.0)
            .build();
        assert_eq!(opts.search_region_percent, 50.0);

        let opts = PageNumberOptions::builder()
            .search_region_percent(1.0)
            .build();
        assert_eq!(opts.search_region_percent, 5.0);

        // Confidence clamped to 0-100
        let opts = PageNumberOptions::builder().min_confidence(150.0).build();
        assert_eq!(opts.min_confidence, 100.0);

        let opts = PageNumberOptions::builder().min_confidence(-10.0).build();
        assert_eq!(opts.min_confidence, 0.0);
    }

    #[test]
    fn test_page_number_position_variants() {
        let positions = [
            PageNumberPosition::BottomCenter,
            PageNumberPosition::BottomOutside,
            PageNumberPosition::BottomInside,
            PageNumberPosition::TopCenter,
            PageNumberPosition::TopOutside,
        ];

        for pos in positions {
            let _clone = pos;
            assert!(matches!(
                pos,
                PageNumberPosition::BottomCenter
                    | PageNumberPosition::BottomOutside
                    | PageNumberPosition::BottomInside
                    | PageNumberPosition::TopCenter
                    | PageNumberPosition::TopOutside
            ));
        }
    }

    #[test]
    fn test_page_number_rect() {
        let rect = PageNumberRect {
            x: 100,
            y: 200,
            width: 50,
            height: 30,
        };
        assert_eq!(rect.x, 100);
        assert_eq!(rect.y, 200);
        assert_eq!(rect.width, 50);
        assert_eq!(rect.height, 30);
    }

    #[test]
    fn test_detected_page_number() {
        let detected = DetectedPageNumber {
            page_index: 5,
            number: Some(42),
            position: PageNumberRect {
                x: 100,
                y: 900,
                width: 50,
                height: 30,
            },
            confidence: 95.5,
            raw_text: "42".to_string(),
        };

        assert_eq!(detected.page_index, 5);
        assert_eq!(detected.number, Some(42));
        assert_eq!(detected.confidence, 95.5);
        assert_eq!(detected.raw_text, "42");
    }

    #[test]
    fn test_error_types() {
        let _err1 = PageNumberError::ImageNotFound(PathBuf::from("/test/path"));
        let _err2 = PageNumberError::OcrFailed("OCR error".to_string());
        let _err3 = PageNumberError::NoPageNumbersDetected;
        let _err4 = PageNumberError::InconsistentPageNumbers;
        let _err5: PageNumberError = std::io::Error::other("test").into();
    }
}
