// OCR service for text extraction from images
use crate::models::errors::AppError;
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

/// Result of OCR text extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCRResult {
    /// Extracted text from the image
    pub text: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Detected language (e.g., "eng", "spa", "fra")
    pub detected_language: Option<String>,
    /// Whether the result needs manual review
    pub needs_review: bool,
}

/// Configuration for OCR processing
#[derive(Debug, Clone)]
pub struct OCRConfig {
    /// Minimum confidence threshold (0.0 to 1.0)
    pub min_confidence: f32,
    /// Maximum processing time in seconds
    pub timeout_seconds: u64,
    /// Languages to use for OCR (e.g., ["eng", "spa"])
    pub languages: Vec<String>,
    /// Whether to preprocess images for better OCR
    pub preprocess: bool,
}

impl Default for OCRConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            timeout_seconds: 30,
            languages: vec!["eng".to_string()],
            preprocess: true,
        }
    }
}

/// OCR Service for extracting text from images
pub struct OCRService {
    config: OCRConfig,
    #[cfg(feature = "tesseract")]
    tesseract: Option<tesseract::Tesseract>,
}

impl OCRService {
    /// Create a new OCR service with default configuration
    pub fn new() -> Result<Self, AppError> {
        Self::with_config(OCRConfig::default())
    }

    /// Create a new OCR service with custom configuration
    pub fn with_config(config: OCRConfig) -> Result<Self, AppError> {
        #[cfg(feature = "tesseract")]
        {
            let tesseract = Self::initialize_tesseract(&config)?;
            Ok(Self {
                config,
                tesseract: Some(tesseract),
            })
        }

        #[cfg(not(feature = "tesseract"))]
        {
            tracing::warn!("Tesseract OCR is not available. OCR functionality will be limited.");
            Ok(Self { config })
        }
    }

    #[cfg(feature = "tesseract")]
    fn initialize_tesseract(config: &OCRConfig) -> Result<tesseract::Tesseract, AppError> {
        let mut tess = tesseract::Tesseract::new(None, Some(&config.languages.join("+")))
            .map_err(|e| AppError::ocr_failed(format!("Failed to initialize Tesseract: {}", e)))?;

        // Set OCR engine mode to LSTM (best accuracy)
        tess.set_variable("tessedit_ocr_engine_mode", "1")
            .map_err(|e| AppError::ocr_failed(format!("Failed to configure Tesseract: {}", e)))?;

        // Optimize for code/text blocks
        tess.set_variable("tessedit_pageseg_mode", "6") // Assume uniform block of text
            .map_err(|e| AppError::ocr_failed(format!("Failed to configure page segmentation: {}", e)))?;

        Ok(tess)
    }

    /// Extract text from image bytes
    pub async fn extract_text(&self, image_data: &[u8]) -> Result<OCRResult, AppError> {
        // Validate image data
        if image_data.is_empty() {
            return Err(AppError::ocr_failed("Image data is empty"));
        }

        // Load and validate image
        let image = self.load_image(image_data)?;

        // Preprocess image if enabled
        let processed_image = if self.config.preprocess {
            self.preprocess_image(image)?
        } else {
            image
        };

        // Perform OCR with timeout
        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        let ocr_future = self.perform_ocr(processed_image);

        match timeout(timeout_duration, ocr_future).await {
            Ok(result) => result,
            Err(_) => Err(AppError::timeout(self.config.timeout_seconds)),
        }
    }

    /// Load image from bytes
    fn load_image(&self, image_data: &[u8]) -> Result<DynamicImage, AppError> {
        image::load_from_memory(image_data).map_err(|e| {
            AppError::ocr_failed(format!("Failed to load image: {}. Ensure the file is a valid PNG, JPG, or JPEG image.", e))
        })
    }

    /// Preprocess image for better OCR results
    fn preprocess_image(&self, image: DynamicImage) -> Result<DynamicImage, AppError> {
        // Convert to grayscale for better OCR
        let gray = image.grayscale();

        // Increase contrast
        let adjusted = gray.adjust_contrast(20.0);

        // Resize if image is too small (OCR works better with larger images)
        let (width, height) = adjusted.dimensions();
        if width < 800 || height < 600 {
            let scale = (800.0 / width as f32).max(600.0 / height as f32);
            let new_width = (width as f32 * scale) as u32;
            let new_height = (height as f32 * scale) as u32;
            Ok(adjusted.resize(new_width, new_height, image::imageops::FilterType::Lanczos3))
        } else {
            Ok(adjusted)
        }
    }

