use std::collections::HashMap;
use std::sync::Arc;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style, Theme as SyntectTheme, ThemeSet};
use syntect::parsing::{SyntaxSet, SyntaxReference};
use syntect::util::LinesWithEndings;
use crate::models::theme::{SyntaxColors, Theme};
use crate::models::errors::AppError;

/// Result of syntax highlighting operation
#[derive(Debug, Clone)]
pub struct HighlightResult {
    pub highlighted_lines: Vec<HighlightedLine>,
    pub language: String,
    pub total_lines: usize,
}

/// A single highlighted line with styled segments
#[derive(Debug, Clone)]
pub struct HighlightedLine {
    pub segments: Vec<HighlightedSegment>,
    pub line_number: usize,
}

/// A segment of text with styling information
#[derive(Debug, Clone)]
pub struct HighlightedSegment {
    pub text: String,
    pub style: SegmentStyle,
}

/// Style information for a text segment
#[derive(Debug, Clone)]
pub struct SegmentStyle {
    pub color: String,
    pub bold: bool,
    pub italic: bool,
}

/// Service for syntax highlighting with caching and theme integration
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    cache: Arc<std::sync::Mutex<HashMap<String, Arc<SyntaxReference>>>>,
    fallback_syntax: Arc<SyntaxReference>,
}

impl SyntaxHighlighter {
    /// Creates a new SyntaxHighlighter with default syntax and theme sets
    pub fn new() -> Result<Self, AppError> {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        // Find a fallback syntax (plain text) - clone to avoid borrow issues
        let fallback_syntax = syntax_set.find_syntax_plain_text().clone();

        Ok(SyntaxHighlighter {
            syntax_set,
            theme_set,
            cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
            fallback_syntax: Arc::new(fallback_syntax),
        })
    }

    /// Highlights code with the given language and theme
    pub fn highlight_code(
        &self,
        code: &str,
        language: &str,
        theme: &Theme,
    ) -> Result<HighlightResult, AppError> {
        // Get syntax reference for the language
        let syntax = self.get_syntax_for_language(language);
        
        // Create a custom syntect theme from our theme
        let syntect_theme = self.create_syntect_theme_from_custom(theme)?;
        
        // Perform highlighting
        let mut highlighter = HighlightLines::new(syntax.as_ref(), &syntect_theme);
        let mut highlighted_lines = Vec::new();
        
        for (line_number, line) in LinesWithEndings::from(code).enumerate() {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .map_err(|e| AppError::SyntaxHighlightingError { message: format!("Highlighting failed: {}", e) })?;
            
            let segments = ranges
                .into_iter()
                .map(|(style, text)| HighlightedSegment {
                    text: text.to_string(),
                    style: self.convert_syntect_style_to_segment_style(style),
                })
                .collect();
            
            highlighted_lines.push(HighlightedLine {
                segments,
                line_number: line_number + 1,
            });
        }

        let total_lines = highlighted_lines.len();

        Ok(HighlightResult {
            highlighted_lines,
            language: language.to_string(),
            total_lines,
        })
    }

    /// Gets syntax reference for a language with caching
    fn get_syntax_for_language(&self, language: &str) -> Arc<SyntaxReference> {
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(syntax) = cache.get(language) {
                return syntax.clone();
            }
        }

        // Try to find syntax by name or extension
        let syntax = self
            .syntax_set
            .find_syntax_by_name(language)
            .or_else(|| self.syntax_set.find_syntax_by_extension(language))
            .or_else(|| self.syntax_set.find_syntax_by_first_line(""))
            .unwrap_or(self.fallback_syntax.as_ref());

        let syntax_arc = Arc::new(syntax.clone());

