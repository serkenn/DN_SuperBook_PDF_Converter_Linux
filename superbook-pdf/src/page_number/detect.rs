//! Page Number Detection Implementation
//!
//! Tesseract-based page number detection.

use super::types::{
    DetectedPageNumber, OffsetCorrection, PageNumberAnalysis, PageNumberError, PageNumberOptions,
    PageNumberPosition, PageNumberRect, Result,
};
use image::GenericImageView;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Tesseract-based page number detector
pub struct TesseractPageDetector;

impl TesseractPageDetector {
    /// Detect page number from single image
    pub fn detect_single(
        image_path: &Path,
        page_index: usize,
        options: &PageNumberOptions,
    ) -> Result<DetectedPageNumber> {
        if !image_path.exists() {
            return Err(PageNumberError::ImageNotFound(image_path.to_path_buf()));
        }

        let img = image::open(image_path)
            .map_err(|_| PageNumberError::ImageNotFound(image_path.to_path_buf()))?;

        let (width, height) = img.dimensions();

        // Determine search region based on position hint
        let (search_y, search_height) = match options.position_hint {
            Some(PageNumberPosition::TopCenter | PageNumberPosition::TopOutside) => {
                let h = (height as f32 * options.search_region_percent / 100.0) as u32;
                (0, h)
            }
            _ => {
                let h = (height as f32 * options.search_region_percent / 100.0) as u32;
                (height.saturating_sub(h), h)
            }
        };

        // Crop search region
        let search_region = img.crop_imm(0, search_y, width, search_height);

        // For now, use simple image analysis instead of Tesseract
        // In a full implementation, this would call tesseract OCR
        let (number, raw_text, confidence) =
            Self::analyze_region_for_numbers(&search_region, options);

        Ok(DetectedPageNumber {
            page_index,
            number: if confidence >= options.min_confidence {
                number
            } else {
                None
            },
            position: PageNumberRect {
                x: 0,
                y: search_y,
                width,
                height: search_height,
            },
            confidence: confidence / 100.0,
            raw_text,
        })
    }

    /// Analyze image region for numbers (simplified implementation)
    fn analyze_region_for_numbers(
        _img: &image::DynamicImage,
        _options: &PageNumberOptions,
    ) -> (Option<i32>, String, f32) {
        // In a full implementation, this would:
        // 1. Save region to temp file
        // 2. Call tesseract with appropriate settings
        // 3. Parse the result

        // For now, return a placeholder
        (None, String::new(), 0.0)
    }

    /// Analyze multiple images
    pub fn analyze_batch(
        images: &[PathBuf],
        options: &PageNumberOptions,
    ) -> Result<PageNumberAnalysis> {
        let detections: Vec<DetectedPageNumber> = images
            .par_iter()
            .enumerate()
            .map(|(i, path)| Self::detect_single(path, i, options))
            .collect::<Result<Vec<_>>>()?;

        // Analyze pattern
        let (position_pattern, odd_offset, even_offset) = Self::analyze_pattern(&detections);

        // Find missing and duplicate pages
        let detected_numbers: Vec<i32> = detections.iter().filter_map(|d| d.number).collect();
        let missing_pages = Self::find_missing_pages(&detected_numbers);
        let duplicate_pages = Self::find_duplicate_pages(&detected_numbers);

        let overall_confidence = if detections.is_empty() {
            0.0
        } else {
            detections.iter().map(|d| d.confidence).sum::<f32>() / detections.len() as f32
        };

        Ok(PageNumberAnalysis {
            detections,
            position_pattern,
            odd_page_offset_x: odd_offset,
            even_page_offset_x: even_offset,
            overall_confidence,
            missing_pages,
            duplicate_pages,
        })
    }

