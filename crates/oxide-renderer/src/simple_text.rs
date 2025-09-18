//! Simple text rendering system
//!
//! Uses system fonts first, falls back to bitmap fonts when system fonts are not available

use oxide_core::types::{Color, Point, Rect};
use oxide_core::layout::Size;
use crate::vertex::Vertex;
use std::collections::HashMap;

/// Simple font representation
#[derive(Debug, Clone)]
pub struct SimpleFont {
    pub family: String,
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Normal = 400,
    Bold = 700,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl Default for SimpleFont {
    fn default() -> Self {
        Self {
            family: Self::get_system_default_font(),
            size: 14.0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
        }
    }
}

impl SimpleFont {
    /// Get the system default font based on OS
    pub fn get_system_default_font() -> String {
        #[cfg(target_os = "windows")]
        return "Segoe UI".to_string();
        
        #[cfg(target_os = "macos")]
        return "SF Pro Display".to_string();
        
        #[cfg(target_os = "linux")]
        return "Ubuntu".to_string();
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return "Arial".to_string();
    }

    /// Create new font with system default
    pub fn system(size: f32) -> Self {
        Self {
            family: Self::get_system_default_font(),
            size,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
        }
    }

    /// Create new font with specified family
    pub fn new(family: impl Into<String>, size: f32) -> Self {
        Self {
            family: family.into(),
            size,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
        }
    }

    /// Set weight
    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Set style
    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }
}

/// Bitmap font character data
#[derive(Debug, Clone)]
pub struct BitmapChar {
    pub width: u32,
    pub height: u32,
    pub bitmap: Vec<u8>, // 1 bit per pixel, packed
    pub advance: f32,
    pub bearing_x: i32,
    pub bearing_y: i32,
}

/// Simple bitmap font for fallback
pub struct BitmapFont {
    pub size: f32,
    pub line_height: f32,
    pub chars: HashMap<char, BitmapChar>,
}

impl BitmapFont {
    /// Create a basic bitmap font with common ASCII characters
    pub fn create_basic_ascii(size: f32) -> Self {
        let mut chars = HashMap::new();
        
        // Create simple 8x8 bitmap patterns for basic ASCII
        // This is a very basic implementation - you could load from file or embed data
        
        // Space
        chars.insert(' ', BitmapChar {
            width: 4,
            height: 8,
            bitmap: vec![0; 4], // Empty bitmap
            advance: size * 0.5,
            bearing_x: 0,
            bearing_y: 0,
        });

        // Letter 'A'
        chars.insert('A', BitmapChar {
            width: 8,
            height: 8,
            bitmap: vec![
                0b00111000, // ..XXX...
                0b01101100, // .XX.XX..
                0b11000110, // XX...XX.
                0b11111110, // XXXXXXX.
                0b11000110, // XX...XX.
                0b11000110, // XX...XX.
                0b11000110, // XX...XX.
                0b00000000, // ........
            ],
            advance: size * 0.6,
            bearing_x: 0,
            bearing_y: 0,
        });

        // Add more characters as needed...
        // For now, we'll generate simple rectangles for missing chars
        
        Self {
            size,
            line_height: size * 1.2,
            chars,
        }
    }

    /// Get character data, or return fallback rectangle
    pub fn get_char(&self, c: char) -> BitmapChar {
        self.chars.get(&c).cloned().unwrap_or_else(|| {
            // Fallback: create a simple rectangle for unknown characters
            let char_width = (self.size * 0.6) as u32;
            let char_height = self.size as u32;
            BitmapChar {
                width: char_width,
                height: char_height,
                bitmap: vec![0xFF; ((char_width * char_height + 7) / 8) as usize], // Filled rectangle
                advance: self.size * 0.6,
                bearing_x: 0,
                bearing_y: 0,
            }
        })
    }
}

/// Simple text renderer that tries system fonts first, then bitmap fallback
pub struct SimpleTextRenderer {
    cosmic_text_available: bool,
    font_system: Option<cosmic_text::FontSystem>,
    bitmap_font: BitmapFont,
}

