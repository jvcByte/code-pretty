use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::theme::Theme;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeSnippet {
    pub id: String,
    pub content: String,
    pub language: String,
    pub created_at: DateTime<Utc>,
    pub theme: Theme,
    pub metadata: SnippetMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnippetMetadata {
    pub source: InputSource,
    pub original_filename: Option<String>,
    pub line_count: usize,
    pub character_count: usize,
    pub detected_language_confidence: Option<f32>,
    pub ocr_confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InputSource {
    Upload,
    Paste,
    Type,
}

impl CodeSnippet {
    /// Creates a new code snippet with generated ID and current timestamp
    pub fn new(
        content: String,
        language: String,
        theme: Theme,
        source: InputSource,
        original_filename: Option<String>,
    ) -> Self {
        let line_count = content.lines().count();
        let character_count = content.chars().count();
        
        CodeSnippet {
            id: Uuid::new_v4().to_string(),
            content: content.clone(),
            language,
            created_at: Utc::now(),
            theme,
            metadata: SnippetMetadata {
                source,
                original_filename,
                line_count,
                character_count,
                detected_language_confidence: None,
                ocr_confidence: None,
            },
        }
    }

    /// Updates the content and recalculates metadata
    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        self.metadata.line_count = self.content.lines().count();
        self.metadata.character_count = self.content.chars().count();
    }

    /// Sets the detected language confidence score
    pub fn set_language_confidence(&mut self, confidence: f32) {
        self.metadata.detected_language_confidence = Some(confidence);
    }

    /// Sets the OCR confidence score
    pub fn set_ocr_confidence(&mut self, confidence: f32) {
        self.metadata.ocr_confidence = Some(confidence);
    }

    /// Validates the code snippet
    pub fn validate(&self) -> Result<(), String> {
        if self.content.is_empty() {
            return Err("Code content cannot be empty".to_string());
        }

        if self.language.is_empty() {
            return Err("Language must be specified".to_string());
        }

        if self.id.is_empty() {
            return Err("ID cannot be empty".to_string());
        }

        // Validate theme
        self.theme.validate()?;

        // Validate confidence scores if present
        if let Some(lang_conf) = self.metadata.detected_language_confidence {
            if lang_conf < 0.0 || lang_conf > 1.0 {
                return Err("Language confidence must be between 0.0 and 1.0".to_string());
            }
        }

        if let Some(ocr_conf) = self.metadata.ocr_confidence {
            if ocr_conf < 0.0 || ocr_conf > 1.0 {
                return Err("OCR confidence must be between 0.0 and 1.0".to_string());
            }
        }

        Ok(())
    }

    /// Returns true if the snippet likely came from OCR processing
    pub fn is_from_ocr(&self) -> bool {
        matches!(self.metadata.source, InputSource::Upload) && self.metadata.ocr_confidence.is_some()
    }

    /// Returns true if the language detection has low confidence
    pub fn has_low_language_confidence(&self) -> bool {
        self.metadata.detected_language_confidence
            .map(|conf| conf < 0.7)
            .unwrap_or(false)
    }

    /// Returns a summary of the snippet for display purposes
    pub fn summary(&self) -> SnippetSummary {
        let preview = if self.content.len() > 100 {
            format!("{}...", &self.content[..97])
        } else {
            self.content.clone()
        };

        SnippetSummary {
            id: self.id.clone(),
            language: self.language.clone(),
            line_count: self.metadata.line_count,
            character_count: self.metadata.character_count,
            source: self.metadata.source.clone(),
            preview,
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetSummary {
    pub id: String,
    pub language: String,
    pub line_count: usize,
    pub character_count: usize,
    pub source: InputSource,
    pub preview: String,
    pub created_at: DateTime<Utc>,
}

impl InputSource {
    /// Returns a human-readable description of the input source
    pub fn description(&self) -> &'static str {
        match self {
            InputSource::Upload => "Uploaded from image",
            InputSource::Paste => "Pasted from clipboard",
            InputSource::Type => "Typed directly",
        }
    }
}