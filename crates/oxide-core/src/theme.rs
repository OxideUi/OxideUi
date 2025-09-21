//! Theming system for OxideUI
//!
//! Provides comprehensive theming support including dark/light modes,
//! custom color schemes, typography, spacing, and component styling

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

/// Color representation with alpha channel
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color from RGB values (0.0 - 1.0)
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a new color from RGBA values (0.0 - 1.0)
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from hex string (#RRGGBB or #RRGGBBAA)
    pub fn from_hex(hex: &str) -> Result<Self, &'static str> {
        let hex = hex.trim_start_matches('#');
        
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
                Ok(Self::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid hex color")?;
                Ok(Self::rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0))
            }
            _ => Err("Invalid hex color length"),
        }
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        if self.a < 1.0 {
            format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                (self.r * 255.0) as u8,
                (self.g * 255.0) as u8,
                (self.b * 255.0) as u8,
                (self.a * 255.0) as u8
            )
        } else {
            format!(
                "#{:02X}{:02X}{:02X}",
                (self.r * 255.0) as u8,
                (self.g * 255.0) as u8,
                (self.b * 255.0) as u8
            )
        }
    }

    /// Lighten the color by a factor (0.0 - 1.0)
    pub fn lighten(&self, factor: f32) -> Self {
        Self {
            r: (self.r + (1.0 - self.r) * factor).min(1.0),
            g: (self.g + (1.0 - self.g) * factor).min(1.0),
            b: (self.b + (1.0 - self.b) * factor).min(1.0),
            a: self.a,
        }
    }

    /// Darken the color by a factor (0.0 - 1.0)
    pub fn darken(&self, factor: f32) -> Self {
        Self {
            r: (self.r * (1.0 - factor)).max(0.0),
            g: (self.g * (1.0 - factor)).max(0.0),
            b: (self.b * (1.0 - factor)).max(0.0),
            a: self.a,
        }
    }

    /// Set alpha channel
    pub fn with_alpha(&self, alpha: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha.clamp(0.0, 1.0),
        }
    }

    /// Convert to array format [r, g, b, a] for renderer compatibility
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Mix with another color
    pub fn mix(&self, other: &Color, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * factor,
            g: self.g + (other.g - self.g) * factor,
            b: self.b + (other.b - self.b) * factor,
            a: self.a + (other.a - self.a) * factor,
        }
    }

    /// Convert to types::Color for rendering
    pub fn to_types_color(&self) -> crate::types::Color {
        crate::types::Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

/// Common color constants
impl Color {
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
    pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);
}

/// Typography configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Typography {
    /// Font family
    pub font_family: String,
    /// Base font size in pixels
    pub base_size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Font weight
    pub weight: FontWeight,
    /// Letter spacing
    pub letter_spacing: f32,
}

impl Default for Typography {
    fn default() -> Self {
        // Use platform-specific default fonts with proper fallbacks
        #[cfg(target_os = "windows")]
        let font_family = "Segoe UI, Tahoma, Arial, sans-serif";
        
        #[cfg(target_os = "macos")]
        let font_family = "SF Pro Display, Helvetica Neue, Arial, sans-serif";
        
        #[cfg(target_os = "linux")]
        let font_family = "Ubuntu, DejaVu Sans, Liberation Sans, Arial, sans-serif";
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let font_family = "Arial, sans-serif";
        
        Self {
            font_family: font_family.to_string(),
            base_size: 14.0,
            line_height: 1.5,
            weight: FontWeight::Normal,
            letter_spacing: 0.0,
        }
    }
}

/// Font weight enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

/// Spacing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spacing {
    /// Extra small spacing
    pub xs: f32,
    /// Small spacing
    pub sm: f32,
    /// Medium spacing
    pub md: f32,
    /// Large spacing
    pub lg: f32,
    /// Extra large spacing
    pub xl: f32,
    /// Extra extra large spacing
    pub xxl: f32,
}

impl Default for Spacing {
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

/// Border radius configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderRadius {
    /// No radius
    pub none: f32,
    /// Small radius
    pub sm: f32,
    /// Medium radius
    pub md: f32,
    /// Large radius
    pub lg: f32,
    /// Full radius (circular)
    pub full: f32,
}

impl Default for BorderRadius {
    fn default() -> Self {
        Self {
            none: 0.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            full: 9999.0,
        }
    }
}

/// Shadow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shadow {
    pub color: Color,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
}

impl Shadow {
    pub fn new(color: Color, offset_x: f32, offset_y: f32, blur_radius: f32) -> Self {
        Self {
            color,
            offset_x,
            offset_y,
            blur_radius,
            spread_radius: 0.0,
        }
    }
}

