//! Image widget for displaying images in OxideUI applications
//!
//! Supports various image formats, scaling modes, and loading states.

use oxide_core::{
    event::{Event, EventResult},
    layout::{Constraints, Size, Layout},
    types::{Rect, Color},
    vdom::VNode,
};
use oxide_renderer::batch::RenderBatch;
use crate::widget::{Widget, WidgetId, WidgetContext, generate_id};
use std::path::PathBuf;
use std::sync::Arc;

/// Image scaling modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFit {
    /// Fill the entire container, may crop the image
    Fill,
    /// Fit the image within the container, maintaining aspect ratio
    Contain,
    /// Cover the entire container, maintaining aspect ratio, may crop
    Cover,
    /// Scale down to fit if larger, otherwise display at original size
    ScaleDown,
    /// Display at original size
    None,
}

/// Image loading state
#[derive(Debug, Clone, PartialEq)]
pub enum ImageState {
    Loading,
    Loaded(ImageData),
    Error(String),
}

/// Image data representation
#[derive(Debug, Clone, PartialEq)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Arc<Vec<u8>>,
    pub format: ImageFormat,
}

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Svg,
    Bmp,
}

/// Image source types
#[derive(Debug, Clone)]
pub enum ImageSource {
    /// Load from file path
    File(PathBuf),
    /// Load from URL
    Url(String),
    /// Use embedded data
    Data(ImageData),
    /// Use placeholder
    Placeholder { width: u32, height: u32, color: Color },
}

/// Image widget styling
#[derive(Debug, Clone)]
pub struct ImageStyle {
    pub fit: ImageFit,
    pub border_radius: f32,
    pub opacity: f32,
    pub filter: ImageFilter,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f32,
}

impl Default for ImageStyle {
    fn default() -> Self {
        Self {
            fit: ImageFit::Contain,
            border_radius: 0.0,
            opacity: 1.0,
            filter: ImageFilter::None,
            background_color: None,
            border_color: None,
            border_width: 0.0,
        }
    }
}

/// Image filters
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFilter {
    None,
    Blur(f32),
    Brightness(f32),
    Contrast(f32),
    Grayscale(f32),
    Sepia(f32),
    Saturate(f32),
    HueRotate(f32),
    Invert(f32),
}

/// Image widget
pub struct Image {
    id: WidgetId,
    source: ImageSource,
    style: ImageStyle,
    state: ImageState,
    alt_text: Option<String>,
    on_load: Option<Box<dyn Fn(&ImageData) + Send + Sync>>,
    on_error: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    loading_placeholder: Option<VNode>,
    error_placeholder: Option<VNode>,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("id", &self.id)
            .field("source", &self.source)
            .field("style", &self.style)
            .field("state", &self.state)
            .field("alt_text", &self.alt_text)
            .field("on_load", &self.on_load.as_ref().map(|_| "Fn(&ImageData)"))
            .field("on_error", &self.on_error.as_ref().map(|_| "Fn(&str)"))
            .field("on_click", &self.on_click.as_ref().map(|_| "Fn()"))
            .field("loading_placeholder", &self.loading_placeholder)
            .field("error_placeholder", &self.error_placeholder)
            .finish()
    }
}

impl Clone for Image {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            source: self.source.clone(),
            style: self.style.clone(),
            state: self.state.clone(),
            alt_text: self.alt_text.clone(),
            on_load: None, // Function pointers can't be cloned
            on_error: None,
            on_click: None,
            loading_placeholder: self.loading_placeholder.clone(),
            error_placeholder: self.error_placeholder.clone(),
        }
    }
}

impl Image {
    /// Create a new image widget
    pub fn new(source: ImageSource) -> Self {
        Self {
            id: generate_id(),
            source,
            style: ImageStyle::default(),
            state: ImageState::Loading,
            alt_text: None,
            on_load: None,
            on_error: None,
            on_click: None,
            loading_placeholder: None,
            error_placeholder: None,
        }
    }

    /// Create image from file path
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Self {
        Self::new(ImageSource::File(path.into()))
    }

    /// Create image from URL
    pub fn from_url<S: Into<String>>(url: S) -> Self {
        Self::new(ImageSource::Url(url.into()))
    }

    /// Create image from data
    pub fn from_data(data: ImageData) -> Self {
        Self::new(ImageSource::Data(data))
    }

    /// Create placeholder image
    pub fn placeholder(width: u32, height: u32, color: Color) -> Self {
        Self::new(ImageSource::Placeholder { width, height, color })
    }

