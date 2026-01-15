use crate::models::theme::{Theme, BackgroundStyle, BackgroundType, SyntaxColors, WindowStyle, WindowStyleType, TypographyStyle};
use crate::models::errors::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Theme management service for handling visual themes
pub struct ThemeManager {
    themes: Arc<RwLock<HashMap<String, Theme>>>,
}

impl ThemeManager {
    /// Creates a new ThemeManager with built-in themes
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        
        // Add built-in themes
        let builtin_themes = Self::create_builtin_themes();
        for theme in builtin_themes {
            themes.insert(theme.id.clone(), theme);
        }
        
        Self {
            themes: Arc::new(RwLock::new(themes)),
        }
    }
    
    /// Creates all built-in theme definitions
    fn create_builtin_themes() -> Vec<Theme> {
        vec![
            Self::create_dark_theme(),
            Self::create_light_theme(),
            Self::create_high_contrast_theme(),
            Self::create_vscode_dark_theme(),
            Self::create_monokai_theme(),
            Self::create_github_light_theme(),
            Self::create_dracula_theme(),
            Self::create_solarized_dark_theme(),
            Self::create_solarized_light_theme(),
        ]
    }
    
    /// Default dark theme
    fn create_dark_theme() -> Theme {
        Theme::default_dark()
    }
    
    /// Default light theme
    fn create_light_theme() -> Theme {
        Theme::default_light()
    }
    
    /// High contrast theme for accessibility
    fn create_high_contrast_theme() -> Theme {
        Theme {
            id: "high-contrast".to_string(),
            name: "High Contrast".to_string(),
            background: BackgroundStyle {
                bg_type: BackgroundType::Solid,
                primary: "#000000".to_string(),
                secondary: None,
                opacity: 1.0,
            },
            syntax: SyntaxColors {
                keyword: "#00ffff".to_string(),
                string: "#ffff00".to_string(),
                comment: "#00ff00".to_string(),
                number: "#ff00ff".to_string(),
                operator: "#ffffff".to_string(),
                function: "#ffaa00".to_string(),
                variable: "#00aaff".to_string(),
                type_name: "#ff6600".to_string(),
            },
            window: WindowStyle {
                style_type: WindowStyleType::Clean,
                show_title_bar: false,
                title: None,
                show_controls: false,
                border_radius: 0.0,
                shadow: false,
            },
            typography: TypographyStyle {
                font_family: "Monaco".to_string(),
                font_size: 16.0,
                line_height: 1.6,
                letter_spacing: 0.5,
                show_line_numbers: true,
            },
        }
    }
    
    /// VS Code Dark theme
    fn create_vscode_dark_theme() -> Theme {
        Theme {
            id: "vscode-dark".to_string(),
            name: "VS Code Dark".to_string(),
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
                title: Some("Visual Studio Code".to_string()),
                show_controls: true,
                border_radius: 6.0,
                shadow: true,
            },
            typography: TypographyStyle {
                font_family: "Consolas".to_string(),
                font_size: 14.0,
                line_height: 1.5,
                letter_spacing: 0.0,
                show_line_numbers: true,
            },
        }
    }
    
    /// Monokai theme
    fn create_monokai_theme() -> Theme {
        Theme {
            id: "monokai".to_string(),
            name: "Monokai".to_string(),
            background: BackgroundStyle {
                bg_type: BackgroundType::Solid,
                primary: "#272822".to_string(),
                secondary: None,
                opacity: 1.0,
            },
            syntax: SyntaxColors {
                keyword: "#f92672".to_string(),
                string: "#e6db74".to_string(),
                comment: "#75715e".to_string(),
                number: "#ae81ff".to_string(),
                operator: "#f8f8f2".to_string(),
                function: "#a6e22e".to_string(),
                variable: "#f8f8f2".to_string(),
                type_name: "#66d9ef".to_string(),
            },
            window: WindowStyle {
                style_type: WindowStyleType::Terminal,
                show_title_bar: true,
                title: Some("Monokai".to_string()),
                show_controls: true,
                border_radius: 4.0,
                shadow: true,
            },
            typography: TypographyStyle {
                font_family: "Monaco".to_string(),
                font_size: 13.0,
                line_height: 1.4,
                letter_spacing: 0.0,
                show_line_numbers: false,
            },
        }
    }
    
    /// GitHub Light theme
    fn create_github_light_theme() -> Theme {
        Theme {
            id: "github-light".to_string(),
            name: "GitHub Light".to_string(),
            background: BackgroundStyle {
                bg_type: BackgroundType::Solid,
                primary: "#ffffff".to_string(),
                secondary: None,
                opacity: 1.0,
            },
            syntax: SyntaxColors {
                keyword: "#d73a49".to_string(),
                string: "#032f62".to_string(),
                comment: "#6a737d".to_string(),
                number: "#005cc5".to_string(),
                operator: "#24292e".to_string(),
                function: "#6f42c1".to_string(),
                variable: "#e36209".to_string(),
                type_name: "#005cc5".to_string(),
            },
            window: WindowStyle {
                style_type: WindowStyleType::Clean,
                show_title_bar: true,
                title: Some("GitHub".to_string()),
                show_controls: false,
                border_radius: 6.0,
                shadow: false,
            },
            typography: TypographyStyle {
                font_family: "SFMono-Regular".to_string(),
                font_size: 14.0,
                line_height: 1.45,
                letter_spacing: 0.0,
                show_line_numbers: true,
            },
        }
    }
    
    /// Dracula theme
    fn create_dracula_theme() -> Theme {
        Theme {
            id: "dracula".to_string(),
            name: "Dracula".to_string(),
            background: BackgroundStyle {
                bg_type: BackgroundType::Gradient,
                primary: "#282a36".to_string(),
                secondary: Some("#44475a".to_string()),
                opacity: 1.0,
            },
            syntax: SyntaxColors {
                keyword: "#ff79c6".to_string(),
                string: "#f1fa8c".to_string(),
                comment: "#6272a4".to_string(),
                number: "#bd93f9".to_string(),
                operator: "#f8f8f2".to_string(),
                function: "#50fa7b".to_string(),
                variable: "#f8f8f2".to_string(),
                type_name: "#8be9fd".to_string(),
            },
            window: WindowStyle {
                style_type: WindowStyleType::MacOS,
                show_title_bar: true,
                title: Some("Dracula".to_string()),
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
    
    /// Solarized Dark theme
    fn create_solarized_dark_theme() -> Theme {
        Theme {
            id: "solarized-dark".to_string(),
            name: "Solarized Dark".to_string(),
            background: BackgroundStyle {
                bg_type: BackgroundType::Solid,
                primary: "#002b36".to_string(),
                secondary: None,
                opacity: 1.0,
            },
            syntax: SyntaxColors {
                keyword: "#859900".to_string(),
                string: "#2aa198".to_string(),
                comment: "#586e75".to_string(),
                number: "#d33682".to_string(),
                operator: "#839496".to_string(),
                function: "#268bd2".to_string(),
                variable: "#b58900".to_string(),
                type_name: "#cb4b16".to_string(),
            },
            window: WindowStyle {
                style_type: WindowStyleType::Terminal,
                show_title_bar: true,
                title: Some("Solarized".to_string()),
                show_controls: true,
                border_radius: 4.0,
                shadow: false,
            },
            typography: TypographyStyle {
                font_family: "Source Code Pro".to_string(),
                font_size: 13.0,
                line_height: 1.4,
                letter_spacing: 0.0,
                show_line_numbers: true,
            },
        }
    }
    
    /// Solarized Light theme
    fn create_solarized_light_theme() -> Theme {
        Theme {
            id: "solarized-light".to_string(),
            name: "Solarized Light".to_string(),
            background: BackgroundStyle {
                bg_type: BackgroundType::Solid,
                primary: "#fdf6e3".to_string(),
                secondary: None,
                opacity: 1.0,
            },
            syntax: SyntaxColors {
                keyword: "#859900".to_string(),
                string: "#2aa198".to_string(),
                comment: "#93a1a1".to_string(),
                number: "#d33682".to_string(),
                operator: "#657b83".to_string(),
                function: "#268bd2".to_string(),
                variable: "#b58900".to_string(),
                type_name: "#cb4b16".to_string(),
            },
            window: WindowStyle {
                style_type: WindowStyleType::Clean,
                show_title_bar: false,
                title: None,
                show_controls: false,
                border_radius: 6.0,
                shadow: true,
            },
            typography: TypographyStyle {
                font_family: "Source Code Pro".to_string(),
                font_size: 13.0,
                line_height: 1.4,
                letter_spacing: 0.0,
                show_line_numbers: false,
            },
        }
    }    

    /// Retrieves a theme by ID
    pub async fn get_theme(&self, id: &str) -> Option<Theme> {
        let themes = self.themes.read().await;
        themes.get(id).cloned()
    }
    
    /// Lists all available themes
    pub async fn list_themes(&self) -> Vec<Theme> {
        let themes = self.themes.read().await;
        themes.values().cloned().collect()
    }
    
    /// Lists theme IDs and names for quick reference
    pub async fn list_theme_info(&self) -> Vec<(String, String)> {
        let themes = self.themes.read().await;
        themes.values()
            .map(|theme| (theme.id.clone(), theme.name.clone()))
            .collect()
    }
    
    /// Adds a custom theme to the manager
    pub async fn add_theme(&self, theme: Theme) -> Result<(), AppError> {
        // Validate the theme before adding
        theme.validate().map_err(|e| AppError::theme_error(e))?;
        
        let mut themes = self.themes.write().await;
        themes.insert(theme.id.clone(), theme);
        Ok(())
    }
    
    /// Updates an existing theme
    pub async fn update_theme(&self, theme: Theme) -> Result<(), AppError> {
        // Validate the theme before updating
        theme.validate().map_err(|e| AppError::theme_error(e))?;
        
        let mut themes = self.themes.write().await;
        if themes.contains_key(&theme.id) {
            themes.insert(theme.id.clone(), theme);
            Ok(())
        } else {
            Err(AppError::theme_not_found(theme.id))
        }
    }
    
    /// Removes a theme (only custom themes, not built-in ones)
    pub async fn remove_theme(&self, id: &str) -> Result<(), AppError> {
        // Prevent removal of built-in themes
        let builtin_ids = vec![
            "default-dark", "default-light", "high-contrast", "vscode-dark",
            "monokai", "github-light", "dracula", "solarized-dark", "solarized-light"
        ];
        
        if builtin_ids.contains(&id) {
            return Err(AppError::theme_error("Cannot remove built-in themes"));
        }
        
        let mut themes = self.themes.write().await;
        if themes.remove(id).is_some() {
            Ok(())
        } else {
            Err(AppError::theme_not_found(id))
        }
    }
    
    /// Validates a theme configuration
    pub fn validate_theme(&self, theme: &Theme) -> Result<(), AppError> {
        theme.validate().map_err(|e| AppError::theme_error(e))
    }
    
    /// Gets the default theme (dark theme)
    pub async fn get_default_theme(&self) -> Theme {
        self.get_theme("default-dark").await
            .unwrap_or_else(|| Theme::default_dark())
    }
    
    /// Checks if a theme exists
    pub async fn theme_exists(&self, id: &str) -> bool {
        let themes = self.themes.read().await;
        themes.contains_key(id)
    }
    
    /// Gets themes by category/type
    pub async fn get_themes_by_type(&self, theme_type: ThemeType) -> Vec<Theme> {
        let themes = self.themes.read().await;
        themes.values()
            .filter(|theme| self.categorize_theme(theme) == theme_type)
            .cloned()
            .collect()
    }
    
    /// Categorizes a theme based on its characteristics
    fn categorize_theme(&self, theme: &Theme) -> ThemeType {
        match theme.id.as_str() {
            "default-dark" | "vscode-dark" | "monokai" | "dracula" | "solarized-dark" => ThemeType::Dark,
            "default-light" | "github-light" | "solarized-light" => ThemeType::Light,
            "high-contrast" => ThemeType::HighContrast,
            _ => {
                // Determine by background color brightness
                if self.is_dark_color(&theme.background.primary) {
                    ThemeType::Dark
                } else {
                    ThemeType::Light
                }
            }
        }
    }
    
    /// Determines if a color is dark based on its hex value
    fn is_dark_color(&self, hex_color: &str) -> bool {
        if let Ok(rgb) = self.hex_to_rgb(hex_color) {
            // Calculate luminance using relative luminance formula
            let luminance = 0.299 * rgb.0 as f32 + 0.587 * rgb.1 as f32 + 0.114 * rgb.2 as f32;
            luminance < 128.0
        } else {
            true // Default to dark if parsing fails
        }
    }
    
    /// Converts hex color to RGB tuple
    fn hex_to_rgb(&self, hex: &str) -> Result<(u8, u8, u8), AppError> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(AppError::theme_error("Invalid hex color format"));
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| AppError::theme_error("Invalid red component"))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| AppError::theme_error("Invalid green component"))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| AppError::theme_error("Invalid blue component"))?;
        
        Ok((r, g, b))
    }
    
    /// Creates a customized theme based on an existing theme
    pub async fn customize_theme(
        &self,
        base_theme_id: &str,
        customizations: ThemeCustomization,
    ) -> Result<Theme, AppError> {
        let base_theme = self.get_theme(base_theme_id).await
            .ok_or_else(|| AppError::theme_not_found(base_theme_id))?;
        
        let mut customized_theme = base_theme.clone();
        
        // Apply customizations
        if let Some(id) = customizations.id {
            customized_theme.id = id;
        }
        
        if let Some(name) = customizations.name {
            customized_theme.name = name;
        }
        
        if let Some(background) = customizations.background {
            self.apply_background_customization(&mut customized_theme.background, background)?;
        }
        
        if let Some(syntax) = customizations.syntax {
            self.apply_syntax_customization(&mut customized_theme.syntax, syntax)?;
        }
        
        if let Some(window) = customizations.window {
            self.apply_window_customization(&mut customized_theme.window, window)?;
        }
        
        if let Some(typography) = customizations.typography {
            self.apply_typography_customization(&mut customized_theme.typography, typography)?;
        }
        
        // Validate the customized theme
        customized_theme.validate().map_err(|e| AppError::theme_error(e))?;
        
        Ok(customized_theme)
    }
    
    /// Applies background style customizations
    fn apply_background_customization(
        &self,
        background: &mut BackgroundStyle,
        customization: BackgroundCustomization,
    ) -> Result<(), AppError> {
        if let Some(bg_type) = customization.bg_type {
            background.bg_type = bg_type;
        }
        
        if let Some(primary) = customization.primary {
            if !Theme::is_valid_color(&primary) {
                return Err(AppError::theme_error(format!("Invalid primary color: {}", primary)));
            }
            background.primary = primary;
        }
        
        if let Some(secondary) = customization.secondary {
            if !Theme::is_valid_color(&secondary) {
                return Err(AppError::theme_error(format!("Invalid secondary color: {}", secondary)));
            }
            background.secondary = Some(secondary);
        }
        
        if let Some(opacity) = customization.opacity {
            if opacity < 0.0 || opacity > 1.0 {
                return Err(AppError::theme_error("Opacity must be between 0.0 and 1.0".to_string()));
            }
            background.opacity = opacity;
        }
        
        Ok(())
    }
    
    /// Applies syntax color customizations
    fn apply_syntax_customization(
        &self,
        syntax: &mut SyntaxColors,
        customization: SyntaxCustomization,
    ) -> Result<(), AppError> {
        if let Some(keyword) = customization.keyword {
            if !Theme::is_valid_color(&keyword) {
                return Err(AppError::theme_error(format!("Invalid keyword color: {}", keyword)));
            }
            syntax.keyword = keyword;
        }
        
        if let Some(string) = customization.string {
            if !Theme::is_valid_color(&string) {
                return Err(AppError::theme_error(format!("Invalid string color: {}", string)));
            }
            syntax.string = string;
        }
        
        if let Some(comment) = customization.comment {
            if !Theme::is_valid_color(&comment) {
                return Err(AppError::theme_error(format!("Invalid comment color: {}", comment)));
            }
            syntax.comment = comment;
        }
        
        if let Some(number) = customization.number {
            if !Theme::is_valid_color(&number) {
                return Err(AppError::theme_error(format!("Invalid number color: {}", number)));
            }
            syntax.number = number;
        }
        
        if let Some(operator) = customization.operator {
            if !Theme::is_valid_color(&operator) {
                return Err(AppError::theme_error(format!("Invalid operator color: {}", operator)));
            }
            syntax.operator = operator;
        }
        
        if let Some(function) = customization.function {
            if !Theme::is_valid_color(&function) {
                return Err(AppError::theme_error(format!("Invalid function color: {}", function)));
            }
            syntax.function = function;
        }
        
        if let Some(variable) = customization.variable {
            if !Theme::is_valid_color(&variable) {
                return Err(AppError::theme_error(format!("Invalid variable color: {}", variable)));
            }
            syntax.variable = variable;
        }
        
        if let Some(type_name) = customization.type_name {
            if !Theme::is_valid_color(&type_name) {
                return Err(AppError::theme_error(format!("Invalid type name color: {}", type_name)));
            }
            syntax.type_name = type_name;
        }
        
        Ok(())
    }
    
    /// Applies window style customizations
    fn apply_window_customization(
        &self,
        window: &mut WindowStyle,
        customization: WindowCustomization,
    ) -> Result<(), AppError> {
        if let Some(style_type) = customization.style_type {
            window.style_type = style_type;
        }
        
        if let Some(show_title_bar) = customization.show_title_bar {
            window.show_title_bar = show_title_bar;
        }
        
        if let Some(title) = customization.title {
            window.title = Some(title);
        }
        
        if let Some(show_controls) = customization.show_controls {
            window.show_controls = show_controls;
        }
        
        if let Some(border_radius) = customization.border_radius {
            if border_radius < 0.0 {
                return Err(AppError::theme_error("Border radius cannot be negative".to_string()));
            }
            window.border_radius = border_radius;
        }
        
        if let Some(shadow) = customization.shadow {
            window.shadow = shadow;
        }
        
        Ok(())
    }
    
    /// Applies typography customizations
    fn apply_typography_customization(
        &self,
        typography: &mut TypographyStyle,
        customization: TypographyCustomization,
    ) -> Result<(), AppError> {
        if let Some(font_family) = customization.font_family {
            typography.font_family = font_family;
        }
        
        if let Some(font_size) = customization.font_size {
            if font_size <= 0.0 {
                return Err(AppError::theme_error("Font size must be greater than 0".to_string()));
            }
            typography.font_size = font_size;
        }
        
        if let Some(line_height) = customization.line_height {
            if line_height <= 0.0 {
                return Err(AppError::theme_error("Line height must be greater than 0".to_string()));
            }
            typography.line_height = line_height;
        }
        
        if let Some(letter_spacing) = customization.letter_spacing {
            typography.letter_spacing = letter_spacing;
        }
        
        if let Some(show_line_numbers) = customization.show_line_numbers {
            typography.show_line_numbers = show_line_numbers;
        }
        
        Ok(())
    }
    
    /// Creates a theme preset with common customizations
    pub async fn create_preset_theme(&self, preset: ThemePreset) -> Result<Theme, AppError> {
        match preset {
            ThemePreset::HighContrastDark => {
                let customization = ThemeCustomization {
                    id: Some("high-contrast-dark-custom".to_string()),
                    name: Some("High Contrast Dark Custom".to_string()),
                    background: Some(BackgroundCustomization {
                        bg_type: Some(BackgroundType::Solid),
                        primary: Some("#000000".to_string()),
                        secondary: None,
                        opacity: Some(1.0),
                    }),
                    syntax: Some(SyntaxCustomization {
                        keyword: Some("#00ffff".to_string()),
                        string: Some("#ffff00".to_string()),
                        comment: Some("#00ff00".to_string()),
                        number: Some("#ff00ff".to_string()),
                        operator: Some("#ffffff".to_string()),
                        function: Some("#ffaa00".to_string()),
                        variable: Some("#00aaff".to_string()),
                        type_name: Some("#ff6600".to_string()),
                    }),
                    window: Some(WindowCustomization {
                        style_type: Some(WindowStyleType::Clean),
                        show_title_bar: Some(false),
                        title: None,
                        show_controls: Some(false),
                        border_radius: Some(0.0),
                        shadow: Some(false),
                    }),
                    typography: Some(TypographyCustomization {
                        font_family: Some("Monaco".to_string()),
                        font_size: Some(16.0),
                        line_height: Some(1.6),
                        letter_spacing: Some(0.5),
                        show_line_numbers: Some(true),
                    }),
                };
                self.customize_theme("default-dark", customization).await
            },
            ThemePreset::MinimalLight => {
                let customization = ThemeCustomization {
                    id: Some("minimal-light".to_string()),
                    name: Some("Minimal Light".to_string()),
                    background: Some(BackgroundCustomization {
                        bg_type: Some(BackgroundType::Solid),
                        primary: Some("#fafafa".to_string()),
                        secondary: None,
                        opacity: Some(1.0),
                    }),
                    syntax: Some(SyntaxCustomization {
                        keyword: Some("#0066cc".to_string()),
                        string: Some("#008000".to_string()),
                        comment: Some("#999999".to_string()),
                        number: Some("#cc6600".to_string()),
                        operator: Some("#333333".to_string()),
                        function: Some("#6600cc".to_string()),
                        variable: Some("#000080".to_string()),
                        type_name: Some("#cc0066".to_string()),
                    }),
                    window: Some(WindowCustomization {
                        style_type: Some(WindowStyleType::Clean),
                        show_title_bar: Some(false),
                        title: None,
                        show_controls: Some(false),
                        border_radius: Some(4.0),
                        shadow: Some(false),
                    }),
                    typography: Some(TypographyCustomization {
                        font_family: Some("SF Mono".to_string()),
                        font_size: Some(13.0),
                        line_height: Some(1.4),
                        letter_spacing: Some(0.0),
                        show_line_numbers: Some(false),
                    }),
                };
                self.customize_theme("default-light", customization).await
            },
            ThemePreset::NeonDark => {
                let customization = ThemeCustomization {
                    id: Some("neon-dark".to_string()),
                    name: Some("Neon Dark".to_string()),
                    background: Some(BackgroundCustomization {
                        bg_type: Some(BackgroundType::Gradient),
                        primary: Some("#0a0a0a".to_string()),
                        secondary: Some("#1a1a2e".to_string()),
                        opacity: Some(1.0),
                    }),
                    syntax: Some(SyntaxCustomization {
                        keyword: Some("#ff0080".to_string()),
                        string: Some("#00ff80".to_string()),
                        comment: Some("#8080ff".to_string()),
                        number: Some("#ffff00".to_string()),
                        operator: Some("#ff8000".to_string()),
                        function: Some("#80ff00".to_string()),
                        variable: Some("#00ffff".to_string()),
                        type_name: Some("#ff4080".to_string()),
                    }),
                    window: Some(WindowCustomization {
                        style_type: Some(WindowStyleType::Terminal),
                        show_title_bar: Some(true),
                        title: Some("Neon Terminal".to_string()),
                        show_controls: Some(true),
                        border_radius: Some(6.0),
                        shadow: Some(true),
                    }),
                    typography: Some(TypographyCustomization {
                        font_family: Some("JetBrains Mono".to_string()),
                        font_size: Some(14.0),
                        line_height: Some(1.5),
                        letter_spacing: Some(0.2),
                        show_line_numbers: Some(true),
                    }),
                };
                self.customize_theme("default-dark", customization).await
            },
        }
    }
    
    /// Gets available font families for typography customization
    pub fn get_available_fonts(&self) -> Vec<String> {
        vec![
            "Fira Code".to_string(),
            "JetBrains Mono".to_string(),
            "Source Code Pro".to_string(),
            "Monaco".to_string(),
            "Consolas".to_string(),
            "SF Mono".to_string(),
            "SFMono-Regular".to_string(),
            "Menlo".to_string(),
            "Courier New".to_string(),
            "monospace".to_string(),
        ]
    }
    
    /// Gets available window style types
    pub fn get_available_window_styles(&self) -> Vec<WindowStyleType> {
        vec![
            WindowStyleType::MacOS,
            WindowStyleType::Windows,
            WindowStyleType::Terminal,
            WindowStyleType::Clean,
        ]
    }
    
    /// Gets available background types
    pub fn get_available_background_types(&self) -> Vec<BackgroundType> {
        vec![
            BackgroundType::Solid,
            BackgroundType::Gradient,
            BackgroundType::Pattern,
        ]
    }
    
    /// Validates a color string and suggests corrections if invalid
    pub fn validate_and_suggest_color(&self, color: &str) -> ColorValidationResult {
        if Theme::is_valid_color(color) {
            ColorValidationResult {
                is_valid: true,
                suggestions: vec![],
                error_message: None,
            }
        } else {
            let mut suggestions = vec![];
            let mut error_message = "Invalid color format".to_string();
            
            // Try to fix common issues
            if !color.starts_with('#') && color.len() == 6 && color.chars().all(|c| c.is_ascii_hexdigit()) {
                suggestions.push(format!("#{}", color));
                error_message = "Color should start with #".to_string();
            } else if color.starts_with('#') && color.len() == 4 {
                // Convert 3-digit hex to 6-digit
                let short = &color[1..];
                if short.chars().all(|c| c.is_ascii_hexdigit()) {
                    let expanded = format!("#{}{}{}{}{}{}", 
                        short.chars().nth(0).unwrap(), short.chars().nth(0).unwrap(),
                        short.chars().nth(1).unwrap(), short.chars().nth(1).unwrap(),
                        short.chars().nth(2).unwrap(), short.chars().nth(2).unwrap()
                    );
                    suggestions.push(expanded);
                    error_message = "3-digit hex colors should be expanded to 6 digits".to_string();
                }
            } else if color.starts_with('#') && (color.len() != 7 && color.len() != 4) {
                error_message = "Hex colors should be #RGB or #RRGGBB format".to_string();
            } else if !color.starts_with('#') {
                error_message = "Color should be in hex format (#RRGGBB)".to_string();
            }
            
            ColorValidationResult {
                is_valid: false,
                suggestions,
                error_message: Some(error_message),
            }
        }
    }
}

