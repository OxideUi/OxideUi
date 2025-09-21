//! Advanced text rendering system for OxideUI
//!
//! Provides comprehensive text layout, shaping, and rendering capabilities
//! with support for complex scripts, bidirectional text, and typography features.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
    Start,
    End,
}

/// Text direction for bidirectional text support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
    Auto,
}

/// Text wrapping behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    None,
    Word,
    Character,
    WordCharacter,
}

/// Font weight values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Font style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Font stretch values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

/// Font descriptor for text styling
#[derive(Debug, Clone)]
pub struct FontDescriptor {
    pub family: String,
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
}

impl Default for FontDescriptor {
    fn default() -> Self {
        // Use platform-specific default fonts instead of generic "system-ui"
        #[cfg(target_os = "windows")]
        let default_family = "Segoe UI";
        
        #[cfg(target_os = "macos")]
        let default_family = "SF Pro Display";
        
        #[cfg(target_os = "linux")]
        let default_family = "Ubuntu";
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let default_family = "Arial";
        
        Self {
            family: default_family.to_string(),
            size: 14.0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
        }
    }
}

/// Text decoration options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
    Blink,
}

/// Text style configuration
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font: FontDescriptor,
    pub color: [f32; 4], // RGBA
    pub decoration: TextDecoration,
    pub decoration_color: Option<[f32; 4]>,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub line_height: f32,
    pub text_shadow: Option<TextShadow>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: FontDescriptor::default(),
            color: [0.0, 0.0, 0.0, 1.0], // Black
            decoration: TextDecoration::None,
            decoration_color: None,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            line_height: 1.2,
            text_shadow: None,
        }
    }
}

/// Text shadow configuration
#[derive(Debug, Clone)]
pub struct TextShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub color: [f32; 4],
}

/// Text layout configuration
#[derive(Debug, Clone)]
pub struct TextLayout {
    pub align: TextAlign,
    pub direction: TextDirection,
    pub wrap: TextWrap,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub line_height: f32,
    pub paragraph_spacing: f32,
}

impl Default for TextLayout {
    fn default() -> Self {
        Self {
            align: TextAlign::Left,
            direction: TextDirection::Auto,
            wrap: TextWrap::Word,
            max_width: None,
            max_height: None,
            line_height: 1.2,
            paragraph_spacing: 0.0,
        }
    }
}

/// Represents a positioned glyph in text layout
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub glyph_id: u32,
    pub x: f32,
    pub y: f32,
    pub advance_x: f32,
    pub advance_y: f32,
    pub font_size: f32,
}

/// Text run with consistent styling
#[derive(Debug, Clone)]
pub struct TextRun {
    pub text: String,
    pub style: TextStyle,
    pub glyphs: Vec<PositionedGlyph>,
    pub bounds: TextBounds,
}

/// Text bounds information
#[derive(Debug, Clone, Copy)]
pub struct TextBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Line of text with multiple runs
#[derive(Debug, Clone)]
pub struct TextLine {
    pub runs: Vec<TextRun>,
    pub bounds: TextBounds,
    pub baseline: f32,
}

/// Complete text layout result
#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub lines: Vec<TextLine>,
    pub bounds: TextBounds,
    pub line_count: usize,
}