    /// Set image fit mode
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.style.fit = fit;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.style.border_radius = radius;
        self
    }

    /// Set opacity
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.style.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set image filter
    pub fn filter(mut self, filter: ImageFilter) -> Self {
        self.style.filter = filter;
        self
    }

    /// Set background color
    pub fn background_color(mut self, color: Color) -> Self {
        self.style.background_color = Some(color);
        self
    }

    /// Set border
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.style.border_width = width;
        self.style.border_color = Some(color);
        self
    }

    /// Set alt text for accessibility
    pub fn alt_text<S: Into<String>>(mut self, text: S) -> Self {
        self.alt_text = Some(text.into());
        self
    }

    /// Set load callback
    pub fn on_load<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ImageData) + Send + Sync + 'static,
    {
        self.on_load = Some(Box::new(callback));
        self
    }

    /// Set error callback
    pub fn on_error<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_error = Some(Box::new(callback));
        self
    }

    /// Set click callback
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(callback));
        self
    }

    /// Set loading placeholder
    pub fn loading_placeholder(mut self, placeholder: VNode) -> Self {
        self.loading_placeholder = Some(placeholder);
        self
    }

    /// Set error placeholder
    pub fn error_placeholder(mut self, placeholder: VNode) -> Self {
        self.error_placeholder = Some(placeholder);
        self
    }

    /// Get current image state
    pub fn state(&self) -> &ImageState {
        &self.state
    }

    /// Load image from source
    pub fn load_image(&mut self) {
        let source = self.source.clone();
        match source {
            ImageSource::File(path) => {
                self.load_from_file(&path);
            }
            ImageSource::Url(url) => {
                self.load_from_url(&url);
            }
            ImageSource::Data(data) => {
                self.state = ImageState::Loaded(data.clone());
                if let Some(callback) = &self.on_load {
                    callback(&data);
                }
            }
            ImageSource::Placeholder { width, height, color } => {
                let data = self.create_placeholder_data(width, height, color);
                self.state = ImageState::Loaded(data.clone());
                if let Some(callback) = &self.on_load {
                    callback(&data);
                }
            }
        }
    }

    fn load_from_file(&mut self, path: &PathBuf) {
        // In a real implementation, this would use an image loading library
        // For now, we'll simulate loading
        match std::fs::read(path) {
            Ok(bytes) => {
                if let Ok(data) = self.decode_image_data(bytes) {
                    self.state = ImageState::Loaded(data.clone());
                    if let Some(callback) = &self.on_load {
                        callback(&data);
                    }
                } else {
                    let error = "Failed to decode image".to_string();
                    self.state = ImageState::Error(error.clone());
                    if let Some(callback) = &self.on_error {
                        callback(&error);
                    }
                }
            }
            Err(e) => {
                let error = format!("Failed to load image: {}", e);
                self.state = ImageState::Error(error.clone());
                if let Some(callback) = &self.on_error {
                    callback(&error);
                }
            }
        }
    }

    fn load_from_url(&mut self, _url: &str) {
        // In a real implementation, this would make an HTTP request
        // For now, we'll simulate an error
        let error = "URL loading not implemented".to_string();
        self.state = ImageState::Error(error.clone());
        if let Some(callback) = &self.on_error {
            callback(&error);
        }
    }

    fn decode_image_data(&self, _bytes: Vec<u8>) -> Result<ImageData, String> {
        // In a real implementation, this would use an image decoding library
        // For now, we'll create a dummy image
        Ok(ImageData {
            width: 100,
            height: 100,
            data: Arc::new(vec![255; 100 * 100 * 4]), // White RGBA
            format: ImageFormat::Png,
        })
    }

    fn create_placeholder_data(&self, width: u32, height: u32, color: Color) -> ImageData {
        let pixel_count = (width * height) as usize;
        let mut data = Vec::with_capacity(pixel_count * 4);
        
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        
        for _ in 0..pixel_count {
            data.extend_from_slice(&[r, g, b, a]);
        }
        
        ImageData {
            width,
            height,
            data: Arc::new(data),
            format: ImageFormat::Png,
        }
    }

    fn calculate_display_size(&self, container_size: Size, image_size: Size) -> (Size, Rect) {
        match self.style.fit {
            ImageFit::Fill => {
                (container_size, Rect::new(0.0, 0.0, container_size.width, container_size.height))
            }
            ImageFit::Contain => {
                let scale = (container_size.width / image_size.width)
                    .min(container_size.height / image_size.height);
                let scaled_width = image_size.width * scale;
                let scaled_height = image_size.height * scale;
                let x = (container_size.width - scaled_width) / 2.0;
                let y = (container_size.height - scaled_height) / 2.0;
                (Size::new(scaled_width, scaled_height), Rect::new(x, y, scaled_width, scaled_height))
            }
            ImageFit::Cover => {
                let scale = (container_size.width / image_size.width)
                    .max(container_size.height / image_size.height);
                let scaled_width = image_size.width * scale;
                let scaled_height = image_size.height * scale;
                let x = (container_size.width - scaled_width) / 2.0;
                let y = (container_size.height - scaled_height) / 2.0;
                (Size::new(scaled_width, scaled_height), Rect::new(x, y, scaled_width, scaled_height))
            }
            ImageFit::ScaleDown => {
                if image_size.width <= container_size.width && image_size.height <= container_size.height {
                    let x = (container_size.width - image_size.width) / 2.0;
                    let y = (container_size.height - image_size.height) / 2.0;
                    (image_size, Rect::new(x, y, image_size.width, image_size.height))
                } else {
                    self.calculate_display_size(container_size, image_size) // Use contain logic
                }
            }
            ImageFit::None => {
                let x = (container_size.width - image_size.width) / 2.0;
                let y = (container_size.height - image_size.height) / 2.0;
                (image_size, Rect::new(x, y, image_size.width, image_size.height))
            }
        }
    }
}

