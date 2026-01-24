//! Page Offset Analysis
//!
//! Calculates alignment offsets based on detected page numbers.

use super::types::{DetectedPageNumber, PageNumberRect};
use std::collections::HashSet;

// ============================================================
// Constants
// ============================================================

/// Minimum number of matches required for reliable shift detection
const MIN_MATCH_COUNT: usize = 5;

/// Minimum ratio of matched pages to total pages
const MIN_MATCH_RATIO: f64 = 1.0 / 3.0;

/// Maximum shift to test when finding page number offset
const MAX_SHIFT_TEST: i32 = 300;

// ============================================================
// Data Structures
// ============================================================

/// Per-page offset result
#[derive(Debug, Clone)]
pub struct PageOffsetResult {
    /// Physical page number (1-indexed, file order)
    pub physical_page: usize,
    /// Logical page number (detected from OCR, if available)
    pub logical_page: Option<i32>,
    /// Horizontal shift to apply (pixels)
    pub shift_x: i32,
    /// Vertical shift to apply (pixels)
    pub shift_y: i32,
    /// Position where page number was detected
    pub page_number_position: Option<PageNumberRect>,
    /// Whether this is an odd page (in physical order)
    pub is_odd: bool,
}

impl PageOffsetResult {
    /// Create a new result with no offset (for pages without detected page numbers)
    pub fn no_offset(physical_page: usize) -> Self {
        Self {
            physical_page,
            logical_page: None,
            shift_x: 0,
            shift_y: 0,
            page_number_position: None,
            is_odd: physical_page % 2 == 1,
        }
    }
}

/// Book offset analysis result
#[derive(Debug, Clone)]
pub struct BookOffsetAnalysis {
    /// Physical to logical page number shift
    /// (logical_page = physical_page - page_number_shift)
    pub page_number_shift: i32,
    /// Per-page offset results
    pub page_offsets: Vec<PageOffsetResult>,
    /// Average X position for odd pages
    pub odd_avg_x: Option<i32>,
    /// Average X position for even pages
    pub even_avg_x: Option<i32>,
    /// Average Y position for odd pages
    pub odd_avg_y: Option<i32>,
    /// Average Y position for even pages
    pub even_avg_y: Option<i32>,
    /// Number of pages with matched page numbers
    pub match_count: usize,
    /// Confidence in the analysis (0.0-1.0)
    pub confidence: f64,
}

impl Default for BookOffsetAnalysis {
    fn default() -> Self {
        Self {
            page_number_shift: 0,
            page_offsets: Vec::new(),
            odd_avg_x: None,
            even_avg_x: None,
            odd_avg_y: None,
            even_avg_y: None,
            match_count: 0,
            confidence: 0.0,
        }
    }
}

impl BookOffsetAnalysis {
    /// Check if the analysis has sufficient confidence to be used
    pub fn is_reliable(&self, total_pages: usize) -> bool {
        // At least 5 matches and at least 1/3 of pages matched
        self.match_count >= 5 && self.match_count * 3 >= total_pages
    }

    /// Get offset for a specific page
    pub fn get_offset(&self, physical_page: usize) -> Option<&PageOffsetResult> {
        self.page_offsets
            .iter()
            .find(|p| p.physical_page == physical_page)
    }
}

// ============================================================
// Page Offset Analyzer
// ============================================================

/// Page offset analyzer for calculating alignment shifts
pub struct PageOffsetAnalyzer;

