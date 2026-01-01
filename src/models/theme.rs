use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub background: BackgroundStyle,
    pub syntax: SyntaxColors,
    pub window: WindowStyle,
    pub typography: TypographyStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackgroundStyle {
    pub bg_type: BackgroundType,
    pub primary: String,
    pub secondary: Option<String>,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackgroundType {
    Solid,
    Gradient,
    Pattern,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyntaxColors {
    pub keyword: String,
    pub string: String,
    pub comment: String,
    pub number: String,
    pub operator: String,
    pub function: String,
    pub variable: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowStyle {
    pub style_type: WindowStyleType,
    pub show_title_bar: bool,
    pub title: Option<String>,
    pub show_controls: bool,
    pub border_radius: f32,
    pub shadow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WindowStyleType {
    MacOS,
    Windows,
    Terminal,
    Clean,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypographyStyle {
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub show_line_numbers: bool,
}

impl Theme {
    /// Validates the theme configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate opacity is between 0.0 and 1.0
        if self.background.opacity < 0.0 || self.background.opacity > 1.0 {
            return Err("Background opacity must be between 0.0 and 1.0".to_string());
        }

        // Validate color formats (basic hex color validation)
        let colors = vec![
            &self.background.primary,
            &self.syntax.keyword,
            &self.syntax.string,
            &self.syntax.comment,
            &self.syntax.number,
            &self.syntax.operator,
            &self.syntax.function,
            &self.syntax.variable,
            &self.syntax.type_name,
        ];

        for color in colors {
            if !Self::is_valid_color(color) {
                return Err(format!("Invalid color format: {}", color));
            }
        }

        // Validate secondary color if present
        if let Some(ref secondary) = self.background.secondary {
            if !Self::is_valid_color(secondary) {
                return Err(format!("Invalid secondary color format: {}", secondary));
            }
        }

        // Validate typography values
        if self.typography.font_size <= 0.0 {
            return Err("Font size must be greater than 0".to_string());
        }

        if self.typography.line_height <= 0.0 {
            return Err("Line height must be greater than 0".to_string());
        }

        if self.window.border_radius < 0.0 {
            return Err("Border radius cannot be negative".to_string());
        }

        Ok(())
    }

    /// Basic hex color validation
    fn is_valid_color(color: &str) -> bool {
        if !color.starts_with('#') {
            return false;
        }
        
        let hex_part = &color[1..];
        if hex_part.len() != 3 && hex_part.len() != 6 && hex_part.len() != 8 {
            return false;
        }
        
        hex_part.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Creates a default dark theme
    pub fn default_dark() -> Self {
        Theme {
            id: "default-dark".to_string(),
            name: "Default Dark".to_string(),
            background: BackgroundStyle {
                bg_type: BackgroundType::Solid,
                primary: "#1e1e1e".to_string(),
                secondary: None,
                opacity: 1.0,
            },
            syntax: SyntaxColors {
                keyword: "#569cd6".to_string(),
                string: "#ce9178".to_string(),
                comment: "#6a9955".to_string(),
                number: "#b5cea8".to_string(),
                operator: "#d4d4d4".to_string(),
                function: "#dcdcaa".to_string(),
                variable: "#9cdcfe".to_string(),
                type_name: "#4ec9b0".to_string(),
            },
            window: WindowStyle {
                style_type: WindowStyleType::MacOS,
                show_title_bar: true,
                title: None,
                show_controls: true,
                border_radius: 8.0,
                shadow: true,
            },
            typography: TypographyStyle {
                font_family: "Fira Code".to_string(),
                font_size: 14.0,
                line_height: 1.5,
                letter_spacing: 0.0,
                show_line_numbers: false,
            },
        }
    }

    /// Creates a default light theme
    pub fn default_light() -> Self {
        Theme {
            id: "default-light".to_string(),
            name: "Default Light".to_string(),
            background: BackgroundStyle {
                bg_type: BackgroundType::Solid,
                primary: "#ffffff".to_string(),
                secondary: None,
                opacity: 1.0,
            },
            syntax: SyntaxColors {
                keyword: "#0000ff".to_string(),
                string: "#a31515".to_string(),
                comment: "#008000".to_string(),
                number: "#098658".to_string(),
                operator: "#000000".to_string(),
                function: "#795e26".to_string(),
                variable: "#001080".to_string(),
                type_name: "#267f99".to_string(),
            },
            window: WindowStyle {
                style_type: WindowStyleType::MacOS,
                show_title_bar: true,
                title: None,
                show_controls: true,
                border_radius: 8.0,
                shadow: true,
            },
            typography: TypographyStyle {
                font_family: "Fira Code".to_string(),
                font_size: 14.0,
                line_height: 1.5,
                letter_spacing: 0.0,
                show_line_numbers: false,
            },
        }
    }
}