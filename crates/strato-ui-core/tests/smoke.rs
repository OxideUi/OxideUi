use strato_ui_core::{text, AppContext, Element, Size};

#[test]
fn creates_basic_element_and_layout() {
    let app = AppContext::new("Strato Smoke");
    let element = text("hello strato");
    let rect = element.layout(Size::new(200.0, 100.0), &app);

    assert_eq!(app.app_name(), "Strato Smoke");
    assert_eq!(element.describe(), "text:hello strato");
    assert!(rect.width > 0.0);
    assert_eq!(rect.height, 20.0);
}
