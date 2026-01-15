use crate::models::errors::AppError;
use crate::models::theme::{Theme, BackgroundType, WindowStyleType};
use crate::services::syntax_highlighter::{SyntaxHighlighter, HighlightResult};
use image::{ImageBuffer, Rgba, RgbaImage};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration options for image export
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ImageFormat,
    pub resolution: Resolution,
    pub quality: u8, // 1-100 for JPEG
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImageFormat {
    PNG,
    JPEG,
    SVG,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Resolution {
    Standard, // 1x
    High,     // 2x
    Ultra,    // 3x
}

/// Layout configuration for code rendering
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub padding: Padding,
    pub margin: Margin,
    pub line_height: f32,
    pub font_size: f32,
    pub max_width: u32,
    pub min_width: u32,
    pub show_line_numbers: bool,
    pub line_number_width: u32,
}

#[derive(Debug, Clone)]
pub struct Padding {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

#[derive(Debug, Clone)]
pub struct Margin {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

/// Calculated dimensions for the final image
#[derive(Debug, Clone)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
    pub content_width: u32,
    pub content_height: u32,
    pub code_area_x: u32,
    pub code_area_y: u32,
    pub code_area_width: u32,
    pub code_area_height: u32,
}

/// Font metrics for text rendering calculations
#[derive(Debug, Clone)]
pub struct FontMetrics {
    pub char_width: f32,
    pub line_height: f32,
    pub ascent: f32,
    pub descent: f32,
}

/// Core image generation service
pub struct ImageGenerator {
    syntax_highlighter: Arc<SyntaxHighlighter>,
    font_cache: HashMap<String, FontMetrics>,
}

impl ImageGenerator {
    /// Creates a new ImageGenerator with syntax highlighting support
    pub fn new() -> Result<Self, AppError> {
        let syntax_highlighter = Arc::new(
            SyntaxHighlighter::new()
                .map_err(|e| AppError::image_generation_failed(format!("Failed to initialize syntax highlighter: {}", e)))?
        );

        Ok(ImageGenerator {
            syntax_highlighter,
            font_cache: HashMap::new(),
        })
    }

    /// Generates a styled code snippet image
    pub async fn generate_image(
        &self,
        code: &str,
        language: &str,
        theme: &Theme,
        options: &ExportOptions,
    ) -> Result<Vec<u8>, AppError> {
        // Validate inputs
        if code.is_empty() {
            return Err(AppError::image_generation_failed("Code content cannot be empty"));
        }

        // Perform syntax highlighting
        let highlight_result = self.syntax_highlighter
            .highlight_code(code, language, theme)
            .map_err(|e| AppError::image_generation_failed(format!("Syntax highlighting failed: {}", e)))?;

        // Calculate layout dimensions
        let layout_config = self.create_layout_config(theme, options)?;
        let dimensions = self.calculate_dimensions(&highlight_result, &layout_config, options)?;

        // Create the image buffer
        let mut image = self.create_base_image(&dimensions, theme)?;

        // Render the background
        self.render_background(&mut image, &dimensions, theme)?;

        // Render window frame if needed
        if theme.window.show_title_bar || theme.window.show_controls {
            self.render_window_frame(&mut image, &dimensions, theme)?;
        }

        // Render the code content
        self.render_code_content(&mut image, &highlight_result, &dimensions, &layout_config, theme)?;

        // Apply border radius if specified
        if theme.window.border_radius > 0.0 {
            self.apply_border_radius(&mut image, theme.window.border_radius)?;
        }

        // Apply shadow if enabled
        if theme.window.shadow {
            image = self.apply_shadow(&mut image, (4, 4), 8)?;
        }

        // Convert to requested format
        self.encode_image(image, options).await
    }

    /// Creates layout configuration based on theme and export options
    fn create_layout_config(&self, theme: &Theme, options: &ExportOptions) -> Result<LayoutConfig, AppError> {
        let scale_factor = match options.resolution {
            Resolution::Standard => 1.0,
            Resolution::High => 2.0,
            Resolution::Ultra => 3.0,
        };

        let font_size = theme.typography.font_size * scale_factor;
        let line_height = font_size * theme.typography.line_height;

        // Calculate padding based on window style and scale
        let base_padding = match theme.window.style_type {
            WindowStyleType::MacOS => 40,
            WindowStyleType::Windows => 35,
            WindowStyleType::Terminal => 20,
            WindowStyleType::Clean => 30,
        };

        let scaled_padding = (base_padding as f32 * scale_factor) as u32;

        // Add extra padding for title bar
        let title_bar_height = if theme.window.show_title_bar { 
            (30.0 * scale_factor) as u32 
        } else { 
            0 
        };

        Ok(LayoutConfig {
            padding: Padding {
                top: scaled_padding + title_bar_height,
                right: scaled_padding,
                bottom: scaled_padding,
                left: scaled_padding,
            },
            margin: Margin {
                top: (10.0 * scale_factor) as u32,
                right: (10.0 * scale_factor) as u32,
                bottom: (10.0 * scale_factor) as u32,
                left: (10.0 * scale_factor) as u32,
            },
            line_height,
            font_size,
            max_width: options.width.unwrap_or((800.0 * scale_factor) as u32),
            min_width: (400.0 * scale_factor) as u32,
            show_line_numbers: theme.typography.show_line_numbers,
            line_number_width: if theme.typography.show_line_numbers {
                (50.0 * scale_factor) as u32
            } else {
                0
            },
        })
    }

