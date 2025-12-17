use std::time::Duration;
use strato_core::inspector::{inspector, InspectorConfig};
use strato_core::state::Signal;
use strato_core::types::Color;
use strato_platform::{ApplicationBuilder, WindowBuilder};
use strato_widgets::animation::{Curve, KeyframeAnimation, Parallel, Timeline, Tween};
use strato_widgets::prelude::*;
use strato_widgets::scroll_view::ScrollView;
use strato_widgets::InspectorOverlay;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    inspector().configure(InspectorConfig {
        enabled: true,
        ..Default::default()
    });

    // Create state for animation targets
    let box_color = Signal::new(Color::RED);
    let box_width = Signal::new(100.0);

    // Create a ScrollView content
    let mut list = Column::new().spacing(10.0);
    for i in 0..20 {
        list = list.child(Box::new(
            Container::new()
                .padding(20.0)
                .background(if i % 2 == 0 {
                    Color::rgba(0.2, 0.2, 0.2, 1.0)
                } else {
                    Color::rgba(0.3, 0.3, 0.3, 1.0)
                })
                .child(Text::new(format!("Scroll Item {}", i)).color(Color::WHITE)),
        ));
    }

    let scroll_view = ScrollView::new(list);

    // Setup animation
    let mut timeline = Timeline::new();

    // Animation 1: Change color Red -> Blue
    let color_anim = KeyframeAnimation::new(
        Duration::from_secs(2),
        Tween::new(Color::RED, Color::BLUE),
        box_color.clone(),
    )
    .with_curve(Curve::EaseInOut);

    // Animation 2: Change width 100 -> 300
    let width_anim = KeyframeAnimation::new(
        Duration::from_secs(2),
        Tween::new(100.0, 300.0),
        box_width.clone(),
    )
    .with_curve(Curve::EaseOut);

    // Run parallel
    let parallel = Parallel::new(vec![Box::new(color_anim), Box::new(width_anim)]);

    timeline.add(parallel);
    timeline.play();

    ApplicationBuilder::new()
        .title("Animated Showcase")
        .window(WindowBuilder::new().with_size(1024.0, 768.0))
        .run(InspectorOverlay::new(
            Container::new().child(
                Row::new().child(Box::new(scroll_view)).child(Box::new(
                    Container::new()
                        .width(300.0)
                        .background(Color::BLACK)
                        .child(Text::new("Animation Placeholder")),
                )),
            ),
        ));
}