    /// Analyze position pattern from detections
    fn analyze_pattern(detections: &[DetectedPageNumber]) -> (PageNumberPosition, i32, i32) {
        // Analyze X positions of detected page numbers
        let mut odd_positions: Vec<i32> = Vec::new();
        let mut even_positions: Vec<i32> = Vec::new();

        for detection in detections {
            if let Some(num) = detection.number {
                let center_x = detection.position.x as i32 + detection.position.width as i32 / 2;
                if num % 2 == 1 {
                    odd_positions.push(center_x);
                } else {
                    even_positions.push(center_x);
                }
            }
        }

        let odd_avg = if odd_positions.is_empty() {
            0
        } else {
            odd_positions.iter().sum::<i32>() / odd_positions.len() as i32
        };

        let even_avg = if even_positions.is_empty() {
            0
        } else {
            even_positions.iter().sum::<i32>() / even_positions.len() as i32
        };

        // Determine pattern based on position difference
        let position_pattern = if (odd_avg - even_avg).abs() < 50 {
            PageNumberPosition::BottomCenter
        } else if odd_avg > even_avg {
            PageNumberPosition::BottomOutside
        } else {
            PageNumberPosition::BottomInside
        };

        (position_pattern, odd_avg, even_avg)
    }

    /// Find missing page numbers
    fn find_missing_pages(numbers: &[i32]) -> Vec<usize> {
        if numbers.is_empty() {
            return vec![];
        }

        let min = *numbers.iter().min().unwrap();
        let max = *numbers.iter().max().unwrap();
        let set: HashSet<_> = numbers.iter().collect();

        (min..=max)
            .filter(|n| !set.contains(n))
            .map(|n| (n - min) as usize)
            .collect()
    }

    /// Find duplicate page numbers
    fn find_duplicate_pages(numbers: &[i32]) -> Vec<i32> {
        let mut seen = HashSet::new();
        numbers
            .iter()
            .filter(|n| !seen.insert(*n))
            .cloned()
            .collect()
    }

    /// Calculate offset correction
    pub fn calculate_offset(
        analysis: &PageNumberAnalysis,
        _image_width: u32,
    ) -> Result<OffsetCorrection> {
        let page_offsets: Vec<(usize, i32)> = analysis
            .detections
            .iter()
            .enumerate()
            .filter_map(|(i, d)| {
                d.number.map(|num| {
                    let offset = if num % 2 == 1 {
                        analysis.odd_page_offset_x
                    } else {
                        analysis.even_page_offset_x
                    };
                    (i, offset)
                })
            })
            .collect();

        let unified_offset = if !page_offsets.is_empty() {
            page_offsets.iter().map(|(_, o)| *o).sum::<i32>() / page_offsets.len() as i32
        } else {
            0
        };

        Ok(OffsetCorrection {
            page_offsets,
            unified_offset,
        })
    }