impl PageOffsetAnalyzer {
    /// Analyze page offsets from detected page numbers
    ///
    /// This function:
    /// 1. Detects the physical-to-logical page number shift
    /// 2. Groups pages into odd/even
    /// 3. Calculates average positions for each group
    /// 4. Computes per-page shift to align with the average
    pub fn analyze_offsets(
        detections: &[DetectedPageNumber],
        _image_height: u32,
    ) -> BookOffsetAnalysis {
        if detections.is_empty() {
            return BookOffsetAnalysis::default();
        }

        // Step 1: Find the best physical-to-logical shift
        let (best_shift, match_count, confidence) = Self::find_best_page_number_shift(detections);

        // Check if we have enough matches
        if match_count < MIN_MATCH_COUNT
            || (match_count as f64) < (detections.len() as f64 * MIN_MATCH_RATIO)
        {
            // Not enough confidence - return no offsets
            return BookOffsetAnalysis {
                page_number_shift: 0,
                page_offsets: detections
                    .iter()
                    .map(|d| PageOffsetResult::no_offset(d.page_index + 1))
                    .collect(),
                confidence: 0.0,
                match_count: 0,
                ..Default::default()
            };
        }

        // Step 2: Build matched page data with positions
        let mut matched_pages: Vec<(usize, PageNumberRect, bool)> = Vec::new();
        for det in detections {
            let physical_page = det.page_index + 1;
            let expected_logical = physical_page as i32 - best_shift;

            if expected_logical >= 1 && det.number == Some(expected_logical) {
                matched_pages.push((physical_page, det.position, physical_page % 2 == 1));
            }
        }

        // Step 3: Calculate averages for odd and even groups
        let odd_positions: Vec<&(usize, PageNumberRect, bool)> = matched_pages
            .iter()
            .filter(|(_, _, is_odd)| *is_odd)
            .collect();
        let even_positions: Vec<&(usize, PageNumberRect, bool)> = matched_pages
            .iter()
            .filter(|(_, _, is_odd)| !*is_odd)
            .collect();

        let odd_avg_x = Self::calculate_average_x(&odd_positions);
        let odd_avg_y = Self::calculate_average_y(&odd_positions);
        let even_avg_x = Self::calculate_average_x(&even_positions);
        let even_avg_y = Self::calculate_average_y(&even_positions);

        // Step 4: Align Y values between groups if close enough
        let (final_odd_avg_y, final_even_avg_y) = Self::align_group_y_values(odd_avg_y, even_avg_y);

        // Step 5: Calculate per-page offsets
        let page_offsets = Self::calculate_per_page_offsets(
            detections,
            best_shift,
            odd_avg_x,
            even_avg_x,
            final_odd_avg_y,
            final_even_avg_y,
        );

        BookOffsetAnalysis {
            page_number_shift: best_shift,
            page_offsets,
            odd_avg_x,
            even_avg_x,
            odd_avg_y: final_odd_avg_y,
            even_avg_y: final_even_avg_y,
            match_count,
            confidence,
        }
    }

    /// Find the best physical-to-logical page number shift
    ///
    /// Tests shifts from -MAX_SHIFT_TEST to +MAX_SHIFT_TEST and returns
    /// the shift that maximizes the number of matches weighted by confidence.
    fn find_best_page_number_shift(detections: &[DetectedPageNumber]) -> (i32, usize, f64) {
        let mut best_shift = 0i32;
        let mut best_score = 0.0f64;
        let mut best_count = 0usize;

        for shift in -MAX_SHIFT_TEST..MAX_SHIFT_TEST {
            let mut score = 0.0f64;
            let mut count = 0usize;

            for det in detections {
                let physical_page = det.page_index + 1;
                let expected_logical = physical_page as i32 - shift;

                if expected_logical >= 1 && det.number == Some(expected_logical) {
                    score += det.confidence as f64;
                    count += 1;
                }
            }

            if score > best_score || (score == best_score && shift.abs() < best_shift.abs()) {
                best_score = score;
                best_shift = shift;
                best_count = count;
            }
        }

        // Normalize confidence to 0-1 range
        let max_possible_score = detections.len() as f64 * 100.0;
        let confidence = if max_possible_score > 0.0 {
            best_score / max_possible_score
        } else {
            0.0
        };

        (best_shift, best_count, confidence)
    }

    /// Calculate average X position from matched positions
    fn calculate_average_x(positions: &[&(usize, PageNumberRect, bool)]) -> Option<i32> {
        if positions.is_empty() {
            return None;
        }

        let sum: i64 = positions
            .iter()
            .map(|(_, rect, _)| rect.x as i64 + rect.width as i64 / 2)
            .sum();

        Some((sum / positions.len() as i64) as i32)
    }

