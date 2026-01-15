use crate::models::errors::AppError;
use crate::models::theme::Theme;
use crate::services::image_generator::{ImageGenerator, ExportOptions, ImageFormat, Resolution};
use crate::services::syntax_highlighter::{SyntaxHighlighter, HighlightResult};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use uuid::Uuid;

/// Enhanced export service with multiple format support
pub struct ExportService {
    image_generator: Arc<ImageGenerator>,
    syntax_highlighter: Arc<SyntaxHighlighter>,
}

/// Export result containing the generated image data and metadata
#[derive(Debug, Clone)]
pub struct ExportResult {
    pub data: Vec<u8>,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub file_size: usize,
    pub export_id: String,
}

/// Enhanced export options with additional configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedExportOptions {
    pub format: ImageFormat,
    pub resolution: Resolution,
    pub quality: u8, // 1-100 for JPEG
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub dpi: Option<u32>, // For high-quality exports
    pub compression_level: Option<u8>, // For PNG compression
    pub progressive: bool, // For progressive JPEG
    pub include_metadata: bool, // Include EXIF/metadata
}

impl ExportService {
    /// Creates a new ExportService
    pub fn new() -> Result<Self, AppError> {
        let image_generator = Arc::new(ImageGenerator::new()?);
        let syntax_highlighter = Arc::new(SyntaxHighlighter::new()
            .map_err(|e| AppError::image_generation_failed(format!("Failed to initialize syntax highlighter: {}", e)))?);

        Ok(ExportService {
            image_generator,
            syntax_highlighter,
        })
    }

    /// Export code snippet to image with enhanced options
    pub async fn export_code_snippet(
        &self,
        code: &str,
        language: &str,
        theme: &Theme,
        options: &EnhancedExportOptions,
    ) -> Result<ExportResult, AppError> {
        // Validate inputs
        if code.is_empty() {
            return Err(AppError::image_generation_failed("Code content cannot be empty"));
        }

        // Convert enhanced options to basic export options
        let basic_options = ExportOptions {
            format: options.format.clone(),
            resolution: options.resolution.clone(),
            quality: options.quality,
            width: options.width,
            height: options.height,
        };

        // Generate the image based on format
        let (data, width, height) = match options.format {
            ImageFormat::PNG => self.export_png(code, language, theme, &basic_options, options).await?,
            ImageFormat::JPEG => self.export_jpeg(code, language, theme, &basic_options, options).await?,
            ImageFormat::SVG => self.export_svg(code, language, theme, options).await?,
        };

        let export_id = Uuid::new_v4().to_string();
        let file_size = data.len();

        Ok(ExportResult {
            data,
            format: options.format.clone(),
            width,
            height,
            file_size,
            export_id,
        })
    }

    /// Export as PNG with enhanced options
    async fn export_png(
        &self,
        code: &str,
        language: &str,
        theme: &Theme,
        basic_options: &ExportOptions,
        enhanced_options: &EnhancedExportOptions,
    ) -> Result<(Vec<u8>, u32, u32), AppError> {
        // Generate the base image
        let image_data = self.image_generator.generate_image(code, language, theme, basic_options).await?;
        
        // If no enhanced options are needed, return the basic result
        if enhanced_options.compression_level.is_none() && enhanced_options.dpi.is_none() {
            // We need to decode the image to get dimensions
            let image = image::load_from_memory(&image_data)
                .map_err(|e| AppError::image_generation_failed(format!("Failed to decode generated image: {}", e)))?;
            
            return Ok((image_data, image.width(), image.height()));
        }

        // Decode the image for enhanced processing
        let image = image::load_from_memory(&image_data)
            .map_err(|e| AppError::image_generation_failed(format!("Failed to decode generated image: {}", e)))?;
        
        let rgba_image = image.to_rgba8();
        let width = rgba_image.width();
        let height = rgba_image.height();

        // Re-encode with enhanced options
        let mut buffer = Vec::new();
        
        use image::codecs::png::PngEncoder;
        use image::ImageEncoder;
        
        let encoder = PngEncoder::new(&mut buffer);
        
        encoder
            .write_image(
                rgba_image.as_raw(),
                width,
                height,
                image::ColorType::Rgba8,
            )
            .map_err(|e| AppError::image_generation_failed(format!("PNG encoding failed: {}", e)))?;

        Ok((buffer, width, height))
    }