    /// Calculates the required image dimensions
    fn calculate_dimensions(
        &self,
        highlight_result: &HighlightResult,
        layout_config: &LayoutConfig,
        options: &ExportOptions,
    ) -> Result<ImageDimensions, AppError> {
        // Get font metrics for calculations
        let font_metrics = self.get_font_metrics(layout_config.font_size)?;

        // Calculate content dimensions
        let max_line_length = highlight_result.highlighted_lines
            .iter()
            .map(|line| line.segments.iter().map(|seg| seg.text.len()).sum::<usize>())
            .max()
            .unwrap_or(0);

        let content_width = (max_line_length as f32 * font_metrics.char_width) as u32 
            + layout_config.line_number_width;
        
        let content_height = (highlight_result.total_lines as f32 * layout_config.line_height) as u32;

        // Calculate total image dimensions
        let total_padding_width = layout_config.padding.left + layout_config.padding.right 
            + layout_config.margin.left + layout_config.margin.right;
        let total_padding_height = layout_config.padding.top + layout_config.padding.bottom 
            + layout_config.margin.top + layout_config.margin.bottom;

        let width = std::cmp::max(
            content_width + total_padding_width,
            layout_config.min_width
        );
        let width = if let Some(max_w) = options.width {
            std::cmp::min(width, max_w)
        } else {
            std::cmp::min(width, layout_config.max_width)
        };

        let height = content_height + total_padding_height;
        let height = if let Some(max_h) = options.height {
            std::cmp::min(height, max_h)
        } else {
            height
        };

        // Calculate code area position and size
        let code_area_x = layout_config.margin.left + layout_config.padding.left;
        let code_area_y = layout_config.margin.top + layout_config.padding.top;
        let code_area_width = width - total_padding_width;
        let code_area_height = height - total_padding_height;

        Ok(ImageDimensions {
            width,
            height,
            content_width,
            content_height,
            code_area_x,
            code_area_y,
            code_area_width,
            code_area_height,
        })
    }

    /// Creates the base image buffer with proper dimensions
    fn create_base_image(&self, dimensions: &ImageDimensions, theme: &Theme) -> Result<RgbaImage, AppError> {
        let background_color = self.parse_color(&theme.background.primary)?;
        let mut image = ImageBuffer::from_pixel(
            dimensions.width,
            dimensions.height,
            background_color,
        );

        // Apply opacity if needed
        if theme.background.opacity < 1.0 {
            for pixel in image.pixels_mut() {
                pixel[3] = (pixel[3] as f32 * theme.background.opacity) as u8;
            }
        }

        Ok(image)
    }

    /// Renders the background according to the theme
    fn render_background(
        &self,
        image: &mut RgbaImage,
        dimensions: &ImageDimensions,
        theme: &Theme,
    ) -> Result<(), AppError> {
        match theme.background.bg_type {
            BackgroundType::Solid => {
                // Already handled in create_base_image
                Ok(())
            }
            BackgroundType::Gradient => {
                self.render_gradient_background(image, dimensions, theme)
            }
            BackgroundType::Pattern => {
                self.render_pattern_background(image, dimensions, theme)
            }
        }
    }

    /// Renders a gradient background
    fn render_gradient_background(
        &self,
        image: &mut RgbaImage,
        dimensions: &ImageDimensions,
        theme: &Theme,
    ) -> Result<(), AppError> {
        let primary_color = self.parse_color(&theme.background.primary)?;
        let secondary_color = if let Some(ref secondary) = theme.background.secondary {
            self.parse_color(secondary)?
        } else {
            // Create a slightly lighter/darker version of primary
            let mut color = primary_color;
            color[0] = ((color[0] as u16 + 30).min(255)) as u8;
            color[1] = ((color[1] as u16 + 30).min(255)) as u8;
            color[2] = ((color[2] as u16 + 30).min(255)) as u8;
            color
        };

        // Vertical gradient from top to bottom
        for y in 0..dimensions.height {
            let ratio = y as f32 / dimensions.height as f32;
            let blended_color = self.blend_colors(primary_color, secondary_color, ratio);
            
            for x in 0..dimensions.width {
                image.put_pixel(x, y, blended_color);
            }
        }

        Ok(())
    }