    /// Validate page order
    pub fn validate_order(analysis: &PageNumberAnalysis) -> Result<bool> {
        let numbers: Vec<i32> = analysis
            .detections
            .iter()
            .filter_map(|d| d.number)
            .collect();

        if numbers.len() < 2 {
            return Ok(true);
        }

        // Check if numbers are in ascending order
        for i in 1..numbers.len() {
            if numbers[i] <= numbers[i - 1] {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Parse Roman numeral to integer
    pub fn parse_roman_numeral(text: &str) -> Option<i32> {
        let text = text.to_lowercase().trim().to_string();
        let roman_map = [
            ("m", 1000),
            ("cm", 900),
            ("d", 500),
            ("cd", 400),
            ("c", 100),
            ("xc", 90),
            ("l", 50),
            ("xl", 40),
            ("x", 10),
            ("ix", 9),
            ("v", 5),
            ("iv", 4),
            ("i", 1),
        ];

        let mut result = 0;
        let mut remaining = text.as_str();

        for (numeral, value) in &roman_map {
            while remaining.starts_with(numeral) {
                result += value;
                remaining = &remaining[numeral.len()..];
            }
        }

        if remaining.is_empty() && result > 0 {
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_nonexistent_file() {
        let options = PageNumberOptions::default();
        let result =
            TesseractPageDetector::detect_single(Path::new("/nonexistent/image.png"), 0, &options);
        assert!(matches!(result, Err(PageNumberError::ImageNotFound(_))));
    }

    #[test]
    fn test_roman_numeral_parsing() {
        assert_eq!(TesseractPageDetector::parse_roman_numeral("I"), Some(1));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("IV"), Some(4));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("V"), Some(5));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("IX"), Some(9));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("X"), Some(10));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("XL"), Some(40));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("L"), Some(50));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("XC"), Some(90));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("C"), Some(100));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("CD"), Some(400));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("D"), Some(500));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("CM"), Some(900));
        assert_eq!(TesseractPageDetector::parse_roman_numeral("M"), Some(1000));
        assert_eq!(
            TesseractPageDetector::parse_roman_numeral("MCMXCIX"),
            Some(1999)
        );
        assert_eq!(
            TesseractPageDetector::parse_roman_numeral("MMXXIII"),
            Some(2023)
        );
    }

    #[test]
    fn test_roman_numeral_invalid() {
        assert_eq!(TesseractPageDetector::parse_roman_numeral(""), None);
        assert_eq!(TesseractPageDetector::parse_roman_numeral("ABC"), None);
        assert_eq!(TesseractPageDetector::parse_roman_numeral("123"), None);
    }

    #[test]
    fn test_find_missing_pages() {
        let numbers = vec![1, 2, 4, 5, 7];
        let missing = TesseractPageDetector::find_missing_pages(&numbers);
        assert!(missing.contains(&2)); // 3 is missing (index 2 from min)
        assert!(missing.contains(&5)); // 6 is missing (index 5 from min)
    }

    #[test]
    fn test_find_duplicate_pages() {
        let numbers = vec![1, 2, 2, 3, 4, 4, 4];
        let duplicates = TesseractPageDetector::find_duplicate_pages(&numbers);
        assert!(duplicates.contains(&2));
        assert!(duplicates.contains(&4));
    }

    #[test]
    fn test_analyze_empty_batch() {
        let images: Vec<PathBuf> = vec![];
        let options = PageNumberOptions::default();
        let result = TesseractPageDetector::analyze_batch(&images, &options).unwrap();
        assert!(result.detections.is_empty());
        assert_eq!(result.overall_confidence, 0.0);
    }

    #[test]
    fn test_validate_order_ascending() {
        let analysis = PageNumberAnalysis {
            detections: vec![
                DetectedPageNumber {
                    page_index: 0,
                    number: Some(1),
                    position: PageNumberRect {
                        x: 0,
                        y: 0,
                        width: 100,
                        height: 50,
                    },
                    confidence: 0.9,
                    raw_text: "1".to_string(),
                },
                DetectedPageNumber {
                    page_index: 1,
                    number: Some(2),
                    position: PageNumberRect {
                        x: 0,
                        y: 0,
                        width: 100,
                        height: 50,
                    },
                    confidence: 0.9,
                    raw_text: "2".to_string(),
                },
                DetectedPageNumber {
                    page_index: 2,
                    number: Some(3),
                    position: PageNumberRect {
                        x: 0,
                        y: 0,
                        width: 100,
                        height: 50,
                    },
                    confidence: 0.9,
                    raw_text: "3".to_string(),
                },
            ],
            position_pattern: PageNumberPosition::BottomCenter,
            odd_page_offset_x: 0,
            even_page_offset_x: 0,
            overall_confidence: 0.9,
            missing_pages: vec![],
            duplicate_pages: vec![],
        };

        assert!(TesseractPageDetector::validate_order(&analysis).unwrap());
    }

    #[test]
    fn test_validate_order_not_ascending() {
        let analysis = PageNumberAnalysis {
            detections: vec![
                DetectedPageNumber {
                    page_index: 0,
                    number: Some(3),
                    position: PageNumberRect {
                        x: 0,
                        y: 0,
                        width: 100,
                        height: 50,
                    },
                    confidence: 0.9,
                    raw_text: "3".to_string(),
                },
                DetectedPageNumber {
                    page_index: 1,
                    number: Some(1),
                    position: PageNumberRect {
                        x: 0,
                        y: 0,
                        width: 100,
                        height: 50,
                    },
                    confidence: 0.9,
                    raw_text: "1".to_string(),
                },
            ],
            position_pattern: PageNumberPosition::BottomCenter,
            odd_page_offset_x: 0,
            even_page_offset_x: 0,
            overall_confidence: 0.9,
            missing_pages: vec![],
            duplicate_pages: vec![],
        };

        assert!(!TesseractPageDetector::validate_order(&analysis).unwrap());
    }
}