impl Widget for Image {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::MouseMove(_mouse_event) => {
                // Handle mouse move for hover effects
                EventResult::Handled
            }
            Event::MouseDown(_mouse_event) => {
                // Handle mouse down for click effects
                if let Some(ref on_click) = self.on_click {
                    on_click();
                }
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    fn update(&mut self, _context: &WidgetContext) {
        // Update widget state if needed
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        match &self.state {
            ImageState::Loaded(data) => {
                let image_size = Size::new(data.width as f32, data.height as f32);
                let container_size = Size::new(constraints.max_width, constraints.max_height);
                let (display_size, _) = self.calculate_display_size(container_size, image_size);
                Size::new(
                    display_size.width.min(constraints.max_width),
                    display_size.height.min(constraints.max_height),
                )
            }
            _ => Size::new(constraints.max_width, constraints.max_height),
        }
    }

    fn render(&self, _batch: &mut RenderBatch, _layout: Layout) {
        // Rendering is handled by the platform layer
        // TODO: Implement actual image rendering
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(Image {
            id: self.id,
            source: self.source.clone(),
            style: self.style.clone(),
            state: self.state.clone(),
            alt_text: self.alt_text.clone(),
            on_load: None, // Cannot clone function pointers
            on_error: None, // Cannot clone function pointers
            on_click: None, // Cannot clone function pointers
            loading_placeholder: self.loading_placeholder.clone(),
            error_placeholder: self.error_placeholder.clone(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Image builder for fluent API
pub struct ImageBuilder {
    image: Image,
}

impl ImageBuilder {
    pub fn new(source: ImageSource) -> Self {
        Self {
            image: Image::new(source),
        }
    }

    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.image = self.image.fit(fit);
        self
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.image = self.image.border_radius(radius);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.image = self.image.opacity(opacity);
        self
    }

    pub fn filter(mut self, filter: ImageFilter) -> Self {
        self.image = self.image.filter(filter);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.image = self.image.background_color(color);
        self
    }

    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.image = self.image.border(width, color);
        self
    }

    pub fn alt_text<S: Into<String>>(mut self, text: S) -> Self {
        self.image = self.image.alt_text(text);
        self
    }

    pub fn on_load<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ImageData) + Send + Sync + 'static,
    {
        self.image = self.image.on_load(callback);
        self
    }

    pub fn on_error<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.image = self.image.on_error(callback);
        self
    }

    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.image = self.image.on_click(callback);
        self
    }

    pub fn loading_placeholder(mut self, placeholder: VNode) -> Self {
        self.image = self.image.loading_placeholder(placeholder);
        self
    }

    pub fn error_placeholder(mut self, placeholder: VNode) -> Self {
        self.image = self.image.error_placeholder(placeholder);
        self
    }

    pub fn build(mut self) -> Image {
        self.image.load_image();
        self.image
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_core::types::Color;

    #[test]
    fn test_image_creation() {
        let image = Image::from_file("test.png");
        assert!(matches!(image.source, ImageSource::File(_)));
        assert_eq!(image.style.fit, ImageFit::Contain);
    }

    #[test]
    fn test_image_builder() {
        let image = ImageBuilder::new(ImageSource::Url("https://example.com/image.png".to_string()))
            .fit(ImageFit::Cover)
            .opacity(0.8)
            .border_radius(10.0)
            .alt_text("Test image")
            .build();

        assert_eq!(image.style.fit, ImageFit::Cover);
        assert_eq!(image.style.opacity, 0.8);
        assert_eq!(image.style.border_radius, 10.0);
        assert_eq!(image.alt_text, Some("Test image".to_string()));
    }

    #[test]
    fn test_placeholder_image() {
        let color = Color::rgba(1.0, 0.0, 0.0, 1.0); // Red
        let image = Image::placeholder(100, 100, color);
        
        if let ImageSource::Placeholder { width, height, color: c } = image.source {
            assert_eq!(width, 100);
            assert_eq!(height, 100);
            assert_eq!(c, color);
        } else {
            panic!("Expected placeholder source");
        }
    }

    #[test]
    fn test_image_fit_calculations() {
        let image = Image::from_file("test.png");
        let container_size = Size::new(200.0, 100.0);
        let image_size = Size::new(100.0, 100.0);

        let (display_size, rect) = image.calculate_display_size(container_size, image_size);
        
        // For contain fit, should maintain aspect ratio
        assert!(display_size.width <= container_size.width);
        assert!(display_size.height <= container_size.height);
    }

    #[test]
    fn test_image_filters() {
        let image = Image::from_file("test.png")
            .filter(ImageFilter::Blur(5.0));
        
        assert!(matches!(image.style.filter, ImageFilter::Blur(5.0)));
    }
}