    /// Export as JPEG with enhanced options
    async fn export_jpeg(
        &self,
        code: &str,
        language: &str,
        theme: &Theme,
        basic_options: &ExportOptions,
        enhanced_options: &EnhancedExportOptions,
    ) -> Result<(Vec<u8>, u32, u32), AppError> {
        // Generate the base image
        let image_data = self.image_generator.generate_image(code, language, theme, basic_options).await?;
        
        // Decode the image
        let image = image::load_from_memory(&image_data)
            .map_err(|e| AppError::image_generation_failed(format!("Failed to decode generated image: {}", e)))?;
        
        // Convert to RGB (JPEG doesn't support transparency)
        let rgb_image = image.to_rgb8();
        let width = rgb_image.width();
        let height = rgb_image.height();

        // Re-encode with enhanced options
        let mut buffer = Vec::new();
        
        use image::codecs::jpeg::JpegEncoder;
        use image::ImageEncoder;
        
        let encoder = JpegEncoder::new_with_quality(&mut buffer, enhanced_options.quality);
        
        // Set progressive encoding if requested
        if enhanced_options.progressive {
            // Note: The image crate doesn't directly support progressive JPEG in the current API
            // This would require a more advanced JPEG library like mozjpeg-rust
            tracing::warn!("Progressive JPEG encoding not yet implemented, using standard encoding");
        }
        
        encoder
            .write_image(
                rgb_image.as_raw(),
                width,
                height,
                image::ColorType::Rgb8,
            )
            .map_err(|e| AppError::image_generation_failed(format!("JPEG encoding failed: {}", e)))?;

        Ok((buffer, width, height))
    }

    /// Export as SVG with vector graphics
    async fn export_svg(
        &self,
        code: &str,
        language: &str,
        theme: &Theme,
        options: &EnhancedExportOptions,
    ) -> Result<(Vec<u8>, u32, u32), AppError> {
        // Perform syntax highlighting
        let highlight_result = self.syntax_highlighter
            .highlight_code(code, language, theme)
            .map_err(|e| AppError::image_generation_failed(format!("Syntax highlighting failed: {}", e)))?;

        // Calculate dimensions
        let scale_factor = match options.resolution {
            Resolution::Standard => 1.0,
            Resolution::High => 2.0,
            Resolution::Ultra => 3.0,
        };

        let font_size = theme.typography.font_size * scale_factor;
        let line_height = font_size * theme.typography.line_height;
        let char_width = font_size * 0.6; // Monospace approximation

        // Calculate content dimensions
        let max_line_length = highlight_result.highlighted_lines
            .iter()
            .map(|line| line.segments.iter().map(|seg| seg.text.len()).sum::<usize>())
            .max()
            .unwrap_or(0);

        let content_width = (max_line_length as f32 * char_width) as u32;
        let content_height = (highlight_result.total_lines as f32 * line_height) as u32;

        // Add padding
        let padding = 40.0 * scale_factor;
        let width = content_width + (padding * 2.0) as u32;
        let height = content_height + (padding * 2.0) as u32;

        // Generate SVG content
        let svg_content = self.generate_svg_content(
            &highlight_result,
            theme,
            width,
            height,
            padding,
            font_size,
            line_height,
            char_width,
        )?;

        Ok((svg_content.into_bytes(), width, height))
    }

    /// Generate SVG content for the code snippet
    fn generate_svg_content(
        &self,
        highlight_result: &HighlightResult,
        theme: &Theme,
        width: u32,
        height: u32,
        padding: f32,
        font_size: f32,
        line_height: f32,
        char_width: f32,
    ) -> Result<String, AppError> {
        let mut svg = String::new();
        
        // SVG header
        svg.push_str(&format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">\n<defs>\n<style type=\"text/css\">\n<![CDATA[\n.code-text {{\n    font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;\n    font-size: {}px;\n    line-height: {};\n}}\n]]>\n</style>\n</defs>\n",
            width, height, font_size, line_height / font_size
        ));

        // Background
        svg.push_str(&format!(
            "<rect width=\"100%\" height=\"100%\" fill=\"{}\"/>",
            theme.background.primary
        ));

