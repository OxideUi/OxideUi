//! Text widget implementation
//! 
//! Provides text display components with various styles, formatting, and layout options.

use oxide_core::{
    layout::{Size, Constraints, Layout},
    types::{Rect, Color, Point},
    state::{Signal},
    theme::{Theme},
    event::{Event, EventResult},
};
use oxide_renderer::{
    vertex::{Vertex, VertexBuilder},
    batch::RenderBatch,
};
use crate::widget::{Widget, WidgetId, generate_id};
use std::{sync::Arc, any::Any};

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Text vertical alignment options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
    Baseline,
}

/// Text overflow behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextOverflow {
    Clip,
    Ellipsis,
    Wrap,
    Scroll,
}

/// Text decoration options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
}

/// Font weight options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl FontWeight {
    pub fn to_numeric(&self) -> u16 {
        match self {
            FontWeight::Thin => 100,
            FontWeight::ExtraLight => 200,
            FontWeight::Light => 300,
            FontWeight::Normal => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
        }
    }
}

/// Font style options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Text style configuration
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub color: Color,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub text_align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub text_decoration: TextDecoration,
    pub decoration_color: Color,
    pub text_overflow: TextOverflow,
    pub max_lines: Option<usize>,
    pub selectable: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "system-ui".to_string(),
            font_size: 14.0,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            line_height: 1.4,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            text_align: TextAlign::Left,
            vertical_align: VerticalAlign::Top,
            text_decoration: TextDecoration::None,
            decoration_color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            text_overflow: TextOverflow::Clip,
            max_lines: None,
            selectable: false,
        }
    }
}

impl TextStyle {
    /// Create a heading style
    pub fn heading(level: u8) -> Self {
        let font_size = match level {
            1 => 32.0,
            2 => 24.0,
            3 => 20.0,
            4 => 18.0,
            5 => 16.0,
            6 => 14.0,
            _ => 14.0,
        };

        Self {
            font_size,
            font_weight: FontWeight::Bold,
            line_height: 1.2,
            ..Default::default()
        }
    }

    /// Create a body text style
    pub fn body() -> Self {
        Self {
            font_size: 14.0,
            line_height: 1.5,
            ..Default::default()
        }
    }

    /// Create a caption style
    pub fn caption() -> Self {
        Self {
            font_size: 12.0,
            color: Color::rgba(0.5, 0.5, 0.5, 1.0),
            line_height: 1.3,
            ..Default::default()
        }
    }

    /// Create a code style
    pub fn code() -> Self {
        Self {
            font_family: "monospace".to_string(),
            font_size: 13.0,
            color: Color::rgba(0.2, 0.2, 0.2, 1.0),
            letter_spacing: 0.5,
            ..Default::default()
        }
    }

    /// Create a link style
    pub fn link() -> Self {
        Self {
            color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            text_decoration: TextDecoration::Underline,
            decoration_color: Color::rgba(0.0, 0.4, 0.8, 1.0),
            ..Default::default()
        }
    }
}

/// Text span for rich text formatting
#[derive(Debug, Clone)]
pub struct TextSpan {
    pub text: String,
    pub style: Option<TextStyle>,
    pub start: usize,
    pub end: usize,
}

impl TextSpan {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let len = text.len();
        Self {
            text,
            style: None,
            start: 0,
            end: len,
        }
    }

    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn with_range(mut self, start: usize, end: usize) -> Self {
        self.start = start;
        self.end = end;
        self
    }
}

/// Text widget
#[derive(Debug)]
pub struct Text {
    id: WidgetId,
    content: String,
    spans: Vec<TextSpan>,
    style: TextStyle,
    bounds: Signal<Rect>,
    visible: Signal<bool>,
    selectable: Signal<bool>,
    selection_start: Signal<Option<usize>>,
    selection_end: Signal<Option<usize>>,
    theme: Option<Arc<Theme>>,
    measured_size: Signal<Size>,
    line_breaks: Signal<Vec<usize>>,
}