/// Elevation shadows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Elevation {
    pub none: Option<Shadow>,
    pub sm: Shadow,
    pub md: Shadow,
    pub lg: Shadow,
    pub xl: Shadow,
}

impl Default for Elevation {
    fn default() -> Self {
        Self {
            none: None,
            sm: Shadow::new(Color::rgba(0.0, 0.0, 0.0, 0.1), 0.0, 1.0, 3.0),
            md: Shadow::new(Color::rgba(0.0, 0.0, 0.0, 0.15), 0.0, 4.0, 6.0),
            lg: Shadow::new(Color::rgba(0.0, 0.0, 0.0, 0.2), 0.0, 10.0, 15.0),
            xl: Shadow::new(Color::rgba(0.0, 0.0, 0.0, 0.25), 0.0, 20.0, 25.0),
        }
    }
}

/// Color palette for a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    // Primary colors
    pub primary: Color,
    pub primary_variant: Color,
    
    // Secondary colors
    pub secondary: Color,
    pub secondary_variant: Color,
    
    // Background colors
    pub background: Color,
    pub surface: Color,
    pub surface_variant: Color,
    
    // Text colors
    pub on_primary: Color,
    pub on_secondary: Color,
    pub on_background: Color,
    pub on_surface: Color,
    
    // State colors
    pub error: Color,
    pub on_error: Color,
    pub warning: Color,
    pub on_warning: Color,
    pub success: Color,
    pub on_success: Color,
    pub info: Color,
    pub on_info: Color,
    
    // Outline and divider
    pub outline: Color,
    pub outline_variant: Color,
    pub divider: Color,
    
    // Disabled state
    pub disabled: Color,
    pub on_disabled: Color,
}

impl ColorPalette {
    /// Create a light theme color palette
    pub fn light() -> Self {
        Self {
            primary: Color::from_hex("#6366F1").unwrap(),
            primary_variant: Color::from_hex("#4F46E5").unwrap(),
            secondary: Color::from_hex("#EC4899").unwrap(),
            secondary_variant: Color::from_hex("#DB2777").unwrap(),
            background: Color::WHITE,
            surface: Color::from_hex("#F9FAFB").unwrap(),
            surface_variant: Color::from_hex("#F3F4F6").unwrap(),
            on_primary: Color::WHITE,
            on_secondary: Color::WHITE,
            on_background: Color::from_hex("#111827").unwrap(),
            on_surface: Color::from_hex("#FFFF00").unwrap(), // Bright yellow for visibility
            error: Color::from_hex("#EF4444").unwrap(),
            on_error: Color::WHITE,
            warning: Color::from_hex("#F59E0B").unwrap(),
            on_warning: Color::WHITE,
            success: Color::from_hex("#10B981").unwrap(),
            on_success: Color::WHITE,
            info: Color::from_hex("#3B82F6").unwrap(),
            on_info: Color::WHITE,
            outline: Color::from_hex("#D1D5DB").unwrap(),
            outline_variant: Color::from_hex("#E5E7EB").unwrap(),
            divider: Color::from_hex("#F3F4F6").unwrap(),
            disabled: Color::from_hex("#9CA3AF").unwrap(),
            on_disabled: Color::from_hex("#6B7280").unwrap(),
        }
    }

    /// Create a dark theme color palette
    pub fn dark() -> Self {
        Self {
            primary: Color::from_hex("#818CF8").unwrap(),
            primary_variant: Color::from_hex("#6366F1").unwrap(),
            secondary: Color::from_hex("#F472B6").unwrap(),
            secondary_variant: Color::from_hex("#EC4899").unwrap(),
            background: Color::from_hex("#111827").unwrap(),
            surface: Color::from_hex("#1F2937").unwrap(),
            surface_variant: Color::from_hex("#374151").unwrap(),
            on_primary: Color::from_hex("#1E1B4B").unwrap(),
            on_secondary: Color::from_hex("#831843").unwrap(),
            on_background: Color::from_hex("#F9FAFB").unwrap(),
            on_surface: Color::from_hex("#00FFFF").unwrap(), // Bright cyan for visibility
            error: Color::from_hex("#F87171").unwrap(),
            on_error: Color::from_hex("#7F1D1D").unwrap(),
            warning: Color::from_hex("#FBBF24").unwrap(),
            on_warning: Color::from_hex("#78350F").unwrap(),
            success: Color::from_hex("#34D399").unwrap(),
            on_success: Color::from_hex("#064E3B").unwrap(),
            info: Color::from_hex("#60A5FA").unwrap(),
            on_info: Color::from_hex("#1E3A8A").unwrap(),
            outline: Color::from_hex("#4B5563").unwrap(),
            outline_variant: Color::from_hex("#374151").unwrap(),
            divider: Color::from_hex("#374151").unwrap(),
            disabled: Color::from_hex("#6B7280").unwrap(),
            on_disabled: Color::from_hex("#9CA3AF").unwrap(),
        }
    }
}