/// Theme categorization for filtering and organization
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeType {
    Dark,
    Light,
    HighContrast,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Customization options for creating modified themes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThemeCustomization {
    pub id: Option<String>,
    pub name: Option<String>,
    pub background: Option<BackgroundCustomization>,
    pub syntax: Option<SyntaxCustomization>,
    pub window: Option<WindowCustomization>,
    pub typography: Option<TypographyCustomization>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackgroundCustomization {
    pub bg_type: Option<BackgroundType>,
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub opacity: Option<f32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyntaxCustomization {
    pub keyword: Option<String>,
    pub string: Option<String>,
    pub comment: Option<String>,
    pub number: Option<String>,
    pub operator: Option<String>,
    pub function: Option<String>,
    pub variable: Option<String>,
    pub type_name: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowCustomization {
    pub style_type: Option<WindowStyleType>,
    pub show_title_bar: Option<bool>,
    pub title: Option<String>,
    pub show_controls: Option<bool>,
    pub border_radius: Option<f32>,
    pub shadow: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypographyCustomization {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub letter_spacing: Option<f32>,
    pub show_line_numbers: Option<bool>,
}

/// Predefined theme presets for quick customization
#[derive(Debug, Clone, PartialEq)]
pub enum ThemePreset {
    HighContrastDark,
    MinimalLight,
    NeonDark,
}

/// Result of color validation with suggestions
#[derive(Debug, Clone)]
pub struct ColorValidationResult {
    pub is_valid: bool,
    pub suggestions: Vec<String>,
    pub error_message: Option<String>,
}

#[cfg(test)]
mod customization_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_theme_customization() {
        let manager = ThemeManager::new();
        
        let customization = ThemeCustomization {
            id: Some("custom-dark".to_string()),
            name: Some("Custom Dark Theme".to_string()),
            background: Some(BackgroundCustomization {
                bg_type: Some(BackgroundType::Gradient),
                primary: Some("#1a1a1a".to_string()),
                secondary: Some("#2a2a2a".to_string()),
                opacity: Some(0.9),
            }),
            syntax: Some(SyntaxCustomization {
                keyword: Some("#ff6b6b".to_string()),
                string: Some("#4ecdc4".to_string()),
                comment: Some("#95a5a6".to_string()),
                number: Some("#f39c12".to_string()),
                operator: Some("#ecf0f1".to_string()),
                function: Some("#9b59b6".to_string()),
                variable: Some("#3498db".to_string()),
                type_name: Some("#e74c3c".to_string()),
            }),
            window: None,
            typography: Some(TypographyCustomization {
                font_family: Some("JetBrains Mono".to_string()),
                font_size: Some(15.0),
                line_height: Some(1.6),
                letter_spacing: Some(0.1),
                show_line_numbers: Some(true),
            }),
        };
        
        let result = manager.customize_theme("default-dark", customization).await;
        assert!(result.is_ok());
        
        let custom_theme = result.unwrap();
        assert_eq!(custom_theme.id, "custom-dark");
        assert_eq!(custom_theme.name, "Custom Dark Theme");
        assert_eq!(custom_theme.background.bg_type, BackgroundType::Gradient);
        assert_eq!(custom_theme.background.primary, "#1a1a1a");
        assert_eq!(custom_theme.syntax.keyword, "#ff6b6b");
        assert_eq!(custom_theme.typography.font_family, "JetBrains Mono");
    }
    
    #[tokio::test]
    async fn test_preset_themes() {
        let manager = ThemeManager::new();
        
        let high_contrast = manager.create_preset_theme(ThemePreset::HighContrastDark).await;
        assert!(high_contrast.is_ok());
        
        let minimal_light = manager.create_preset_theme(ThemePreset::MinimalLight).await;
        assert!(minimal_light.is_ok());
        
        let neon_dark = manager.create_preset_theme(ThemePreset::NeonDark).await;
        assert!(neon_dark.is_ok());
        
        let theme = high_contrast.unwrap();
        assert_eq!(theme.id, "high-contrast-dark-custom");
        assert_eq!(theme.background.primary, "#000000");
    }
    
    #[tokio::test]
    async fn test_invalid_customization() {
        let manager = ThemeManager::new();
        
        let invalid_customization = ThemeCustomization {
            id: Some("invalid-theme".to_string()),
            name: Some("Invalid Theme".to_string()),
            background: Some(BackgroundCustomization {
                bg_type: Some(BackgroundType::Solid),
                primary: Some("invalid-color".to_string()), // Invalid color
                secondary: None,
                opacity: Some(1.0),
            }),
            syntax: None,
            window: None,
            typography: None,
        };
        
        let result = manager.customize_theme("default-dark", invalid_customization).await;
        assert!(result.is_err());
    }
    
    #[test]
    fn test_color_validation() {
        let manager = ThemeManager::new();
        
        // Valid colors
        let valid_result = manager.validate_and_suggest_color("#ff0000");
        assert!(valid_result.is_valid);
        assert!(valid_result.suggestions.is_empty());
        
        // Invalid color without #
        let invalid_result = manager.validate_and_suggest_color("ff0000");
        assert!(!invalid_result.is_valid);
        assert!(invalid_result.suggestions.contains(&"#ff0000".to_string()));
        
        // 3-digit hex (this is actually valid)
        let short_result = manager.validate_and_suggest_color("#f00");
        assert!(short_result.is_valid); // 3-digit hex is valid
    }
    
    #[test]
    fn test_available_options() {
        let manager = ThemeManager::new();
        
        let fonts = manager.get_available_fonts();
        assert!(!fonts.is_empty());
        assert!(fonts.contains(&"Fira Code".to_string()));
        
        let window_styles = manager.get_available_window_styles();
        assert_eq!(window_styles.len(), 4);
        assert!(window_styles.contains(&WindowStyleType::MacOS));
        
        let bg_types = manager.get_available_background_types();
        assert_eq!(bg_types.len(), 3);
        assert!(bg_types.contains(&BackgroundType::Solid));
    }
}