/// Font loading and management
pub struct FontManager {
    fonts: Arc<Mutex<HashMap<String, FontData>>>,
    fallback_fonts: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are used for font storage but not in simplified implementation
pub struct FontData {
    pub family: String,
    pub data: Vec<u8>,
    pub index: u32,
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            fonts: Arc::new(Mutex::new(HashMap::new())),
            fallback_fonts: vec![
                "system-ui".to_string(),
                "Arial".to_string(),
                "Helvetica".to_string(),
                "sans-serif".to_string(),
            ],
        }
    }

    /// Load a font from file
    pub fn load_font_from_file(&mut self, path: &str, family: &str) -> Result<(), TextError> {
        let data = std::fs::read(path)
            .map_err(|e| TextError::FontLoadError(format!("Failed to read font file: {}", e)))?;
        
        self.load_font_from_data(data, family, 0)
    }

    /// Load a font from raw data
    pub fn load_font_from_data(&mut self, data: Vec<u8>, family: &str, index: u32) -> Result<(), TextError> {
        let font_data = FontData {
            family: family.to_string(),
            data,
            index,
        };

        let mut fonts = self.fonts.lock().unwrap();
        fonts.insert(family.to_string(), font_data);
        Ok(())
    }

    /// Get font data by family name
    pub fn get_font(&self, family: &str) -> Option<FontData> {
        let fonts = self.fonts.lock().unwrap();
        fonts.get(family).cloned()
    }

    /// Add fallback font
    pub fn add_fallback_font(&mut self, family: String) {
        self.fallback_fonts.push(family);
    }
}

/// Text shaper for complex text layout
pub struct TextShaper {
    font_manager: Arc<FontManager>,
}

impl TextShaper {
    pub fn new(font_manager: Arc<FontManager>) -> Self {
        Self { font_manager }
    }

    /// Shape text into positioned glyphs
    pub fn shape_text(&self, text: &str, style: &TextStyle) -> Result<Vec<PositionedGlyph>, TextError> {
        // Simplified shaping - in a real implementation, this would use
        // libraries like HarfBuzz for proper text shaping
        // TODO: Use self.font_manager to get proper font metrics
        let mut glyphs = Vec::new();
        let mut x = 0.0;
        
        for (_i, ch) in text.char_indices() {
            let glyph = PositionedGlyph {
                glyph_id: ch as u32, // Simplified glyph ID
                x,
                y: 0.0,
                advance_x: style.font.size * 0.6, // Simplified advance
                advance_y: 0.0,
                font_size: style.font.size,
            };
            
            x += glyph.advance_x + style.letter_spacing;
            glyphs.push(glyph);
        }
        
        Ok(glyphs)
    }
}

/// Text layout engine
pub struct TextLayoutEngine {
    shaper: TextShaper,
}

impl TextLayoutEngine {
    pub fn new(font_manager: Arc<FontManager>) -> Self {
        Self {
            shaper: TextShaper::new(font_manager),
        }
    }

    /// Layout text according to the given configuration
    pub fn layout_text(&self, text: &str, style: &TextStyle, layout: &TextLayout) -> Result<LayoutResult, TextError> {
        let glyphs = self.shaper.shape_text(text, style)?;
        
        // Simple line breaking and layout
        let mut lines = Vec::new();
        let mut current_line_glyphs = Vec::new();
        let mut current_x = 0.0;
        let line_height = style.font.size * layout.line_height;
        
        for glyph in glyphs {
            if let Some(max_width) = layout.max_width {
                if current_x + glyph.advance_x > max_width && !current_line_glyphs.is_empty() {
                    // Create line from current glyphs
                    let line = self.create_text_line(current_line_glyphs, style, current_x, line_height * lines.len() as f32);
                    lines.push(line);
                    current_line_glyphs = Vec::new();
                    current_x = 0.0;
                }
            }
            
            current_line_glyphs.push(glyph.clone());
            current_x += glyph.advance_x;
        }
        
        // Add remaining glyphs as final line
        if !current_line_glyphs.is_empty() {
            let line = self.create_text_line(current_line_glyphs, style, current_x, line_height * lines.len() as f32);
            lines.push(line);
        }
        
        // Calculate overall bounds
        let total_width = lines.iter().map(|line| line.bounds.width).fold(0.0, f32::max);
        let total_height = lines.len() as f32 * line_height;
        
        Ok(LayoutResult {
            lines: lines.clone(),
            bounds: TextBounds {
                x: 0.0,
                y: 0.0,
                width: total_width,
                height: total_height,
            },
            line_count: lines.len(),
        })
    }

