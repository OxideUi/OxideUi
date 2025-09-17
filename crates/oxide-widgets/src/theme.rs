//! Theming system for OxideUI

use oxide_core::types::{Color, Shadow};
// Removed unused oxide_core::layout::EdgeInsets import
use std::collections::HashMap;

/// Theme configuration
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ColorPalette,
    pub typography: Typography,
    pub spacing: SpacingScale,
    pub borders: BorderStyles,
    pub shadows: ShadowStyles,
    pub components: ComponentThemes,
}

impl Theme {
    /// Create a light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            colors: ColorPalette::light(),
            typography: Typography::default(),
            spacing: SpacingScale::default(),
            borders: BorderStyles::default(),
            shadows: ShadowStyles::default(),
            components: ComponentThemes::light(),
        }
    }

    /// Create a dark theme
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            colors: ColorPalette::dark(),
            typography: Typography::default(),
            spacing: SpacingScale::default(),
            borders: BorderStyles::default(),
            shadows: ShadowStyles::default(),
            components: ComponentThemes::dark(),
        }
    }

    /// Create a custom theme
    pub fn custom(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            colors: ColorPalette::default(),
            typography: Typography::default(),
            spacing: SpacingScale::default(),
            borders: BorderStyles::default(),
            shadows: ShadowStyles::default(),
            components: ComponentThemes::default(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

/// Color palette
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub background: Color,
    pub surface: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub divider: Color,
}

impl ColorPalette {
    /// Light color palette
    pub fn light() -> Self {
        Self {
            primary: Color::rgb(0.129, 0.588, 0.953),
            secondary: Color::rgb(0.0, 0.737, 0.831),
            success: Color::rgb(0.298, 0.686, 0.314),
            warning: Color::rgb(1.0, 0.757, 0.027),
            error: Color::rgb(0.956, 0.263, 0.212),
            info: Color::rgb(0.012, 0.663, 0.957),
            background: Color::rgb(0.98, 0.98, 0.98),
            surface: Color::WHITE,
            text_primary: Color::rgba(0.0, 0.0, 0.0, 0.87),
            text_secondary: Color::rgba(0.0, 0.0, 0.0, 0.60),
            text_disabled: Color::rgba(0.0, 0.0, 0.0, 0.38),
            divider: Color::rgba(0.0, 0.0, 0.0, 0.12),
        }
    }

    /// Dark color palette
    pub fn dark() -> Self {
        Self {
            primary: Color::rgb(0.353, 0.706, 1.0),
            secondary: Color::rgb(0.0, 0.878, 0.922),
            success: Color::rgb(0.463, 0.733, 0.463),
            warning: Color::rgb(1.0, 0.835, 0.306),
            error: Color::rgb(0.961, 0.498, 0.478),
            info: Color::rgb(0.294, 0.745, 0.969),
            background: Color::rgb(0.071, 0.071, 0.071),
            surface: Color::rgb(0.118, 0.118, 0.118),
            text_primary: Color::rgba(1.0, 1.0, 1.0, 0.87),
            text_secondary: Color::rgba(1.0, 1.0, 1.0, 0.60),
            text_disabled: Color::rgba(1.0, 1.0, 1.0, 0.38),
            divider: Color::rgba(1.0, 1.0, 1.0, 0.12),
        }
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::light()
    }
}

/// Typography configuration
#[derive(Debug, Clone)]
pub struct Typography {
    pub font_family: String,
    pub font_family_mono: String,
    pub h1: TextStyle,
    pub h2: TextStyle,
    pub h3: TextStyle,
    pub h4: TextStyle,
    pub h5: TextStyle,
    pub h6: TextStyle,
    pub body1: TextStyle,
    pub body2: TextStyle,
    pub subtitle1: TextStyle,
    pub subtitle2: TextStyle,
    pub button: TextStyle,
    pub caption: TextStyle,
    pub overline: TextStyle,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_family: "system-ui, -apple-system, sans-serif".to_string(),
            font_family_mono: "Consolas, Monaco, monospace".to_string(),
            h1: TextStyle { size: 96.0, weight: 300, letter_spacing: -1.5 },
            h2: TextStyle { size: 60.0, weight: 300, letter_spacing: -0.5 },
            h3: TextStyle { size: 48.0, weight: 400, letter_spacing: 0.0 },
            h4: TextStyle { size: 34.0, weight: 400, letter_spacing: 0.25 },
            h5: TextStyle { size: 24.0, weight: 400, letter_spacing: 0.0 },
            h6: TextStyle { size: 20.0, weight: 500, letter_spacing: 0.15 },
            body1: TextStyle { size: 16.0, weight: 400, letter_spacing: 0.5 },
            body2: TextStyle { size: 14.0, weight: 400, letter_spacing: 0.25 },
            subtitle1: TextStyle { size: 16.0, weight: 400, letter_spacing: 0.15 },
            subtitle2: TextStyle { size: 14.0, weight: 500, letter_spacing: 0.1 },
            button: TextStyle { size: 14.0, weight: 500, letter_spacing: 1.25 },
            caption: TextStyle { size: 12.0, weight: 400, letter_spacing: 0.4 },
            overline: TextStyle { size: 10.0, weight: 400, letter_spacing: 1.5 },
        }
    }
}

