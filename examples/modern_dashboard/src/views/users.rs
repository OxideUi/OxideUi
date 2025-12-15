use oxide_widgets::{Widget, Container, Text};
use crate::theme::AppTheme;

pub struct UsersView;

impl UsersView {
    pub fn build() -> Box<dyn Widget> {
        let theme = AppTheme::dark();
        Box::new(Container::new()
            .child(Text::new("Users Management").color(theme.text_primary).font_size(32.0))
        )
    }
}
