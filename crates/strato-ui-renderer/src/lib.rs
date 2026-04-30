//! Strato UI renderer seed.
//!
//! The renderer starts with a deterministic command stream so the clean-room
//! core can be tested without platform-specific code.

use strato_ui_core::{AppContext, Element, Size};

#[derive(Clone, Debug, PartialEq)]
pub enum RenderCommand {
    Text { description: String },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderFrame {
    pub commands: Vec<RenderCommand>,
}

#[derive(Clone, Debug, Default)]
pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, element: &dyn Element, app: &AppContext, size: Size) -> RenderFrame {
        let _layout = element.layout(size, app);
        RenderFrame {
            commands: vec![RenderCommand::Text {
                description: element.describe(),
            }],
        }
    }
}