impl Text {
    /// Create a new text widget
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: generate_id(),
            content: content.into(),
            spans: Vec::new(),
            style: TextStyle::default(),
            bounds: Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            visible: Signal::new(true),
            selectable: Signal::new(false),
            selection_start: Signal::new(None),
            selection_end: Signal::new(None),
            theme: None,
            measured_size: Signal::new(Size::new(0.0, 0.0)),
            line_breaks: Signal::new(Vec::new()),
        }
    }

    /// Set text style
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// Set as heading
    pub fn heading(mut self, level: u8) -> Self {
        self.style = TextStyle::heading(level);
        self
    }

    /// Set as body text
    pub fn body(mut self) -> Self {
        self.style = TextStyle::body();
        self
    }

    /// Set as caption
    pub fn caption(mut self) -> Self {
        self.style = TextStyle::caption();
        self
    }

    /// Set as code
    pub fn code(mut self) -> Self {
        self.style = TextStyle::code();
        self
    }

    /// Set as link
    pub fn link(mut self) -> Self {
        self.style = TextStyle::link();
        self
    }

    /// Set text color
    pub fn color(mut self, color: Color) -> Self {
        self.style.color = color;
        self
    }

    /// Set font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.style.font_size = size;
        self
    }

    /// Set font weight
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.style.font_weight = weight;
        self
    }

    /// Set text alignment
    pub fn align(mut self, align: TextAlign) -> Self {
        self.style.text_align = align;
        self
    }

    /// Set text overflow behavior
    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.style.text_overflow = overflow;
        self
    }

    /// Set maximum lines
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.style.max_lines = Some(lines);
        self
    }

    /// Set selectable
    pub fn selectable(self, selectable: bool) -> Self {
        self.selectable.set(selectable);
        self
    }

    /// Set visible state
    pub fn visible(self, visible: bool) -> Self {
        self.visible.set(visible);
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: Arc<Theme>) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Add a text span for rich formatting
    pub fn add_span(mut self, span: TextSpan) -> Self {
        self.spans.push(span);
        self
    }



    /// Get text content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Set text content
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.invalidate_layout();
    }

    /// Check if text is visible
    pub fn is_visible(&self) -> bool {
        self.visible.get()
    }

    /// Check if text is selectable
    pub fn is_selectable(&self) -> bool {
        self.selectable.get()
    }

    /// Get current selection
    pub fn get_selection(&self) -> Option<(usize, usize)> {
        match (self.selection_start.get(), self.selection_end.get()) {
            (Some(start), Some(end)) => Some((start.min(end), start.max(end))),
            _ => None,
        }
    }

    /// Set text selection
    pub fn set_selection(&self, start: Option<usize>, end: Option<usize>) {
        self.selection_start.set(start);
        self.selection_end.set(end);
    }

    /// Clear selection
    pub fn clear_selection(&self) {
        self.selection_start.set(None);
        self.selection_end.set(None);
    }

    /// Invalidate layout (force remeasurement)
    fn invalidate_layout(&self) {
        self.measured_size.set(Size::new(0.0, 0.0));
        self.line_breaks.set(Vec::new());
    }

    /// Measure text size
    pub fn measure_text(&self, available_width: f32) -> Size {
        // Simple text measurement (in a real implementation, this would use font metrics)
        let char_width = self.style.font_size * 0.6;
        let line_height = self.style.font_size * self.style.line_height;
        
        let mut lines = Vec::new();
        let mut current_line_width = 0.0;
        let mut current_line_start = 0;
        
        let words: Vec<&str> = self.content.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            let word_width = word.len() as f32 * char_width;
            
            if current_line_width + word_width > available_width && current_line_width > 0.0 {
                // Start new line
                lines.push(current_line_start..i);
                current_line_start = i;
                current_line_width = word_width;
            } else {
                current_line_width += word_width;
                if i < words.len() - 1 {
                    current_line_width += char_width; // Space
                }
            }
        }
        
        if current_line_start < words.len() {
            lines.push(current_line_start..words.len());
        }
        
        // Apply max_lines constraint
        if let Some(max_lines) = self.style.max_lines {
            lines.truncate(max_lines);
        }
        
        let width = if lines.is_empty() {
            0.0
        } else {
            available_width.min(self.content.len() as f32 * char_width)
        };
        
        let height = lines.len() as f32 * line_height;
        
        let size = Size::new(width, height);
        self.measured_size.set(size);
        
        size
    }

    /// Calculate text size
    pub fn calculate_size(&self, available_size: Size) -> Size {
        let measured = self.measured_size.get();
        if measured.width > 0.0 && measured.height > 0.0 {
            return Size::new(
                measured.width.min(available_size.width),
                measured.height.min(available_size.height),
            );
        }
        
        self.measure_text(available_size.width)
    }

    /// Layout the text
    pub fn layout(&self, bounds: Rect) {
        self.bounds.set(bounds);
        self.measure_text(bounds.width);
    }

    /// Handle mouse press for text selection
    pub fn on_mouse_press(&self, point: Point) -> bool {
        if !self.is_selectable() || !self.is_visible() {
            return false;
        }

        let bounds = self.bounds.get();
        if bounds.contains(point) {
            // Calculate character position (simplified)
            let relative_x = point.x - bounds.x;
            let relative_y = point.y - bounds.y;
            
            let char_width = self.style.font_size * 0.6;
            let line_height = self.style.font_size * self.style.line_height;
            
            let line = (relative_y / line_height) as usize;
            let char_in_line = (relative_x / char_width) as usize;
            
            // Simple character position calculation
            let position = char_in_line.min(self.content.len());
            
            self.set_selection(Some(position), Some(position));
            return true;
        }
        false
    }

    /// Handle mouse drag for text selection
    pub fn on_mouse_drag(&self, point: Point) -> bool {
        if !self.is_selectable() || !self.is_visible() {
            return false;
        }

        if let Some(start) = self.selection_start.get() {
            let bounds = self.bounds.get();
            if bounds.contains(point) {
                let relative_x = point.x - bounds.x;
                let char_width = self.style.font_size * 0.6;
                let position = (relative_x / char_width) as usize;
                
                self.selection_end.set(Some(position.min(self.content.len())));
                return true;
            }
        }
        false
    }

    /// Render the text
    pub fn render(&self, batch: &mut RenderBatch) {
        if !self.is_visible() {
            return;
        }

        let bounds = self.bounds.get();
        
        // Render selection background if any
        if let Some((start, end)) = self.get_selection() {
            if start != end {
                let selection_color = Color::rgba(0.0, 0.4, 0.8, 0.3);
                
                // Simple selection rendering (would need proper text metrics)
                let char_width = self.style.font_size * 0.6;
                let selection_x = bounds.x + start as f32 * char_width;
                let selection_width = (end - start) as f32 * char_width;
                
                let (vertices, indices) = VertexBuilder::rectangle(
                    selection_x,
                    bounds.y,
                    selection_width,
                    bounds.height,
                    selection_color,
                );
                batch.add_vertices(&vertices, &indices);
            }
        }
        
        // Calculate text position based on alignment
        let text_x = match self.style.text_align {
            TextAlign::Left => bounds.x,
            TextAlign::Center => bounds.x + bounds.width / 2.0 - (self.content.len() as f32 * self.style.font_size * 0.3),
            TextAlign::Right => bounds.x + bounds.width - (self.content.len() as f32 * self.style.font_size * 0.6),
            TextAlign::Justify => bounds.x, // Simplified
        };
        
        let text_y = match self.style.vertical_align {
            VerticalAlign::Top => bounds.y,
            VerticalAlign::Middle => bounds.y + bounds.height / 2.0 - self.style.font_size / 2.0,
            VerticalAlign::Bottom => bounds.y + bounds.height - self.style.font_size,
            VerticalAlign::Baseline => bounds.y + self.style.font_size * 0.8,
        };
        
        // Render main text
        batch.add_text(
            self.content.clone(),
            (text_x, text_y),
            self.style.color,
            self.style.font_size,
        );
        
        // Render text decoration if any
        if self.style.text_decoration != TextDecoration::None {
            let decoration_y = match self.style.text_decoration {
                TextDecoration::Underline => text_y + self.style.font_size + 2.0,
                TextDecoration::Overline => text_y - 2.0,
                TextDecoration::LineThrough => text_y + self.style.font_size / 2.0,
                TextDecoration::None => text_y,
            };
            
            let text_width = self.content.len() as f32 * self.style.font_size * 0.6;
            let (vertices, indices) = VertexBuilder::line(
                text_x,
                decoration_y,
                text_x + text_width,
                decoration_y,
                1.0,
                self.style.decoration_color,
            );
            batch.add_vertices(&vertices, &indices);
        }
        
        // Render spans if any (rich text)
        for span in &self.spans {
            if let Some(span_style) = &span.style {
                let span_text = &span.text[span.start..span.end.min(span.text.len())];
                let span_x = text_x + span.start as f32 * self.style.font_size * 0.6;
                
                batch.add_text(
                    span_text.to_string(),
                    (span_x, text_y),
                    span_style.color,
                    span_style.font_size,
                );
            }
        }
    }

    /// Apply theme to text
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.style.font_family = theme.typography.font_family.clone();
        self.style.font_size = theme.typography.base_size;
        self.style.color = theme.colors.on_surface;
    }
}