        // Add gradient background if specified
        if let crate::models::theme::BackgroundType::Gradient = theme.background.bg_type {
            if let Some(ref secondary) = theme.background.secondary {
                svg.push_str(&format!(
                    "<defs>\n<linearGradient id=\"bg-gradient\" x1=\"0%\" y1=\"0%\" x2=\"0%\" y2=\"100%\">\n<stop offset=\"0%\" style=\"stop-color:{};stop-opacity:1\" />\n<stop offset=\"100%\" style=\"stop-color:{};stop-opacity:1\" />\n</linearGradient>\n</defs>\n<rect width=\"100%\" height=\"100%\" fill=\"url(#bg-gradient)\"/>",
                    theme.background.primary, secondary
                ));
            }
        }

        // Window frame if enabled
        if theme.window.show_title_bar {
            let title_bar_height = 30.0;
            let stroke_color = "#cccccc";
            svg.push_str(&format!(
                "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#f0f0f0\" stroke=\"{}\" stroke-width=\"1\"/>",
                width, title_bar_height, stroke_color
            ));

            // Window controls
            if theme.window.show_controls {
                svg.push_str("<circle cx=\"15\" cy=\"15\" r=\"6\" fill=\"#ff5f57\"/>");
                svg.push_str("<circle cx=\"35\" cy=\"15\" r=\"6\" fill=\"#ffbd2e\"/>");
                svg.push_str("<circle cx=\"55\" cy=\"15\" r=\"6\" fill=\"#28ca42\"/>");
            }

            // Title text
            if let Some(ref title) = theme.window.title {
                let text_color = "#333333";
                svg.push_str(&format!(
                    "<text x=\"{}\" y=\"20\" class=\"code-text\" fill=\"{}\" text-anchor=\"middle\">{}</text>",
                    width / 2, text_color, title
                ));
            }
        }

        // Code content
        let mut y = padding + font_size;
        if theme.window.show_title_bar {
            y += 30.0;
        }

        for line in &highlight_result.highlighted_lines {
            let mut x = padding;

            // Line number if enabled
            if theme.typography.show_line_numbers {
                let line_color = "#666666";
                svg.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" class=\"code-text\" fill=\"{}\">{:3}</text>",
                    x, y, line_color, line.line_number
                ));
                x += 50.0; // Line number width
            }

            // Code segments
            for segment in &line.segments {
                if !segment.text.is_empty() {
                    let escaped_text = segment.text
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('"', "&quot;")
                        .replace('\'', "&#39;");

                    svg.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" class=\"code-text\" fill=\"{}\">{}</text>",
                        x, y, segment.style.color, escaped_text
                    ));

                    x += segment.text.len() as f32 * char_width;
                }
            }

            y += line_height;
        }

        // SVG footer
        svg.push_str("</svg>");

        Ok(svg)
    }

    /// Get supported export formats
    pub fn supported_formats() -> Vec<ImageFormat> {
        vec![ImageFormat::PNG, ImageFormat::JPEG, ImageFormat::SVG]
    }

    /// Get supported resolutions
    pub fn supported_resolutions() -> Vec<Resolution> {
        vec![Resolution::Standard, Resolution::High, Resolution::Ultra]
    }

    /// Validate export options
    pub fn validate_options(options: &EnhancedExportOptions) -> Result<(), AppError> {
        // Validate quality for JPEG
        if options.format == ImageFormat::JPEG && (options.quality < 1 || options.quality > 100) {
            return Err(AppError::image_generation_failed("JPEG quality must be between 1 and 100"));
        }

        // Validate compression level for PNG
        if let Some(compression) = options.compression_level {
            if compression > 9 {
                return Err(AppError::image_generation_failed("PNG compression level must be between 0 and 9"));
            }
        }

        // Validate dimensions
        if let Some(width) = options.width {
            if width < 100 || width > 8000 {
                return Err(AppError::image_generation_failed("Width must be between 100 and 8000 pixels"));
            }
        }

        if let Some(height) = options.height {
            if height < 100 || height > 8000 {
                return Err(AppError::image_generation_failed("Height must be between 100 and 8000 pixels"));
            }
        }

        Ok(())
    }
}