    /// Perform OCR on the processed image
    async fn perform_ocr(&self, image: DynamicImage) -> Result<OCRResult, AppError> {
        #[cfg(feature = "tesseract")]
        {
            self.perform_tesseract_ocr(image).await
        }

        #[cfg(not(feature = "tesseract"))]
        {
            self.perform_fallback_ocr(image).await
        }
    }

    #[cfg(feature = "tesseract")]
    async fn perform_tesseract_ocr(&self, image: DynamicImage) -> Result<OCRResult, AppError> {
        let tesseract = self.tesseract.as_ref()
            .ok_or_else(|| AppError::ocr_failed("Tesseract not initialized"))?;

        // Convert image to RGB8 format for Tesseract
        let rgb_image = image.to_rgb8();
        let (width, height) = rgb_image.dimensions();
        let raw_data = rgb_image.into_raw();

        // Perform OCR in a blocking task (Tesseract is CPU-intensive)
        let languages = self.config.languages.clone();
        let min_confidence = self.config.min_confidence;

        tokio::task::spawn_blocking(move || {
            let mut tess = tesseract::Tesseract::new(None, Some(&languages.join("+")))
                .map_err(|e| AppError::ocr_failed(format!("Failed to initialize Tesseract: {}", e)))?;

            // Set image data
            tess.set_image(&raw_data, width as i32, height as i32, 3, (width * 3) as i32)
                .map_err(|e| AppError::ocr_failed(format!("Failed to set image: {}", e)))?;

            let text = tess.get_text()
                .map_err(|e| AppError::ocr_failed(format!("Failed to extract text: {}", e)))?;

            // Get confidence score
            let confidence = tess.mean_text_conf() as f32 / 100.0;

            // Detect language
            let detected_language = tess.get_source_language()
                .ok()
                .map(|lang| lang.to_string());

            let needs_review = confidence < min_confidence;

            Ok(OCRResult {
                text,
                confidence,
                detected_language,
                needs_review,
            })
        })
        .await
        .map_err(|e| AppError::ocr_failed(format!("OCR task failed: {}", e)))?
    }

    #[cfg(not(feature = "tesseract"))]
    async fn perform_fallback_ocr(&self, _image: DynamicImage) -> Result<OCRResult, AppError> {
        // Fallback when Tesseract is not available
        tracing::warn!("Tesseract OCR is not available. Returning placeholder result.");
        
        Err(AppError::ocr_failed(
            "OCR functionality is not available. Please install Tesseract OCR or enable the tesseract feature."
        ))
    }

    /// Detect supported languages
    pub fn get_supported_languages(&self) -> Vec<String> {
        #[cfg(feature = "tesseract")]
        {
            // Common languages supported by Tesseract
            vec![
                "eng".to_string(), // English
                "spa".to_string(), // Spanish
                "fra".to_string(), // French
                "deu".to_string(), // German
                "ita".to_string(), // Italian
                "por".to_string(), // Portuguese
                "rus".to_string(), // Russian
                "jpn".to_string(), // Japanese
                "chi_sim".to_string(), // Chinese Simplified
                "chi_tra".to_string(), // Chinese Traditional
                "kor".to_string(), // Korean
                "ara".to_string(), // Arabic
                "hin".to_string(), // Hindi
            ]
        }

        #[cfg(not(feature = "tesseract"))]
        {
            vec![]
        }
    }