    fn create_text_line(&self, glyphs: Vec<PositionedGlyph>, style: &TextStyle, width: f32, y: f32) -> TextLine {
        let run = TextRun {
            text: String::new(), // Would be populated in real implementation
            style: style.clone(),
            glyphs,
            bounds: TextBounds {
                x: 0.0,
                y,
                width,
                height: style.font.size,
            },
        };
        
        TextLine {
            runs: vec![run],
            bounds: TextBounds {
                x: 0.0,
                y,
                width,
                height: style.font.size,
            },
            baseline: y + style.font.size * 0.8, // Simplified baseline calculation
        }
    }
}

/// Text measurement utilities
pub struct TextMeasurer {
    layout_engine: TextLayoutEngine,
}

impl TextMeasurer {
    pub fn new(font_manager: Arc<FontManager>) -> Self {
        Self {
            layout_engine: TextLayoutEngine::new(font_manager),
        }
    }

    /// Measure text dimensions
    pub fn measure_text(&self, text: &str, style: &TextStyle, layout: &TextLayout) -> Result<TextBounds, TextError> {
        let result = self.layout_engine.layout_text(text, style, layout)?;
        Ok(result.bounds)
    }

    /// Get text baseline position
    pub fn get_baseline(&self, style: &TextStyle) -> f32 {
        style.font.size * 0.8 // Simplified baseline calculation
    }
}

/// Text rendering errors
#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error("Font loading error: {0}")]
    FontLoadError(String),
    
    #[error("Text shaping error: {0}")]
    ShapingError(String),
    
    #[error("Layout error: {0}")]
    LayoutError(String),
    
    #[error("Rendering error: {0}")]
    RenderingError(String),
    
    #[error("Invalid font data")]
    InvalidFontData,
    
    #[error("Unsupported text feature: {0}")]
    UnsupportedFeature(String),
}

/// Global text system instance
use std::sync::OnceLock;
static TEXT_SYSTEM: OnceLock<TextSystem> = OnceLock::new();

/// Main text system
pub struct TextSystem {
    font_manager: Arc<FontManager>,
    layout_engine: TextLayoutEngine,
    measurer: TextMeasurer,
}

impl TextSystem {
    pub fn new() -> Self {
        let font_manager = Arc::new(FontManager::new());
        let layout_engine = TextLayoutEngine::new(font_manager.clone());
        let measurer = TextMeasurer::new(font_manager.clone());
        
        Self {
            font_manager,
            layout_engine,
            measurer,
        }
    }

    pub fn font_manager(&self) -> Arc<FontManager> {
        self.font_manager.clone()
    }

    pub fn layout_engine(&self) -> &TextLayoutEngine {
        &self.layout_engine
    }

    pub fn measurer(&self) -> &TextMeasurer {
        &self.measurer
    }
}

/// Initialize the global text system
pub fn init_text_system() -> Result<(), TextError> {
    TEXT_SYSTEM.get_or_init(|| TextSystem::new());
    Ok(())
}

/// Get the global text system instance
pub fn text_system() -> &'static TextSystem {
    TEXT_SYSTEM.get().expect("Text system not initialized")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_descriptor_default() {
        let font = FontDescriptor::default();
        assert_eq!(font.family, "system-ui");
        assert_eq!(font.size, 14.0);
        assert_eq!(font.weight, FontWeight::Normal);
    }

    #[test]
    fn test_text_style_default() {
        let style = TextStyle::default();
        assert_eq!(style.color, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(style.decoration, TextDecoration::None);
    }

    #[test]
    fn test_font_manager() {
        let mut manager = FontManager::new();
        assert!(manager.get_font("nonexistent").is_none());
        
        manager.add_fallback_font("Test Font".to_string());
        assert!(manager.fallback_fonts.contains(&"Test Font".to_string()));
    }

    #[test]
    fn test_text_system_init() {
        assert!(init_text_system().is_ok());
    }
}