impl SimpleTextRenderer {
    /// Create new simple text renderer
    pub fn new() -> Self {
        let mut renderer = Self {
            cosmic_text_available: false,
            font_system: None,
            bitmap_font: BitmapFont::create_basic_ascii(16.0),
        };

        // Try to initialize cosmic-text with system fonts (simplified approach)
        let font_init_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut font_system = cosmic_text::FontSystem::new();
            font_system.db_mut().load_system_fonts();
            font_system
        }));
        
        match font_init_result {
            Ok(font_system) => {
                println!("Successfully loaded system fonts via cosmic-text");
                renderer.cosmic_text_available = true;
                renderer.font_system = Some(font_system);
            },
            Err(_) => {
                println!("Failed to load system fonts, using bitmap fallback");
            }
        }

        renderer
    }

    /// Render text using system fonts (cosmic-text) if available, otherwise bitmap
    pub fn render_text_simple(
        &mut self,
        text: &str,
        font: &SimpleFont,
        position: Point,
        color: Color,
    ) -> Vec<Vertex> {
        if self.cosmic_text_available {
            self.render_with_system_font(text, font, position, color)
        } else {
            self.render_with_bitmap_font(text, font, position, color)
        }
    }

    /// Render using system fonts via cosmic-text
    fn render_with_system_font(
        &mut self,
        text: &str,
        font: &SimpleFont,
        position: Point,
        color: Color,
    ) -> Vec<Vertex> {
        // For now, we'll create simple colored rectangles even for system fonts
        // This keeps the implementation simple while we get the basic system working
        self.render_as_colored_rectangles(text, font, position, color)
    }

    /// Render using bitmap fonts
    fn render_with_bitmap_font(
        &mut self,
        text: &str,
        font: &SimpleFont,
        position: Point,
        color: Color,
    ) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        let mut current_x = position.x;
        let current_y = position.y;

        for ch in text.chars() {
            let bitmap_char = self.bitmap_font.get_char(ch);
            
            // Create rectangle for this character
            let char_rect = Rect {
                x: current_x,
                y: current_y,
                width: bitmap_char.width as f32,
                height: bitmap_char.height as f32,
            };

            // Create vertices for a solid rectangle
            let char_vertices = self.create_char_rectangle(char_rect, color);
            vertices.extend(char_vertices);

            current_x += bitmap_char.advance;
        }

        vertices
    }

    /// Fallback: render text as colored rectangles (current implementation)
    fn render_as_colored_rectangles(
        &self,
        text: &str,
        font: &SimpleFont,
        position: Point,
        color: Color,
    ) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        let char_width = font.size * 0.6;
        let char_height = font.size;
        let mut current_x = position.x;

        for _ch in text.chars() {
            let char_rect = Rect {
                x: current_x,
                y: position.y,
                width: char_width,
                height: char_height,
            };

            let char_vertices = self.create_char_rectangle(char_rect, color);
            vertices.extend(char_vertices);

            current_x += char_width;
        }

        vertices
    }

    /// Create vertices for a character rectangle
    fn create_char_rectangle(&self, rect: Rect, color: Color) -> Vec<Vertex> {
        use crate::simple_text::ColorExt;
        vec![
            Vertex::solid([rect.x, rect.y, 0.0], color.to_array()),
            Vertex::solid([rect.x + rect.width, rect.y, 0.0], color.to_array()),
            Vertex::solid([rect.x + rect.width, rect.y + rect.height, 0.0], color.to_array()),
            Vertex::solid([rect.x, rect.y + rect.height, 0.0], color.to_array()),
        ]
    }

    /// Measure text dimensions
    pub fn measure_text(&self, text: &str, font: &SimpleFont) -> Size {
        let char_width = font.size * 0.6;
        let char_height = font.size;
        
        Size::new(
            text.chars().count() as f32 * char_width,
            char_height,
        )
    }

    /// Check if system fonts are available
    pub fn has_system_fonts(&self) -> bool {
        self.cosmic_text_available
    }

    /// Get font info for debugging
    pub fn get_font_info(&self) -> String {
        if self.cosmic_text_available {
            format!("System fonts available via cosmic-text")
        } else {
            format!("Using bitmap font fallback")
        }
    }
}

impl Default for SimpleTextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// Color extension trait for array conversion
pub trait ColorExt {
    fn to_array(&self) -> [f32; 4];
}

impl ColorExt for Color {
    fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_font_creation() {
        let font = SimpleFont::system(16.0);
        assert_eq!(font.size, 16.0);
        assert_eq!(font.weight, FontWeight::Normal);
        assert_eq!(font.style, FontStyle::Normal);
    }

    #[test]
    fn test_bitmap_font_creation() {
        let font = BitmapFont::create_basic_ascii(12.0);
        assert_eq!(font.size, 12.0);
        assert!(font.chars.contains_key(&' '));
        assert!(font.chars.contains_key(&'A'));
    }

    #[test]
    fn test_simple_text_renderer() {
        let mut renderer = SimpleTextRenderer::new();
        let font = SimpleFont::system(16.0);
        let vertices = renderer.render_text_simple(
            "Test",
            &font,
            Point::new(0.0, 0.0),
            Color::rgba(1.0, 1.0, 1.0, 1.0),
        );
        
        // Should have 4 vertices per character * 4 characters = 16 vertices
        assert_eq!(vertices.len(), 16);
    }

    #[test]
    fn test_text_measurement() {
        let renderer = SimpleTextRenderer::new();
        let font = SimpleFont::system(16.0);
        let size = renderer.measure_text("Hello", &font);
        
        assert_eq!(size.width, 5.0 * 16.0 * 0.6); // 5 chars * size * char_width_ratio
        assert_eq!(size.height, 16.0);
    }
}