    /// Renders a pattern background (simplified dot pattern)
    fn render_pattern_background(
        &self,
        image: &mut RgbaImage,
        dimensions: &ImageDimensions,
        theme: &Theme,
    ) -> Result<(), AppError> {
        let base_color = self.parse_color(&theme.background.primary)?;
        let pattern_color = if let Some(ref secondary) = theme.background.secondary {
            self.parse_color(secondary)?
        } else {
            // Create a slightly different shade for pattern
            let mut color = base_color;
            color[0] = ((color[0] as i16 + 20).max(0).min(255)) as u8;
            color[1] = ((color[1] as i16 + 20).max(0).min(255)) as u8;
            color[2] = ((color[2] as i16 + 20).max(0).min(255)) as u8;
            color
        };

        // Fill with base color first
        for pixel in image.pixels_mut() {
            *pixel = base_color;
        }

        // Add dot pattern
        let dot_spacing = 20;
        let dot_size = 2;

        for y in (0..dimensions.height).step_by(dot_spacing) {
            for x in (0..dimensions.width).step_by(dot_spacing) {
                // Draw small dots
                for dy in 0..dot_size {
                    for dx in 0..dot_size {
                        let px = x + dx;
                        let py = y + dy;
                        if px < dimensions.width && py < dimensions.height {
                            image.put_pixel(px, py, pattern_color);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Gets font metrics for the given font size (simplified calculation)
    fn get_font_metrics(&self, font_size: f32) -> Result<FontMetrics, AppError> {
        // Simplified font metrics calculation
        // In a real implementation, you'd use a proper font rendering library
        Ok(FontMetrics {
            char_width: font_size * 0.6, // Monospace approximation
            line_height: font_size * 1.2,
            ascent: font_size * 0.8,
            descent: font_size * 0.2,
        })
    }

    /// Parses a hex color string to RGBA
    fn parse_color(&self, color_str: &str) -> Result<Rgba<u8>, AppError> {
        if !color_str.starts_with('#') {
            return Err(AppError::image_generation_failed(format!("Invalid color format: {}", color_str)));
        }

        let hex = &color_str[1..];
        let (r, g, b, a) = match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|_| AppError::image_generation_failed("Invalid hex color"))?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|_| AppError::image_generation_failed("Invalid hex color"))?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|_| AppError::image_generation_failed("Invalid hex color"))?;
                (r, g, b, 255)
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| AppError::image_generation_failed("Invalid hex color"))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| AppError::image_generation_failed("Invalid hex color"))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| AppError::image_generation_failed("Invalid hex color"))?;
                (r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| AppError::image_generation_failed("Invalid hex color"))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| AppError::image_generation_failed("Invalid hex color"))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| AppError::image_generation_failed("Invalid hex color"))?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|_| AppError::image_generation_failed("Invalid hex color"))?;
                (r, g, b, a)
            }
            _ => return Err(AppError::image_generation_failed("Invalid hex color length")),
        };

        Ok(Rgba([r, g, b, a]))
    }

    /// Blends two colors with the given ratio (0.0 = first color, 1.0 = second color)
    fn blend_colors(&self, color1: Rgba<u8>, color2: Rgba<u8>, ratio: f32) -> Rgba<u8> {
        let ratio = ratio.clamp(0.0, 1.0);
        let inv_ratio = 1.0 - ratio;

        Rgba([
            (color1[0] as f32 * inv_ratio + color2[0] as f32 * ratio) as u8,
            (color1[1] as f32 * inv_ratio + color2[1] as f32 * ratio) as u8,
            (color1[2] as f32 * inv_ratio + color2[2] as f32 * ratio) as u8,
            (color1[3] as f32 * inv_ratio + color2[3] as f32 * ratio) as u8,
        ])
    }

    /// Renders window frame according to the window style
    fn render_window_frame(
        &self,
        image: &mut RgbaImage,
        dimensions: &ImageDimensions,
        theme: &Theme,
    ) -> Result<(), AppError> {
        match theme.window.style_type {
            WindowStyleType::MacOS => self.render_macos_window_frame(image, dimensions, theme),
            WindowStyleType::Windows => self.render_windows_window_frame(image, dimensions, theme),
            WindowStyleType::Terminal => self.render_terminal_window_frame(image, dimensions, theme),
            WindowStyleType::Clean => self.render_clean_window_frame(image, dimensions, theme),
        }
    }

    /// Renders macOS-style window frame with traffic light controls
    fn render_macos_window_frame(
        &self,
        image: &mut RgbaImage,
        dimensions: &ImageDimensions,
        theme: &Theme,
    ) -> Result<(), AppError> {
        if !theme.window.show_title_bar {
            return Ok(());
        }

        let title_bar_height = 30;
        let title_bar_color = self.get_title_bar_color(theme)?;
        
        // Draw title bar background
        for y in 0..title_bar_height {
            for x in 0..dimensions.width {
                image.put_pixel(x, y, title_bar_color);
            }
        }

        // Draw traffic light controls if enabled
        if theme.window.show_controls {
            self.render_macos_traffic_lights(image, 15, 15)?;
        }

        // Draw title text if provided
        if let Some(ref title) = theme.window.title {
            let title_x = dimensions.width / 2 - (title.len() as u32 * 6) / 2; // Centered
            let title_y = 8.0;
            let title_color = self.parse_color("#333333")?;
            let font_metrics = self.get_font_metrics(12.0)?;
            
            self.render_text(image, title, title_x as f32, title_y, &font_metrics, title_color)?;
        }

        // Draw bottom border
        let border_color = self.parse_color("#cccccc")?;
        for x in 0..dimensions.width {
            image.put_pixel(x, title_bar_height - 1, border_color);
        }

        Ok(())
    }

    /// Renders Windows-style window frame
    fn render_windows_window_frame(
        &self,
        image: &mut RgbaImage,
        dimensions: &ImageDimensions,
        theme: &Theme,
    ) -> Result<(), AppError> {
        if !theme.window.show_title_bar {
            return Ok(());
        }

        let title_bar_height = 30;
        let title_bar_color = self.get_title_bar_color(theme)?;
        
        // Draw title bar background
        for y in 0..title_bar_height {
            for x in 0..dimensions.width {
                image.put_pixel(x, y, title_bar_color);
            }
        }

        // Draw window controls if enabled
        if theme.window.show_controls {
            self.render_windows_controls(image, dimensions.width - 100, 5)?;
        }

        // Draw title text if provided
        if let Some(ref title) = theme.window.title {
            let title_x = 10.0;
            let title_y = 8.0;
            let title_color = self.parse_color("#333333")?;
            let font_metrics = self.get_font_metrics(12.0)?;
            
            self.render_text(image, title, title_x, title_y, &font_metrics, title_color)?;
        }

        Ok(())
    }

    /// Renders terminal-style window frame
    fn render_terminal_window_frame(
        &self,
        image: &mut RgbaImage,
        dimensions: &ImageDimensions,
        theme: &Theme,
    ) -> Result<(), AppError> {
        if !theme.window.show_title_bar {
            return Ok(());
        }

        let title_bar_height = 25;
        let title_bar_color = self.parse_color("#2d2d2d")?;
        
        // Draw title bar background (darker for terminal style)
        for y in 0..title_bar_height {
            for x in 0..dimensions.width {
                image.put_pixel(x, y, title_bar_color);
            }
        }

        // Draw simple controls if enabled
        if theme.window.show_controls {
            self.render_terminal_controls(image, dimensions.width - 60, 8)?;
        }

        // Draw title text if provided
        if let Some(ref title) = theme.window.title {
            let title_x = 10.0;
            let title_y = 6.0;
            let title_color = self.parse_color("#ffffff")?;
            let font_metrics = self.get_font_metrics(11.0)?;
            
            self.render_text(image, title, title_x, title_y, &font_metrics, title_color)?;
        }

        Ok(())
    }

    /// Renders clean/minimal window frame
    fn render_clean_window_frame(
        &self,
        image: &mut RgbaImage,
        dimensions: &ImageDimensions,
        theme: &Theme,
    ) -> Result<(), AppError> {
        if !theme.window.show_title_bar {
            return Ok(());
        }

        let title_bar_height = 20;
        let title_bar_color = self.get_title_bar_color(theme)?;
        
        // Draw minimal title bar
        for y in 0..title_bar_height {
            for x in 0..dimensions.width {
                image.put_pixel(x, y, title_bar_color);
            }
        }

        // Draw title text if provided (no controls in clean style)
        if let Some(ref title) = theme.window.title {
            let title_x = dimensions.width / 2 - (title.len() as u32 * 5) / 2; // Centered
            let title_y = 4.0;
            let title_color = self.parse_color("#666666")?;
            let font_metrics = self.get_font_metrics(10.0)?;
            
            self.render_text(image, title, title_x as f32, title_y, &font_metrics, title_color)?;
        }

        Ok(())
    }

    /// Renders macOS traffic light controls (red, yellow, green circles)
    fn render_macos_traffic_lights(&self, image: &mut RgbaImage, x: u32, y: u32) -> Result<(), AppError> {
        let colors = [
            self.parse_color("#ff5f57")?, // Red
            self.parse_color("#ffbd2e")?, // Yellow
            self.parse_color("#28ca42")?, // Green
        ];

        let radius = 6;
        let spacing = 20;

        for (i, color) in colors.iter().enumerate() {
            let center_x = x + (i as u32 * spacing);
            let center_y = y;
            
            self.render_circle(image, center_x, center_y, radius, *color)?;
        }

        Ok(())
    }

    /// Renders Windows-style window controls (minimize, maximize, close)
    fn render_windows_controls(&self, image: &mut RgbaImage, x: u32, y: u32) -> Result<(), AppError> {
        let button_width = 25;
        let button_height = 20;
        let button_color = self.parse_color("#e1e1e1")?;
        let symbol_color = self.parse_color("#333333")?;

        // Draw three buttons
        for i in 0..3 {
            let button_x = x + (i * button_width);
            
            // Draw button background
            for by in 0..button_height {
                for bx in 0..button_width {
                    let px = button_x + bx;
                    let py = y + by;
                    if px < image.width() && py < image.height() {
                        image.put_pixel(px, py, button_color);
                    }
                }
            }

            // Draw button symbols (simplified)
            let symbol_x = button_x + button_width / 2;
            let symbol_y = y + button_height / 2;
            
            match i {
                0 => self.render_minimize_symbol(image, symbol_x, symbol_y, symbol_color)?, // Minimize
                1 => self.render_maximize_symbol(image, symbol_x, symbol_y, symbol_color)?, // Maximize
                2 => self.render_close_symbol(image, symbol_x, symbol_y, symbol_color)?,    // Close
                _ => {}
            }
        }

        Ok(())
    }

    /// Renders terminal-style controls (simple dots or squares)
    fn render_terminal_controls(&self, image: &mut RgbaImage, x: u32, y: u32) -> Result<(), AppError> {
        let dot_color = self.parse_color("#666666")?;
        let spacing = 15;

        for i in 0..3 {
            let dot_x = x + (i * spacing);
            let dot_y = y;
            
            self.render_circle(image, dot_x, dot_y, 3, dot_color)?;
        }

        Ok(())
    }

    /// Renders a filled circle
    fn render_circle(&self, image: &mut RgbaImage, center_x: u32, center_y: u32, radius: u32, color: Rgba<u8>) -> Result<(), AppError> {
        let radius_sq = (radius * radius) as i32;
        
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                if dx * dx + dy * dy <= radius_sq {
                    let x = (center_x as i32 + dx) as u32;
                    let y = (center_y as i32 + dy) as u32;
                    
                    if x < image.width() && y < image.height() {
                        image.put_pixel(x, y, color);
                    }
                }
            }
        }

        Ok(())
    }

    /// Renders minimize symbol (horizontal line)
    fn render_minimize_symbol(&self, image: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) -> Result<(), AppError> {
        for dx in -4..=4 {
            let px = (x as i32 + dx) as u32;
            if px < image.width() && y < image.height() {
                image.put_pixel(px, y, color);
            }
        }
        Ok(())
    }

    /// Renders maximize symbol (square outline)
    fn render_maximize_symbol(&self, image: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) -> Result<(), AppError> {
        let size = 6;
        
        // Draw square outline
        for i in 0..size {
            // Top and bottom lines
            let top_x = (x as i32 - size/2 + i) as u32;
            let bottom_x = top_x;
            let top_y = (y as i32 - size/2) as u32;
            let bottom_y = (y as i32 + size/2) as u32;
            
            if top_x < image.width() && top_y < image.height() {
                image.put_pixel(top_x, top_y, color);
            }
            if bottom_x < image.width() && bottom_y < image.height() {
                image.put_pixel(bottom_x, bottom_y, color);
            }
            
            // Left and right lines
            let left_x = (x as i32 - size/2) as u32;
            let right_x = (x as i32 + size/2) as u32;
            let left_y = (y as i32 - size/2 + i) as u32;
            let right_y = left_y;
            
            if left_x < image.width() && left_y < image.height() {
                image.put_pixel(left_x, left_y, color);
            }
            if right_x < image.width() && right_y < image.height() {
                image.put_pixel(right_x, right_y, color);
            }
        }
        
        Ok(())
    }

    /// Renders close symbol (X)
    fn render_close_symbol(&self, image: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) -> Result<(), AppError> {
        let size = 4;
        
        // Draw X shape
        for i in -size..=size {
            // Diagonal from top-left to bottom-right
            let x1 = (x as i32 + i) as u32;
            let y1 = (y as i32 + i) as u32;
            if x1 < image.width() && y1 < image.height() {
                image.put_pixel(x1, y1, color);
            }
            
            // Diagonal from top-right to bottom-left
            let x2 = (x as i32 + i) as u32;
            let y2 = (y as i32 - i) as u32;
            if x2 < image.width() && y2 < image.height() {
                image.put_pixel(x2, y2, color);
            }
        }
        
        Ok(())
    }

    /// Gets appropriate title bar color based on theme
    fn get_title_bar_color(&self, theme: &Theme) -> Result<Rgba<u8>, AppError> {
        // Use a slightly lighter/darker version of the background color
        let base_color = self.parse_color(&theme.background.primary)?;
        
        // Determine if we should go lighter or darker
        let luminance = 0.299 * base_color[0] as f32 + 0.587 * base_color[1] as f32 + 0.114 * base_color[2] as f32;
        
        let adjustment = if luminance > 128.0 { -30 } else { 30 };
        
        Ok(Rgba([
            ((base_color[0] as i16 + adjustment).max(0).min(255)) as u8,
            ((base_color[1] as i16 + adjustment).max(0).min(255)) as u8,
            ((base_color[2] as i16 + adjustment).max(0).min(255)) as u8,
            base_color[3],
        ]))
    }

    /// Applies border radius to the image (simplified corner rounding)
    fn apply_border_radius(&self, image: &mut RgbaImage, radius: f32) -> Result<(), AppError> {
        if radius <= 0.0 {
            return Ok(());
        }

        let width = image.width();
        let height = image.height();
        let radius = radius as u32;
        
        // Clear corners outside the border radius
        for y in 0..radius {
            for x in 0..radius {
                let dx = radius - x;
                let dy = radius - y;
                let distance_sq = dx * dx + dy * dy;
                
                if distance_sq > radius * radius {
                    image.put_pixel(x, y, Rgba([0, 0, 0, 0])); // Transparent
                }
            }
        }
        
        // Repeat for other corners
        for y in 0..radius {
            for x in (width - radius)..width {
                let dx = x - (width - radius - 1);
                let dy = radius - y;
                let distance_sq = dx * dx + dy * dy;
                
                if distance_sq > radius * radius {
                    image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }
        
        for y in (height - radius)..height {
            for x in 0..radius {
                let dx = radius - x;
                let dy = y - (height - radius - 1);
                let distance_sq = dx * dx + dy * dy;
                
                if distance_sq > radius * radius {
                    image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }
        
        for y in (height - radius)..height {
            for x in (width - radius)..width {
                let dx = x - (width - radius - 1);
                let dy = y - (height - radius - 1);
                let distance_sq = dx * dx + dy * dy;
                
                if distance_sq > radius * radius {
                    image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }

        Ok(())
    }

    /// Applies shadow effect to the image
    fn apply_shadow(&self, image: &mut RgbaImage, shadow_offset: (i32, i32), shadow_blur: u32) -> Result<RgbaImage, AppError> {
        let (offset_x, offset_y) = shadow_offset;
        let shadow_color = Rgba([0, 0, 0, 80]); // Semi-transparent black
        
        let new_width = (image.width() as i32 + offset_x.abs() + shadow_blur as i32 * 2) as u32;
        let new_height = (image.height() as i32 + offset_y.abs() + shadow_blur as i32 * 2) as u32;
        
        let mut shadow_image = ImageBuffer::from_pixel(new_width, new_height, Rgba([0, 0, 0, 0]));
        
        // Calculate where to place the original image
        let image_x = if offset_x < 0 { offset_x.abs() as u32 } else { 0 };
        let image_y = if offset_y < 0 { offset_y.abs() as u32 } else { 0 };
        
        // Calculate where to place the shadow
        let shadow_x = if offset_x > 0 { offset_x as u32 } else { 0 };
        let shadow_y = if offset_y > 0 { offset_y as u32 } else { 0 };
        
        // Draw shadow (simplified - just offset the image)
        for y in 0..image.height() {
            for x in 0..image.width() {
                let pixel = image.get_pixel(x, y);
                if pixel[3] > 0 { // If not transparent
                    let shadow_px = shadow_x + x;
                    let shadow_py = shadow_y + y;
                    if shadow_px < shadow_image.width() && shadow_py < shadow_image.height() {
                        shadow_image.put_pixel(shadow_px, shadow_py, shadow_color);
                    }
                }
            }
        }
        
        // Draw original image on top
        for y in 0..image.height() {
            for x in 0..image.width() {
                let pixel = *image.get_pixel(x, y);
                let final_x = image_x + x;
                let final_y = image_y + y;
                if final_x < shadow_image.width() && final_y < shadow_image.height() {
                    shadow_image.put_pixel(final_x, final_y, pixel);
                }
            }
        }
        
        Ok(shadow_image)
    }

    /// Renders the code content with syntax highlighting
    fn render_code_content(
        &self,
        image: &mut RgbaImage,
        highlight_result: &HighlightResult,
        dimensions: &ImageDimensions,
        layout_config: &LayoutConfig,
        _theme: &Theme,
    ) -> Result<(), AppError> {
        let font_metrics = self.get_font_metrics(layout_config.font_size)?;
        let mut current_y = dimensions.code_area_y as f32;

        for line in &highlight_result.highlighted_lines {
            let mut current_x = dimensions.code_area_x as f32;

            // Render line number if enabled
            if layout_config.show_line_numbers {
                let line_number = format!("{:3}", line.line_number);
                let line_number_color = self.parse_color("#666666")?; // Gray for line numbers
                
                self.render_text(
                    image,
                    &line_number,
                    current_x,
                    current_y,
                    &font_metrics,
                    line_number_color,
                )?;
                
                current_x += layout_config.line_number_width as f32;
            }

            // Render code segments with syntax highlighting
            for segment in &line.segments {
                let color = self.parse_color(&segment.style.color)?;
                
                self.render_text(
                    image,
                    &segment.text,
                    current_x,
                    current_y,
                    &font_metrics,
                    color,
                )?;
                
                current_x += segment.text.len() as f32 * font_metrics.char_width;
            }

            current_y += layout_config.line_height;
        }

        Ok(())
    }

    /// Renders text at the specified position (simplified bitmap text rendering)
    fn render_text(
        &self,
        image: &mut RgbaImage,
        text: &str,
        x: f32,
        y: f32,
        font_metrics: &FontMetrics,
        color: Rgba<u8>,
    ) -> Result<(), AppError> {
        // This is a very simplified text rendering implementation
        // In a real application, you'd use a proper font rendering library like rusttype or fontdue
        
        let mut char_x = x;
        let char_y = y + font_metrics.ascent;

        for ch in text.chars() {
            if ch == '\n' {
                continue; // Skip newlines in segments
            }
            
            // Simple character rendering - just draw a filled rectangle for now
            // This is a placeholder that should be replaced with proper font rendering
            self.render_simple_char(image, ch, char_x, char_y, font_metrics, color)?;
            
            char_x += font_metrics.char_width;
        }

        Ok(())
    }

    /// Simplified character rendering (placeholder implementation)
    fn render_simple_char(
        &self,
        image: &mut RgbaImage,
        _ch: char,
        x: f32,
        y: f32,
        font_metrics: &FontMetrics,
        color: Rgba<u8>,
    ) -> Result<(), AppError> {
        // This is a very basic placeholder - just draws a small rectangle
        // In a real implementation, you'd render actual font glyphs
        
        let char_width = font_metrics.char_width as u32;
        let char_height = (font_metrics.ascent + font_metrics.descent) as u32;
        
        let start_x = x as u32;
        let start_y = (y - font_metrics.ascent) as u32;
        
        // Draw a simple filled rectangle as placeholder
        for dy in 0..char_height.min(8) {
            for dx in 0..char_width.min(6) {
                let px = start_x + dx;
                let py = start_y + dy;
                
                if px < image.width() && py < image.height() {
                    // Simple pattern to simulate text
                    if (dx + dy) % 2 == 0 {
                        image.put_pixel(px, py, color);
                    }
                }
            }
        }

        Ok(())
    }

    /// Encodes the image to the requested format
    async fn encode_image(&self, image: RgbaImage, options: &ExportOptions) -> Result<Vec<u8>, AppError> {
        let mut buffer = Vec::new();

        match options.format {
            ImageFormat::PNG => {
                use image::codecs::png::PngEncoder;
                use image::ImageEncoder;
                
                let encoder = PngEncoder::new(&mut buffer);
                encoder
                    .write_image(
                        image.as_raw(),
                        image.width(),
                        image.height(),
                        image::ColorType::Rgba8,
                    )
                    .map_err(|e| AppError::image_generation_failed(format!("PNG encoding failed: {}", e)))?;
            }
            ImageFormat::JPEG => {
                use image::codecs::jpeg::JpegEncoder;
                use image::ImageEncoder;
                
                // Convert RGBA to RGB for JPEG
                let rgb_image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = 
                    image::ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
                        let rgba = image.get_pixel(x, y);
                        image::Rgb([rgba[0], rgba[1], rgba[2]])
                    });
                
                let encoder = JpegEncoder::new_with_quality(&mut buffer, options.quality);
                encoder
                    .write_image(
                        rgb_image.as_raw(),
                        rgb_image.width(),
                        rgb_image.height(),
                        image::ColorType::Rgb8,
                    )
                    .map_err(|e| AppError::image_generation_failed(format!("JPEG encoding failed: {}", e)))?;
            }
            ImageFormat::SVG => {
                // SVG generation would require a different approach
                // For now, return an error indicating it's not implemented
                return Err(AppError::image_generation_failed("SVG export not yet implemented"));
            }
        }

        Ok(buffer)
    }
}

impl Default for ImageGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default ImageGenerator")
    }
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            format: ImageFormat::PNG,
            resolution: Resolution::Standard,
            quality: 90,
            width: None,
            height: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::theme::Theme;

    #[tokio::test]
    async fn test_image_generator_creation() {
        let generator = ImageGenerator::new();
        assert!(generator.is_ok());
    }

    #[tokio::test]
    async fn test_generate_simple_image() {
        let generator = ImageGenerator::new().unwrap();
        let theme = Theme::default_dark();
        let options = ExportOptions::default();
        
        let code = r#"fn main() {
    println!("Hello, world!");
}"#;

        let result = generator.generate_image(code, "Rust", &theme, &options).await;
        assert!(result.is_ok());
        
        let image_data = result.unwrap();
        assert!(!image_data.is_empty());
    }

    #[tokio::test]
    async fn test_generate_with_different_themes() {
        let generator = ImageGenerator::new().unwrap();
        let options = ExportOptions::default();
        
        let code = "console.log('Hello, JavaScript!');";
        
        // Test with dark theme
        let dark_theme = Theme::default_dark();
        let result = generator.generate_image(code, "JavaScript", &dark_theme, &options).await;
        assert!(result.is_ok());
        
        // Test with light theme
        let light_theme = Theme::default_light();
        let result = generator.generate_image(code, "JavaScript", &light_theme, &options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_with_different_formats() {
        let generator = ImageGenerator::new().unwrap();
        let theme = Theme::default_dark();
        let code = "print('Hello, Python!')";
        
        // Test PNG format
        let png_options = ExportOptions {
            format: ImageFormat::PNG,
            ..Default::default()
        };
        let result = generator.generate_image(code, "Python", &theme, &png_options).await;
        assert!(result.is_ok());
        
        // Test JPEG format
        let jpeg_options = ExportOptions {
            format: ImageFormat::JPEG,
            quality: 85,
            ..Default::default()
        };
        let result = generator.generate_image(code, "Python", &theme, &jpeg_options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_with_different_resolutions() {
        let generator = ImageGenerator::new().unwrap();
        let theme = Theme::default_dark();
        let code = "package main\n\nfunc main() {\n    fmt.Println(\"Hello, Go!\")\n}";
        
        for resolution in [Resolution::Standard, Resolution::High, Resolution::Ultra] {
            let options = ExportOptions {
                resolution,
                ..Default::default()
            };
            
            let result = generator.generate_image(code, "Go", &theme, &options).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_empty_code_error() {
        let generator = ImageGenerator::new().unwrap();
        let theme = Theme::default_dark();
        let options = ExportOptions::default();
        
        let result = generator.generate_image("", "Rust", &theme, &options).await;
        assert!(result.is_err());
        
        if let Err(AppError::ImageGenerationError { message }) = result {
            assert!(message.contains("empty"));
        } else {
            panic!("Expected ImageGenerationError");
        }
    }

    #[test]
    fn test_color_parsing() {
        let generator = ImageGenerator::new().unwrap();
        
        // Test valid colors
        assert!(generator.parse_color("#ff0000").is_ok());
        assert!(generator.parse_color("#f00").is_ok());
        assert!(generator.parse_color("#ff0000ff").is_ok());
        
        // Test invalid colors
        assert!(generator.parse_color("ff0000").is_err());
        assert!(generator.parse_color("#gg0000").is_err());
        assert!(generator.parse_color("#ff00").is_err());
    }

    #[test]
    fn test_color_blending() {
        let generator = ImageGenerator::new().unwrap();
        let color1 = Rgba([255, 0, 0, 255]); // Red
        let color2 = Rgba([0, 255, 0, 255]); // Green
        
        let blended = generator.blend_colors(color1, color2, 0.5);
        assert_eq!(blended[0], 127); // Should be halfway between 255 and 0
        assert_eq!(blended[1], 127); // Should be halfway between 0 and 255
        assert_eq!(blended[2], 0);   // Blue should remain 0
    }

    #[test]
    fn test_layout_config_creation() {
        let generator = ImageGenerator::new().unwrap();
        let theme = Theme::default_dark();
        let options = ExportOptions::default();
        
        let layout_config = generator.create_layout_config(&theme, &options);
        assert!(layout_config.is_ok());
        
        let config = layout_config.unwrap();
        assert!(config.font_size > 0.0);
        assert!(config.line_height > 0.0);
        assert!(config.padding.top > 0);
    }

    #[test]
    fn test_font_metrics() {
        let generator = ImageGenerator::new().unwrap();
        
        let metrics = generator.get_font_metrics(14.0);
        assert!(metrics.is_ok());
        
        let font_metrics = metrics.unwrap();
        assert!(font_metrics.char_width > 0.0);
        assert!(font_metrics.line_height > 0.0);
        assert!(font_metrics.ascent > 0.0);
        assert!(font_metrics.descent > 0.0);
    }

    #[tokio::test]
    async fn test_window_frame_rendering() {
        let generator = ImageGenerator::new().unwrap();
        let mut theme = Theme::default_dark();
        theme.window.show_title_bar = true;
        theme.window.show_controls = true;
        theme.window.title = Some("Test Window".to_string());
        
        let options = ExportOptions::default();
        let code = "// Test code with window frame\nfn test() {}";
        
        // Test different window styles
        for style in [WindowStyleType::MacOS, WindowStyleType::Windows, WindowStyleType::Terminal, WindowStyleType::Clean] {
            theme.window.style_type = style.clone();
            let result = generator.generate_image(code, "Rust", &theme, &options).await;
            assert!(result.is_ok(), "Failed to generate image with {:?} window style", style);
        }
    }

    #[tokio::test]
    async fn test_background_types() {
        let generator = ImageGenerator::new().unwrap();
        let mut theme = Theme::default_dark();
        let options = ExportOptions::default();
        let code = "console.log('Testing backgrounds');";
        
        // Test different background types
        for bg_type in [BackgroundType::Solid, BackgroundType::Gradient, BackgroundType::Pattern] {
            theme.background.bg_type = bg_type.clone();
            theme.background.secondary = Some("#444444".to_string());
            
            let result = generator.generate_image(code, "JavaScript", &theme, &options).await;
            assert!(result.is_ok(), "Failed to generate image with {:?} background", bg_type);
        }
    }
}