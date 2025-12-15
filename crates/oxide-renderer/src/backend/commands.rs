use oxide_core::types::{Color, Rect, Transform};

/// High-level rendering commands for the UI engine.
/// These are backend-agnostic and declarative.
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Draw a filled rectangle
    DrawRect {
        rect: Rect,
        color: Color,
        transform: Option<Transform>,
        // TODO: Add border_radius, border_color, border_width
    },
    /// Draw text string
    DrawText {
        text: String,
        position: (f32, f32),
        color: Color,
        // TODO: Add font_id, size, alignment
    },
    /// Push a clipping rectangle
    PushClip(Rect),
    /// Pop the last clipping rectangle
    PopClip,
    /// Set a custom viewport
    SetViewport(Rect),
}