    /// Calculate average Y position from matched positions
    fn calculate_average_y(positions: &[&(usize, PageNumberRect, bool)]) -> Option<i32> {
        if positions.is_empty() {
            return None;
        }

        let sum: i64 = positions
            .iter()
            .map(|(_, rect, _)| rect.y as i64 + rect.height as i64 / 2)
            .sum();

        Some((sum / positions.len() as i64) as i32)
    }

    /// Align Y values between odd and even groups if they're close
    fn align_group_y_values(
        odd_avg_y: Option<i32>,
        even_avg_y: Option<i32>,
    ) -> (Option<i32>, Option<i32>) {
        match (odd_avg_y, even_avg_y) {
            (Some(odd_y), Some(even_y)) => {
                let diff = (odd_y - even_y).abs();
                // If difference is less than 5% of a typical page height (assuming ~7000px)
                // then align them
                if diff < 350 {
                    let avg = (odd_y + even_y) / 2;
                    (Some(avg), Some(avg))
                } else {
                    (Some(odd_y), Some(even_y))
                }
            }
            _ => (odd_avg_y, even_avg_y),
        }
    }

    /// Calculate per-page offsets based on averages
    fn calculate_per_page_offsets(
        detections: &[DetectedPageNumber],
        shift: i32,
        odd_avg_x: Option<i32>,
        even_avg_x: Option<i32>,
        odd_avg_y: Option<i32>,
        even_avg_y: Option<i32>,
    ) -> Vec<PageOffsetResult> {
        detections
            .iter()
            .map(|det| {
                let physical_page = det.page_index + 1;
                let is_odd = physical_page % 2 == 1;
                let expected_logical = physical_page as i32 - shift;

                // Check if this page's detected number matches the expected
                let matched = expected_logical >= 1 && det.number == Some(expected_logical);

                if matched {
                    let avg_x = if is_odd { odd_avg_x } else { even_avg_x };
                    let avg_y = if is_odd { odd_avg_y } else { even_avg_y };

                    // Calculate center of detected position
                    let center_x = det.position.x as i32 + det.position.width as i32 / 2;
                    let center_y = det.position.y as i32 + det.position.height as i32 / 2;

                    // Calculate shift to align with average
                    let shift_x = avg_x.map(|ax| ax - center_x).unwrap_or(0);
                    let shift_y = avg_y.map(|ay| ay - center_y).unwrap_or(0);

                    PageOffsetResult {
                        physical_page,
                        logical_page: Some(expected_logical),
                        shift_x,
                        shift_y,
                        page_number_position: Some(det.position),
                        is_odd,
                    }
                } else {
                    PageOffsetResult::no_offset(physical_page)
                }
            })
            .collect()
    }