/// Theme mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

/// Complete theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Theme mode
    pub mode: ThemeMode,
    /// Color palette
    pub colors: ColorPalette,
    /// Typography settings
    pub typography: Typography,
    /// Spacing configuration
    pub spacing: Spacing,
    /// Border radius configuration
    pub border_radius: BorderRadius,
    /// Elevation shadows
    pub elevation: Elevation,
    /// Custom properties
    pub custom: HashMap<String, String>,
}

impl Theme {
    /// Create a new light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            mode: ThemeMode::Light,
            colors: ColorPalette::light(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            border_radius: BorderRadius::default(),
            elevation: Elevation::default(),
            custom: HashMap::new(),
        }
    }

    /// Create a new dark theme
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            mode: ThemeMode::Dark,
            colors: ColorPalette::dark(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            border_radius: BorderRadius::default(),
            elevation: Elevation::default(),
            custom: HashMap::new(),
        }
    }

    /// Get a custom property
    pub fn get_custom(&self, key: &str) -> Option<&String> {
        self.custom.get(key)
    }

    /// Set a custom property
    pub fn set_custom(&mut self, key: String, value: String) {
        self.custom.insert(key, value);
    }
}

/// Theme change event
#[derive(Debug, Clone)]
pub struct ThemeChangeEvent {
    pub old_theme: String,
    pub new_theme: String,
    pub mode_changed: bool,
}

/// Theme change listener
pub trait ThemeChangeListener: Send + Sync {
    fn on_theme_changed(&self, event: &ThemeChangeEvent);
}

/// Theme manager for handling theme switching and persistence
pub struct ThemeManager {
    current_theme: Arc<RwLock<Theme>>,
    themes: Arc<RwLock<HashMap<String, Theme>>>,
    listeners: Arc<RwLock<Vec<Arc<dyn ThemeChangeListener>>>>,
    system_theme_mode: Arc<RwLock<ThemeMode>>,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        let light_theme = Theme::light();
        let dark_theme = Theme::dark();
        
        themes.insert(light_theme.name.clone(), light_theme.clone());
        themes.insert(dark_theme.name.clone(), dark_theme.clone());