    /// Update OCR configuration
    pub fn update_config(&mut self, config: OCRConfig) -> Result<(), AppError> {
        #[cfg(feature = "tesseract")]
        {
            self.tesseract = Some(Self::initialize_tesseract(&config)?);
        }
        self.config = config;
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &OCRConfig {
        &self.config
    }

    /// Process and clean OCR result text
    pub fn process_ocr_result(&self, result: &mut OCRResult) {
        result.text = Self::clean_extracted_text(&result.text);
        result.text = Self::preserve_code_formatting(&result.text);
    }

    /// Clean extracted text from OCR artifacts
    fn clean_extracted_text(text: &str) -> String {
        let mut cleaned = text.to_string();

        // Remove common OCR artifacts
        cleaned = cleaned.replace('\u{FEFF}', ""); // Remove BOM
        cleaned = cleaned.replace('\u{200B}', ""); // Remove zero-width space
        cleaned = cleaned.replace('\u{00A0}', " "); // Replace non-breaking space with regular space

        // Fix common OCR mistakes in code
        cleaned = Self::fix_common_ocr_mistakes(&cleaned);

        // Normalize line endings
        cleaned = cleaned.replace("\r\n", "\n");
        cleaned = cleaned.replace("\r", "\n");

        // Remove excessive blank lines (more than 2 consecutive)
        let lines: Vec<&str> = cleaned.lines().collect();
        let mut result_lines = Vec::new();
        let mut blank_count = 0;

        for line in lines {
            if line.trim().is_empty() {
                blank_count += 1;
                if blank_count <= 2 {
                    result_lines.push(line);
                }
            } else {
                blank_count = 0;
                result_lines.push(line);
            }
        }

        result_lines.join("\n")
    }

    /// Fix common OCR mistakes in code
    fn fix_common_ocr_mistakes(text: &str) -> String {
        let mut fixed = text.to_string();

        // Common character substitutions in code
        let replacements = vec![
            ("l", "1"), // lowercase L to 1 in certain contexts
            ("O", "0"), // uppercase O to 0 in certain contexts
            ("S", "$"), // S to $ in variable names
            ("|", "l"), // pipe to lowercase L
            ("rn", "m"), // rn to m
            ("vv", "w"), // double v to w
        ];

        // Apply context-aware replacements
        // This is a simplified version - in production, you'd want more sophisticated logic
        for (from, to) in replacements {
            // Only replace in specific contexts to avoid false positives
            if from == "O" && to == "0" {
                // Replace O with 0 when surrounded by digits
                let re = regex::Regex::new(r"(\d)O(\d)").unwrap_or_else(|_| regex::Regex::new(r"").unwrap());
                fixed = re.replace_all(&fixed, format!("$1{}$2", to)).to_string();
            }
        }

        fixed
    }

    /// Preserve code formatting and indentation
    fn preserve_code_formatting(text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut formatted_lines = Vec::new();

        for line in lines {
            // Preserve leading whitespace (indentation)
            let leading_spaces = line.len() - line.trim_start().len();
            let trimmed = line.trim_end();

            // Reconstruct line with preserved indentation
            if leading_spaces > 0 {
                formatted_lines.push(format!("{}{}", " ".repeat(leading_spaces), trimmed.trim_start()));
            } else {
                formatted_lines.push(trimmed.to_string());
            }
        }

        formatted_lines.join("\n")
    }

    /// Validate OCR result and determine if manual review is needed
    pub fn validate_result(&self, result: &OCRResult) -> ValidationResult {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // Check confidence threshold
        if result.confidence < self.config.min_confidence {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: format!(
                    "Low confidence score: {:.1}%. Manual review recommended.",
                    result.confidence * 100.0
                ),
                field: "confidence".to_string(),
            });
            suggestions.push("Review and correct any misrecognized characters".to_string());
        }

        // Check if text is empty
        if result.text.trim().is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: "No text was extracted from the image".to_string(),
                field: "text".to_string(),
            });
            suggestions.push("Ensure the image contains clear, readable text".to_string());
            suggestions.push("Try uploading a higher quality image".to_string());
        }

        // Check for suspicious patterns that might indicate OCR errors
        if Self::has_suspicious_patterns(&result.text) {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "Detected patterns that may indicate OCR errors".to_string(),
                field: "text".to_string(),
            });
            suggestions.push("Check for misrecognized special characters".to_string());
        }

        // Check text length
        if result.text.len() < 10 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "Extracted text is very short".to_string(),
                field: "text".to_string(),
            });
        }

        ValidationResult {
            is_valid: !issues.iter().any(|i| i.severity == IssueSeverity::Error),
            needs_review: result.needs_review || !issues.is_empty(),
            issues,
            suggestions,
        }
    }

    /// Check for suspicious patterns in extracted text
    fn has_suspicious_patterns(text: &str) -> bool {
        // Check for excessive special characters that might indicate OCR errors
        // Exclude common code characters like parentheses, quotes, brackets, etc.
        let code_chars = ['(', ')', '{', '}', '[', ']', '"', '\'', ';', ':', ',', '.', '!', '?'];
        let special_char_count = text.chars()
            .filter(|c| !c.is_alphanumeric() && !c.is_whitespace() && !code_chars.contains(c))
            .count();
        let total_chars = text.chars().count();

        if total_chars > 0 {
            let special_ratio = special_char_count as f32 / total_chars as f32;
            // If more than 30% unusual special characters, might be OCR errors
            if special_ratio > 0.3 {
                return true;
            }
        }

        // Check for repeated unusual character sequences
        let unusual_patterns = vec!["|||", "~~~"];
        for pattern in unusual_patterns {
            if text.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Extract text with automatic processing and validation
    pub async fn extract_and_process(&self, image_data: &[u8]) -> Result<ProcessedOCRResult, AppError> {
        // Extract text
        let mut result = self.extract_text(image_data).await?;

        // Process and clean the result
        self.process_ocr_result(&mut result);

        // Validate the result
        let validation = self.validate_result(&result);

        Ok(ProcessedOCRResult {
            ocr_result: result,
            validation,
        })
    }
}

/// Processed OCR result with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedOCRResult {
    pub ocr_result: OCRResult,
    pub validation: ValidationResult,
}