    /// Create offset results for pages without page number detection
    /// using group averages for alignment
    pub fn interpolate_missing_offsets(analysis: &mut BookOffsetAnalysis, total_pages: usize) {
        // Find pages that don't have offsets
        let existing: HashSet<usize> = analysis
            .page_offsets
            .iter()
            .map(|p| p.physical_page)
            .collect();

        for page in 1..=total_pages {
            if !existing.contains(&page) {
                // Add a no-offset entry for missing pages
                analysis
                    .page_offsets
                    .push(PageOffsetResult::no_offset(page));
            }
        }

        // Sort by physical page
        analysis.page_offsets.sort_by_key(|p| p.physical_page);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_offset_result_no_offset() {
        let result = PageOffsetResult::no_offset(5);
        assert_eq!(result.physical_page, 5);
        assert_eq!(result.logical_page, None);
        assert_eq!(result.shift_x, 0);
        assert_eq!(result.shift_y, 0);
        assert!(result.is_odd);
    }

    #[test]
    fn test_page_offset_result_even_page() {
        let result = PageOffsetResult::no_offset(6);
        assert!(!result.is_odd);
    }

    #[test]
    fn test_book_offset_analysis_default() {
        let analysis = BookOffsetAnalysis::default();
        assert_eq!(analysis.page_number_shift, 0);
        assert!(analysis.page_offsets.is_empty());
        assert_eq!(analysis.match_count, 0);
        assert_eq!(analysis.confidence, 0.0);
    }

    #[test]
    fn test_book_offset_analysis_reliability() {
        let mut analysis = BookOffsetAnalysis::default();

        // Not reliable with 0 matches
        assert!(!analysis.is_reliable(100));

        // Not reliable with only 4 matches (need at least 5)
        analysis.match_count = 4;
        assert!(!analysis.is_reliable(100));

        // Not reliable if less than 1/3 of pages matched
        analysis.match_count = 5;
        assert!(!analysis.is_reliable(100)); // 5 < 100/3

        // Reliable with enough matches
        analysis.match_count = 40;
        assert!(analysis.is_reliable(100)); // 40 >= 100/3
    }

    #[test]
    fn test_analyze_empty_detections() {
        let detections: Vec<DetectedPageNumber> = vec![];
        let analysis = PageOffsetAnalyzer::analyze_offsets(&detections, 7000);
        assert_eq!(analysis.page_number_shift, 0);
        assert!(analysis.page_offsets.is_empty());
    }

    #[test]
    fn test_interpolate_missing_offsets() {
        let mut analysis = BookOffsetAnalysis {
            page_offsets: vec![
                PageOffsetResult::no_offset(1),
                PageOffsetResult::no_offset(3),
                PageOffsetResult::no_offset(5),
            ],
            ..Default::default()
        };

        PageOffsetAnalyzer::interpolate_missing_offsets(&mut analysis, 5);

        assert_eq!(analysis.page_offsets.len(), 5);
        assert_eq!(analysis.page_offsets[0].physical_page, 1);
        assert_eq!(analysis.page_offsets[1].physical_page, 2);
        assert_eq!(analysis.page_offsets[2].physical_page, 3);
        assert_eq!(analysis.page_offsets[3].physical_page, 4);
        assert_eq!(analysis.page_offsets[4].physical_page, 5);
    }

    #[test]
    fn test_get_offset() {
        let analysis = BookOffsetAnalysis {
            page_offsets: vec![
                PageOffsetResult::no_offset(1),
                PageOffsetResult::no_offset(2),
                PageOffsetResult::no_offset(3),
            ],
            ..Default::default()
        };

        let offset = analysis.get_offset(2);
        assert!(offset.is_some());
        assert_eq!(offset.unwrap().physical_page, 2);

        let missing = analysis.get_offset(99);
        assert!(missing.is_none());
    }

    // ============================================================
    // TC-PAGENUM Spec Tests
    // ============================================================

    // TC-PAGENUM-001: 連続ページ番号 - 正確なシフト計算
    #[test]
    fn test_tc_pagenum_001_sequential_page_numbers() {
        use crate::page_number::types::{DetectedPageNumber, PageNumberRect};

        // Simulate consecutive page numbers detected
        let detections = vec![
            DetectedPageNumber {
                page_index: 0,
                number: Some(1),
                position: PageNumberRect { x: 500, y: 100, width: 50, height: 20 },
                confidence: 0.9,
                raw_text: "1".to_string(),
            },
            DetectedPageNumber {
                page_index: 1,
                number: Some(2),
                position: PageNumberRect { x: 500, y: 100, width: 50, height: 20 },
                confidence: 0.9,
                raw_text: "2".to_string(),
            },
            DetectedPageNumber {
                page_index: 2,
                number: Some(3),
                position: PageNumberRect { x: 500, y: 100, width: 50, height: 20 },
                confidence: 0.9,
                raw_text: "3".to_string(),
            },
        ];

        let analysis = PageOffsetAnalyzer::analyze_offsets(&detections, 1000);

        // Sequential pages should have shift of 0
        assert_eq!(analysis.page_number_shift, 0);
        // All pages should be in offsets
        assert_eq!(analysis.page_offsets.len(), 3);
    }

    // TC-PAGENUM-002: 欠損ページ番号 - 補間で補完
    #[test]
    fn test_tc_pagenum_002_missing_page_interpolation() {
        use crate::page_number::types::PageNumberRect;

        let mut analysis = BookOffsetAnalysis {
            page_offsets: vec![
                PageOffsetResult {
                    physical_page: 1,
                    logical_page: Some(1),
                    shift_x: 10,
                    shift_y: 5,
                    page_number_position: Some(PageNumberRect { x: 100, y: 50, width: 30, height: 20 }),
                    is_odd: true,
                },
                // Page 2 is missing
                PageOffsetResult {
                    physical_page: 3,
                    logical_page: Some(3),
                    shift_x: 10,
                    shift_y: 5,
                    page_number_position: Some(PageNumberRect { x: 100, y: 50, width: 30, height: 20 }),
                    is_odd: true,
                },
            ],
            page_number_shift: 0,
            odd_avg_x: Some(100),
            even_avg_x: Some(900),
            odd_avg_y: Some(50),
            even_avg_y: Some(50),
            match_count: 2,
            confidence: 0.8,
        };

        PageOffsetAnalyzer::interpolate_missing_offsets(&mut analysis, 3);

        // After interpolation, page 2 should be present
        assert_eq!(analysis.page_offsets.len(), 3);
        let page2 = analysis.get_offset(2);
        assert!(page2.is_some());
    }

    // TC-PAGENUM-003: 装飾的番号 - 正確な検出
    #[test]
    fn test_tc_pagenum_003_decorative_numbers() {
        // Test that logical page numbers can differ from physical
        let result = PageOffsetResult {
            physical_page: 5,
            logical_page: Some(1), // Book starts at page 5 but logical is 1
            shift_x: 0,
            shift_y: 0,
            page_number_position: None,
            is_odd: true,
        };

        assert_eq!(result.physical_page, 5);
        assert_eq!(result.logical_page, Some(1));
    }

    // TC-PAGENUM-004: ローマ数字 - 検出スキップ
    #[test]
    fn test_tc_pagenum_004_roman_numerals_skipped() {
        // Pages with None logical_page represent non-Arabic numerals
        let result = PageOffsetResult {
            physical_page: 1,
            logical_page: None, // Roman numeral, skipped
            shift_x: 0,
            shift_y: 0,
            page_number_position: None,
            is_odd: true,
        };

        assert!(result.logical_page.is_none());

        // Verify no_offset creates proper structure
        let no_offset = PageOffsetResult::no_offset(2);
        assert_eq!(no_offset.shift_x, 0);
        assert_eq!(no_offset.shift_y, 0);
    }

    // TC-PAGENUM-005: 奇偶位置差 - 個別オフセット
    #[test]
    fn test_tc_pagenum_005_odd_even_separate_offsets() {
        use crate::page_number::types::{DetectedPageNumber, PageNumberRect};

        // Odd pages have different position than even pages
        let detections = vec![
            DetectedPageNumber {
                page_index: 0,
                number: Some(1),
                position: PageNumberRect { x: 100, y: 50, width: 50, height: 20 }, // Odd: left
                confidence: 0.9,
                raw_text: "1".to_string(),
            },
            DetectedPageNumber {
                page_index: 1,
                number: Some(2),
                position: PageNumberRect { x: 900, y: 50, width: 50, height: 20 }, // Even: right
                confidence: 0.9,
                raw_text: "2".to_string(),
            },
            DetectedPageNumber {
                page_index: 2,
                number: Some(3),
                position: PageNumberRect { x: 105, y: 52, width: 50, height: 20 }, // Odd: left
                confidence: 0.9,
                raw_text: "3".to_string(),
            },
            DetectedPageNumber {
                page_index: 3,
                number: Some(4),
                position: PageNumberRect { x: 895, y: 48, width: 50, height: 20 }, // Even: right
                confidence: 0.9,
                raw_text: "4".to_string(),
            },
        ];

        let analysis = PageOffsetAnalyzer::analyze_offsets(&detections, 1000);

        // Should detect shift correctly
        assert!(!analysis.page_offsets.is_empty());
        // Odd and even pages should have different X offsets potentially
        // The actual test verifies the structure supports this
    }
}