        Self {
            current_theme: Arc::new(RwLock::new(light_theme)),
            themes: Arc::new(RwLock::new(themes)),
            listeners: Arc::new(RwLock::new(Vec::new())),
            system_theme_mode: Arc::new(RwLock::new(ThemeMode::Light)),
        }
    }

    /// Get the current theme
    pub fn current_theme(&self) -> Theme {
        self.current_theme.read().clone()
    }

    /// Set the current theme by name
    pub fn set_theme(&self, theme_name: &str) -> Result<(), &'static str> {
        let themes = self.themes.read();
        let new_theme = themes.get(theme_name)
            .ok_or("Theme not found")?
            .clone();
        
        let old_theme_name = self.current_theme.read().name.clone();
        let mode_changed = self.current_theme.read().mode != new_theme.mode;
        
        *self.current_theme.write() = new_theme;
        
        // Notify listeners
        let event = ThemeChangeEvent {
            old_theme: old_theme_name,
            new_theme: theme_name.to_string(),
            mode_changed,
        };
        
        self.notify_listeners(&event);
        Ok(())
    }

    /// Switch between light and dark themes
    pub fn toggle_theme_mode(&self) {
        let current_mode = self.current_theme.read().mode;
        let new_mode = match current_mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::System => {
                // Toggle based on current system theme
                let system_mode = *self.system_theme_mode.read();
                match system_mode {
                    ThemeMode::Light => ThemeMode::Dark,
                    _ => ThemeMode::Light,
                }
            }
        };

        let theme_name = match new_mode {
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
            ThemeMode::System => "Light", // Fallback
        };

        let _ = self.set_theme(theme_name);
    }

    /// Add a custom theme
    pub fn add_theme(&self, theme: Theme) {
        let theme_name = theme.name.clone();
        self.themes.write().insert(theme_name, theme);
    }

    /// Remove a theme
    pub fn remove_theme(&self, theme_name: &str) -> Result<(), &'static str> {
        if theme_name == "Light" || theme_name == "Dark" {
            return Err("Cannot remove built-in themes");
        }

        let mut themes = self.themes.write();
        themes.remove(theme_name)
            .ok_or("Theme not found")?;
        
        Ok(())
    }

    /// Get all available theme names
    pub fn get_theme_names(&self) -> Vec<String> {
        self.themes.read().keys().cloned().collect()
    }

    /// Add a theme change listener
    pub fn add_listener(&self, listener: Arc<dyn ThemeChangeListener>) {
        self.listeners.write().push(listener);
    }

    /// Remove all listeners
    pub fn clear_listeners(&self) {
        self.listeners.write().clear();
    }

    /// Update system theme mode (for system theme detection)
    pub fn update_system_theme_mode(&self, mode: ThemeMode) {
        *self.system_theme_mode.write() = mode;
        
        // If current theme is system, update accordingly
        if self.current_theme.read().mode == ThemeMode::System {
            let theme_name = match mode {
                ThemeMode::Light => "Light",
                ThemeMode::Dark => "Dark",
                ThemeMode::System => "Light", // Fallback
            };
            let _ = self.set_theme(theme_name);
        }
    }

    /// Notify all listeners of theme change
    fn notify_listeners(&self, event: &ThemeChangeEvent) {
        let listeners = self.listeners.read();
        for listener in listeners.iter() {
            listener.on_theme_changed(event);
        }
    }

    /// Export theme to JSON
    pub fn export_theme(&self, theme_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let themes = self.themes.read();
        let theme = themes.get(theme_name)
            .ok_or("Theme not found")?;
        
        Ok(serde_json::to_string_pretty(theme)?)
    }

    /// Import theme from JSON
    pub fn import_theme(&self, json: &str) -> Result<(), Box<dyn std::error::Error>> {
        let theme: Theme = serde_json::from_str(json)?;
        self.add_theme(theme);
        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for theme operations
pub mod utils {
    use super::*;

    /// Detect system theme preference (placeholder - would need platform-specific implementation)
    pub fn detect_system_theme() -> ThemeMode {
        // This would need platform-specific implementation
        // For now, return Light as default
        ThemeMode::Light
    }

    /// Generate a color palette from a primary color
    pub fn generate_palette_from_primary(primary: Color) -> ColorPalette {
        let mut palette = ColorPalette::light();
        palette.primary = primary;
        palette.primary_variant = primary.darken(0.1);
        palette
    }

    /// Calculate contrast ratio between two colors
    pub fn contrast_ratio(color1: &Color, color2: &Color) -> f32 {
        let l1 = relative_luminance(color1);
        let l2 = relative_luminance(color2);
        
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Calculate relative luminance of a color
    fn relative_luminance(color: &Color) -> f32 {
        let r = if color.r <= 0.03928 { color.r / 12.92 } else { ((color.r + 0.055) / 1.055).powf(2.4) };
        let g = if color.g <= 0.03928 { color.g / 12.92 } else { ((color.g + 0.055) / 1.055).powf(2.4) };
        let b = if color.b <= 0.03928 { color.b / 12.92 } else { ((color.b + 0.055) / 1.055).powf(2.4) };
        
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Check if a color combination meets WCAG accessibility standards
    pub fn meets_wcag_aa(foreground: &Color, background: &Color) -> bool {
        contrast_ratio(foreground, background) >= 4.5
    }

    /// Check if a color combination meets WCAG AAA accessibility standards
    pub fn meets_wcag_aaa(foreground: &Color, background: &Color) -> bool {
        contrast_ratio(foreground, background) >= 7.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::rgb(1.0, 0.5, 0.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF8000").unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color::rgb(1.0, 0.5, 0.0);
        assert_eq!(color.to_hex(), "#FF8000");
    }

    #[test]
    fn test_color_lighten() {
        let color = Color::rgb(0.5, 0.5, 0.5);
        let lighter = color.lighten(0.2);
        assert!(lighter.r > color.r);
        assert!(lighter.g > color.g);
        assert!(lighter.b > color.b);
    }

    #[test]
    fn test_color_darken() {
        let color = Color::rgb(0.5, 0.5, 0.5);
        let darker = color.darken(0.2);
        assert!(darker.r < color.r);
        assert!(darker.g < color.g);
        assert!(darker.b < color.b);
    }

    #[test]
    fn test_theme_creation() {
        let theme = Theme::light();
        assert_eq!(theme.name, "Light");
        assert_eq!(theme.mode, ThemeMode::Light);
    }

    #[test]
    fn test_theme_manager() {
        let manager = ThemeManager::new();
        assert_eq!(manager.current_theme().name, "Light");
        
        manager.set_theme("Dark").unwrap();
        assert_eq!(manager.current_theme().name, "Dark");
    }

    #[test]
    fn test_contrast_ratio() {
        let white = Color::WHITE;
        let black = Color::BLACK;
        let ratio = utils::contrast_ratio(&white, &black);
        assert!(ratio > 20.0); // Should be 21:1 for perfect white/black
    }

    #[test]
    fn test_wcag_compliance() {
        let white = Color::WHITE;
        let black = Color::BLACK;
        assert!(utils::meets_wcag_aa(&white, &black));
        assert!(utils::meets_wcag_aaa(&white, &black));
    }
}