impl Default for EnhancedExportOptions {
    fn default() -> Self {
        EnhancedExportOptions {
            format: ImageFormat::PNG,
            resolution: Resolution::Standard,
            quality: 90,
            width: None,
            height: None,
            dpi: None,
            compression_level: None,
            progressive: false,
            include_metadata: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::theme::Theme;

    #[tokio::test]
    async fn test_export_service_creation() {
        let service = ExportService::new();
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_export_png() {
        let service = ExportService::new().unwrap();
        let theme = Theme::default_dark();
        let options = EnhancedExportOptions {
            format: ImageFormat::PNG,
            ..Default::default()
        };
        
        let code = r#"fn main() {
    println!("Hello, world!");
}"#;

        let result = service.export_code_snippet(code, "Rust", &theme, &options).await;
        assert!(result.is_ok());
        
        let export_result = result.unwrap();
        assert_eq!(export_result.format, ImageFormat::PNG);
        assert!(!export_result.data.is_empty());
        assert!(export_result.width > 0);
        assert!(export_result.height > 0);
    }

    #[tokio::test]
    async fn test_export_jpeg() {
        let service = ExportService::new().unwrap();
        let theme = Theme::default_light();
        let options = EnhancedExportOptions {
            format: ImageFormat::JPEG,
            quality: 85,
            ..Default::default()
        };
        
        let code = "console.log('Hello, JavaScript!');";

        let result = service.export_code_snippet(code, "JavaScript", &theme, &options).await;
        assert!(result.is_ok());
        
        let export_result = result.unwrap();
        assert_eq!(export_result.format, ImageFormat::JPEG);
        assert!(!export_result.data.is_empty());
    }

    #[tokio::test]
    async fn test_export_svg() {
        let service = ExportService::new().unwrap();
        let theme = Theme::default_dark();
        let options = EnhancedExportOptions {
            format: ImageFormat::SVG,
            ..Default::default()
        };
        
        let code = "print('Hello, Python!')";

        let result = service.export_code_snippet(code, "Python", &theme, &options).await;
        assert!(result.is_ok());
        
        let export_result = result.unwrap();
        assert_eq!(export_result.format, ImageFormat::SVG);
        assert!(!export_result.data.is_empty());
        
        // Verify it's valid SVG
        let svg_content = String::from_utf8(export_result.data).unwrap();
        assert!(svg_content.contains("<svg"));
        assert!(svg_content.contains("</svg>"));
    }

    #[tokio::test]
    async fn test_different_resolutions() {
        let service = ExportService::new().unwrap();
        let theme = Theme::default_dark();
        let code = "package main\n\nfunc main() {\n    fmt.Println(\"Hello, Go!\")\n}";
        
        for resolution in [Resolution::Standard, Resolution::High, Resolution::Ultra] {
            let options = EnhancedExportOptions {
                resolution,
                ..Default::default()
            };
            
            let result = service.export_code_snippet(code, "Go", &theme, &options).await;
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_validate_options() {
        // Valid options
        let valid_options = EnhancedExportOptions::default();
        assert!(ExportService::validate_options(&valid_options).is_ok());

        // Invalid JPEG quality
        let invalid_jpeg = EnhancedExportOptions {
            format: ImageFormat::JPEG,
            quality: 101,
            ..Default::default()
        };
        assert!(ExportService::validate_options(&invalid_jpeg).is_err());

        // Invalid PNG compression
        let invalid_png = EnhancedExportOptions {
            compression_level: Some(10),
            ..Default::default()
        };
        assert!(ExportService::validate_options(&invalid_png).is_err());

        // Invalid dimensions
        let invalid_width = EnhancedExportOptions {
            width: Some(50),
            ..Default::default()
        };
        assert!(ExportService::validate_options(&invalid_width).is_err());
    }

    #[test]
    fn test_supported_formats() {
        let formats = ExportService::supported_formats();
        assert!(formats.contains(&ImageFormat::PNG));
        assert!(formats.contains(&ImageFormat::JPEG));
        assert!(formats.contains(&ImageFormat::SVG));
    }

    #[test]
    fn test_supported_resolutions() {
        let resolutions = ExportService::supported_resolutions();
        assert!(resolutions.contains(&Resolution::Standard));
        assert!(resolutions.contains(&Resolution::High));
        assert!(resolutions.contains(&Resolution::Ultra));
    }
}