/// Text builder for fluent API
pub struct TextBuilder {
    text: Text,
}

impl TextBuilder {
    /// Create a new text builder
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            text: Text::new(content),
        }
    }

    /// Set style
    pub fn style(mut self, style: TextStyle) -> Self {
        self.text = self.text.style(style);
        self
    }

    /// Set as heading
    pub fn heading(mut self, level: u8) -> Self {
        self.text = self.text.heading(level);
        self
    }

    /// Set as body text
    pub fn body(mut self) -> Self {
        self.text = self.text.body();
        self
    }

    /// Set color
    pub fn color(mut self, color: Color) -> Self {
        self.text = self.text.color(color);
        self
    }

    /// Set font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.text = self.text.font_size(size);
        self
    }

    /// Set alignment
    pub fn align(mut self, align: TextAlign) -> Self {
        self.text = self.text.align(align);
        self
    }

    /// Set selectable
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.text = self.text.selectable(selectable);
        self
    }

    /// Build the text widget
    pub fn build(self) -> Text {
        self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_creation() {
        let text = Text::new("Hello, World!");
        assert_eq!(text.content(), "Hello, World!");
        assert!(text.is_visible());
        assert!(!text.is_selectable());
    }

    #[test]
    fn test_text_styles() {
        let heading = Text::new("Heading").heading(1);
        let body = Text::new("Body").body();
        let caption = Text::new("Caption").caption();
        
        assert!(heading.style.font_size > body.style.font_size);
        assert!(body.style.font_size > caption.style.font_size);
    }

    #[test]
    fn test_text_selection() {
        let text = Text::new("Test selection").selectable(true);
        
        assert!(text.is_selectable());
        assert_eq!(text.get_selection(), None);
        
        text.set_selection(Some(0), Some(4));
        assert_eq!(text.get_selection(), Some((0, 4)));
        
        text.clear_selection();
        assert_eq!(text.get_selection(), None);
    }

    #[test]
    fn test_text_builder() {
        let text = TextBuilder::new("Builder Test")
            .heading(2)
            .color(Color::rgba(1.0, 0.0, 0.0, 1.0))
            .selectable(true)
            .build();
            
        assert_eq!(text.content(), "Builder Test");
        assert!(text.is_selectable());
        assert_eq!(text.style.color, Color::rgba(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_text_measurement() {
        let text = Text::new("Test measurement");
        let available = Size::new(200.0, 100.0);
        let size = text.calculate_size(available);
        
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);
        assert!(size.width <= available.width);
        assert!(size.height <= available.height);
    }
}

// Implement Widget trait for Text
impl Widget for Text {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let text_width = self.content.len() as f32 * self.style.font_size * 0.6;
        let text_height = self.style.font_size * self.style.line_height;
        
        let width = text_width.min(constraints.max_width);
        let height = text_height.min(constraints.max_height);
        
        Size::new(width, height)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        if !self.is_visible() {
            return;
        }

        let bounds = layout.bounds();
        
        // Render text
        batch.add_text(
            self.content.clone(),
            (bounds.0, bounds.1),
            Color::rgba(self.style.color.r, self.style.color.g, self.style.color.b, self.style.color.a),
            self.style.font_size,
        );
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        // Handle text events
        EventResult::Ignored
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(Text {
            id: generate_id(),
            content: self.content.clone(),
            spans: self.spans.clone(),
            style: self.style.clone(),
            bounds: Signal::new(self.bounds.get()),
            visible: Signal::new(self.visible.get()),
            selectable: Signal::new(self.selectable.get()),
            selection_start: Signal::new(self.selection_start.get()),
            selection_end: Signal::new(self.selection_end.get()),
            theme: self.theme.clone(),
            measured_size: Signal::new(self.measured_size.get()),
            line_breaks: Signal::new(self.line_breaks.get()),
        })
    }
}