/// Validation result for OCR output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub needs_review: bool,
    pub issues: Vec<ValidationIssue>,
    pub suggestions: Vec<String>,
}

/// Individual validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub field: String,
}

/// Severity level for validation issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

impl Default for OCRService {
    fn default() -> Self {
        Self::new().expect("Failed to create default OCR service")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ocr_service_creation() {
        let service = OCRService::new();
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_ocr_config_default() {
        let config = OCRConfig::default();
        assert_eq!(config.min_confidence, 0.6);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.languages, vec!["eng".to_string()]);
        assert!(config.preprocess);
    }

    #[tokio::test]
    async fn test_empty_image_data() {
        let service = OCRService::new().unwrap();
        let result = service.extract_text(&[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_image_data() {
        let service = OCRService::new().unwrap();
        let invalid_data = vec![0u8; 100];
        let result = service.extract_text(&invalid_data).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_clean_extracted_text() {
        let text = "Hello\r\nWorld\r\n\n\n\n\nTest";
        let cleaned = OCRService::clean_extracted_text(text);
        assert!(!cleaned.contains("\r\n"));
        assert!(cleaned.contains('\n'));
        // Should reduce excessive blank lines (more than 2 consecutive)
        let lines: Vec<&str> = cleaned.lines().collect();
        let mut consecutive_empty = 0;
        let mut max_consecutive = 0;
        for line in lines {
            if line.trim().is_empty() {
                consecutive_empty += 1;
                max_consecutive = max_consecutive.max(consecutive_empty);
            } else {
                consecutive_empty = 0;
            }
        }
        assert!(max_consecutive <= 2, "Should not have more than 2 consecutive empty lines");
    }

    #[test]
    fn test_preserve_code_formatting() {
        let text = "    fn main() {\n        println!(\"Hello\");\n    }";
        let formatted = OCRService::preserve_code_formatting(text);
        assert!(formatted.contains("    fn main()"));
        assert!(formatted.contains("        println!"));
    }

    #[test]
    fn test_has_suspicious_patterns() {
        assert!(OCRService::has_suspicious_patterns("|||||||"));
        assert!(OCRService::has_suspicious_patterns("~~~~~~~"));
        // This should not be suspicious - it's valid code
        assert!(!OCRService::has_suspicious_patterns("fn main() { println!(\"test\"); }"));
        // Test with high ratio of unusual special characters
        assert!(OCRService::has_suspicious_patterns("!@#$%^&*_+|<>"));
        // Dots should not be suspicious (common in code)
        assert!(!OCRService::has_suspicious_patterns("..........."));
    }

    #[tokio::test]
    async fn test_validate_empty_result() {
        let service = OCRService::new().unwrap();
        let result = OCRResult {
            text: "".to_string(),
            confidence: 0.9,
            detected_language: Some("eng".to_string()),
            needs_review: false,
        };
        let validation = service.validate_result(&result);
        assert!(!validation.is_valid);
        assert!(validation.needs_review);
    }

    #[tokio::test]
    async fn test_validate_low_confidence() {
        let service = OCRService::new().unwrap();
        let result = OCRResult {
            text: "Some code here".to_string(),
            confidence: 0.3,
            detected_language: Some("eng".to_string()),
            needs_review: true,
        };
        let validation = service.validate_result(&result);
        assert!(validation.needs_review);
        assert!(!validation.issues.is_empty());
    }

    #[tokio::test]
    async fn test_validate_good_result() {
        let service = OCRService::new().unwrap();
        let result = OCRResult {
            text: "fn main() { println!(\"Hello, world!\"); }".to_string(),
            confidence: 0.95,
            detected_language: Some("eng".to_string()),
            needs_review: false,
        };
        let validation = service.validate_result(&result);
        assert!(validation.is_valid);
    }

    #[test]
    fn test_process_ocr_result() {
        let service = OCRService::new().unwrap();
        let mut result = OCRResult {
            text: "Hello\r\nWorld\u{FEFF}\n\n\n\n\nTest".to_string(),
            confidence: 0.9,
            detected_language: Some("eng".to_string()),
            needs_review: false,
        };
        service.process_ocr_result(&mut result);
        assert!(!result.text.contains('\r'));
        assert!(!result.text.contains('\u{FEFF}'));
    }
}