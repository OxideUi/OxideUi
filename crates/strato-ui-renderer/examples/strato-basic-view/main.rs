use strato_ui_core::{text, AppContext, Size};
use strato_ui_renderer::{RenderCommand, Renderer};

fn main() {
    let app = AppContext::new("Strato UI Example");
    let view = text("Strato UI is the core");
    let frame = Renderer::new().render(&view, &app, Size::new(320.0, 200.0));

    for command in frame.commands {
        match command {
            RenderCommand::Text { description } => println!("{description}"),
        }
    }
}