/// Text style
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub size: f32,
    pub weight: u16,
    pub letter_spacing: f32,
}

/// Spacing scale
#[derive(Debug, Clone)]
pub struct SpacingScale {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl Default for SpacingScale {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
        }
    }
}

/// Border styles
#[derive(Debug, Clone)]
pub struct BorderStyles {
    pub thin: f32,
    pub medium: f32,
    pub thick: f32,
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
    pub radius_round: f32,
}

impl Default for BorderStyles {
    fn default() -> Self {
        Self {
            thin: 1.0,
            medium: 2.0,
            thick: 4.0,
            radius_sm: 4.0,
            radius_md: 8.0,
            radius_lg: 16.0,
            radius_round: 999.0,
        }
    }
}

/// Shadow styles
#[derive(Debug, Clone)]
pub struct ShadowStyles {
    pub sm: Shadow,
    pub md: Shadow,
    pub lg: Shadow,
    pub xl: Shadow,
}

impl Default for ShadowStyles {
    fn default() -> Self {
        Self {
            sm: Shadow::new(
                Color::rgba(0.0, 0.0, 0.0, 0.1),
                oxide_core::types::Point::new(0.0, 1.0),
                2.0,
                0.0,
            ),
            md: Shadow::new(
                Color::rgba(0.0, 0.0, 0.0, 0.15),
                oxide_core::types::Point::new(0.0, 2.0),
                4.0,
                0.0,
            ),
            lg: Shadow::new(
                Color::rgba(0.0, 0.0, 0.0, 0.2),
                oxide_core::types::Point::new(0.0, 4.0),
                8.0,
                0.0,
            ),
            xl: Shadow::new(
                Color::rgba(0.0, 0.0, 0.0, 0.25),
                oxide_core::types::Point::new(0.0, 8.0),
                16.0,
                0.0,
            ),
        }
    }
}

/// Component-specific themes
#[derive(Debug, Clone)]
pub struct ComponentThemes {
    pub button: HashMap<String, crate::button::ButtonStyle>,
    pub input: HashMap<String, crate::input::InputStyle>,
}

impl ComponentThemes {
    /// Light component themes
    pub fn light() -> Self {
        let mut button = HashMap::new();
        button.insert("primary".to_string(), crate::button::ButtonStyle::primary());
        button.insert("secondary".to_string(), crate::button::ButtonStyle::secondary());
        button.insert("text".to_string(), crate::button::ButtonStyle::ghost());

        let mut input = HashMap::new();
        input.insert("outlined".to_string(), crate::input::InputStyle::outlined());
        input.insert("filled".to_string(), crate::input::InputStyle::filled());

        Self { button, input }
    }

    /// Dark component themes
    pub fn dark() -> Self {
        // TODO: Create dark variants
        Self::light()
    }
}

impl Default for ComponentThemes {
    fn default() -> Self {
        Self::light()
    }
}

/// Theme provider for managing themes
pub struct ThemeProvider {
    current: Theme,
    themes: HashMap<String, Theme>,
}

impl ThemeProvider {
    /// Create a new theme provider
    pub fn new(theme: Theme) -> Self {
        let mut themes = HashMap::new();
        themes.insert("light".to_string(), Theme::light());
        themes.insert("dark".to_string(), Theme::dark());
        
        Self {
            current: theme,
            themes,
        }
    }

    /// Get the current theme
    pub fn current(&self) -> &Theme {
        &self.current
    }

    /// Set the current theme
    pub fn set_theme(&mut self, name: &str) {
        if let Some(theme) = self.themes.get(name).cloned() {
            self.current = theme;
        }
    }

    /// Register a custom theme
    pub fn register_theme(&mut self, name: impl Into<String>, theme: Theme) {
        self.themes.insert(name.into(), theme);
    }

    /// Toggle between light and dark themes
    pub fn toggle_theme(&mut self) {
        if self.current.name == "Light" {
            self.set_theme("dark");
        } else {
            self.set_theme("light");
        }
    }
}

impl Default for ThemeProvider {
    fn default() -> Self {
        Self::new(Theme::light())
    }
}