        // Cache the result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(language.to_string(), syntax_arc.clone());
        }

        syntax_arc
    }

    /// Creates a syntect theme from our custom theme structure
    fn create_syntect_theme_from_custom(&self, _theme: &Theme) -> Result<SyntectTheme, AppError> {
        // For now, use a default theme - we can enhance this later
        // The complex theme customization can be implemented in the image generation phase
        let theme = self
            .theme_set
            .themes
            .get("base16-ocean.dark")
            .or_else(|| self.theme_set.themes.values().next())
            .ok_or_else(|| AppError::SyntaxHighlightingError { message: "No base theme available".to_string() })?;

        Ok(theme.clone())
    }

    /// Updates theme scopes with custom syntax colors (simplified version)
    fn update_theme_scopes(&self, _theme: &mut SyntectTheme, _syntax_colors: &SyntaxColors) -> Result<(), AppError> {
        // For now, we'll handle color customization in the image generation phase
        // This allows us to focus on getting the basic syntax highlighting working first
        Ok(())
    }

    /// Parses a hex color string to syntect Color
    fn parse_color(&self, color_str: &str) -> Result<Color, AppError> {
        if !color_str.starts_with('#') {
            return Err(AppError::SyntaxHighlightingError {
                message: format!("Invalid color format: {}", color_str)
            });
        }

        let hex = &color_str[1..];
        let (r, g, b, a) = match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|_| AppError::SyntaxHighlightingError { message: "Invalid hex color".to_string() })?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|_| AppError::SyntaxHighlightingError { message: "Invalid hex color".to_string() })?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|_| AppError::SyntaxHighlightingError { message: "Invalid hex color".to_string() })?;
                (r, g, b, 255)
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| AppError::SyntaxHighlightingError { message: "Invalid hex color".to_string() })?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| AppError::SyntaxHighlightingError { message: "Invalid hex color".to_string() })?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| AppError::SyntaxHighlightingError { message: "Invalid hex color".to_string() })?;
                (r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| AppError::SyntaxHighlightingError { message: "Invalid hex color".to_string() })?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| AppError::SyntaxHighlightingError { message: "Invalid hex color".to_string() })?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| AppError::SyntaxHighlightingError { message: "Invalid hex color".to_string() })?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|_| AppError::SyntaxHighlightingError { message: "Invalid hex color".to_string() })?;
                (r, g, b, a)
            }
            _ => return Err(AppError::SyntaxHighlightingError { message: "Invalid hex color length".to_string() }),
        };

        Ok(Color { r, g, b, a })
    }

    /// Converts syntect Style to our SegmentStyle
    fn convert_syntect_style_to_segment_style(&self, style: Style) -> SegmentStyle {
        let color = format!("#{:02x}{:02x}{:02x}", style.foreground.r, style.foreground.g, style.foreground.b);
        
        SegmentStyle {
            color,
            bold: style.font_style.contains(syntect::highlighting::FontStyle::BOLD),
            italic: style.font_style.contains(syntect::highlighting::FontStyle::ITALIC),
        }
    }

    /// Gets a list of supported languages
    pub fn get_supported_languages(&self) -> Vec<String> {
        self.syntax_set
            .syntaxes()
            .iter()
            .map(|syntax| syntax.name.clone())
            .collect()
    }

    /// Gets a list of supported file extensions
    pub fn get_supported_extensions(&self) -> Vec<String> {
        let mut extensions = Vec::new();
        for syntax in self.syntax_set.syntaxes() {
            extensions.extend(syntax.file_extensions.clone());
        }
        extensions.sort();
        extensions.dedup();
        extensions
    }

    /// Detects language from code content using syntect's built-in detection
    pub fn detect_language_from_content(&self, code: &str) -> Option<String> {
        // Try to detect by first line (for shebangs, etc.)
        if let Some(syntax) = self.syntax_set.find_syntax_by_first_line(code) {
            return Some(syntax.name.clone());
        }

        // Fallback to plain text
        None
    }

    /// Detects language from file extension
    pub fn detect_language_from_extension(&self, extension: &str) -> Option<String> {
        self.syntax_set
            .find_syntax_by_extension(extension)
            .map(|syntax| syntax.name.clone())
    }

    /// Clears the syntax cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Gets cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().unwrap();
        (cache.len(), cache.capacity())
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new().expect("Failed to create default SyntaxHighlighter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::theme::Theme;

    #[test]
    fn test_syntax_highlighter_creation() {
        let highlighter = SyntaxHighlighter::new();
        assert!(highlighter.is_ok());
    }

    #[test]
    fn test_supported_languages() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        let languages = highlighter.get_supported_languages();
        assert!(!languages.is_empty());
        assert!(languages.contains(&"Rust".to_string()));
        assert!(languages.contains(&"JavaScript".to_string()));
    }

    #[test]
    fn test_language_detection_from_extension() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        
        assert_eq!(highlighter.detect_language_from_extension("rs"), Some("Rust".to_string()));
        assert_eq!(highlighter.detect_language_from_extension("js"), Some("JavaScript".to_string()));
        assert_eq!(highlighter.detect_language_from_extension("py"), Some("Python".to_string()));
    }

    #[test]
    fn test_highlight_rust_code() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        let theme = Theme::default_dark();
        
        let code = r#"fn main() {
    println!("Hello, world!");
}"#;

        let result = highlighter.highlight_code(code, "Rust", &theme);
        assert!(result.is_ok());
        
        let highlight_result = result.unwrap();
        assert_eq!(highlight_result.language, "Rust");
        assert_eq!(highlight_result.total_lines, 3);
        assert!(!highlight_result.highlighted_lines.is_empty());
    }

    #[test]
    fn test_fallback_for_unknown_language() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        let theme = Theme::default_dark();
        
        let code = "Some random text that is not code";
        let result = highlighter.highlight_code(code, "unknown-language", &theme);
        
        assert!(result.is_ok());
        let highlight_result = result.unwrap();
        assert_eq!(highlight_result.language, "unknown-language");
    }

    #[test]
    fn test_color_parsing() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        
        // Test 6-digit hex
        let color = highlighter.parse_color("#ff0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        
        // Test 3-digit hex
        let color = highlighter.parse_color("#f00").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        
        // Test invalid color
        assert!(highlighter.parse_color("invalid").is_err());
    }
}