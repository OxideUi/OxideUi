use oxide_core::types::Color;

#[derive(Clone, Copy)]
pub struct AppTheme {
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub border: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl AppTheme {
    pub fn dark() -> Self {
        Self {
            bg_primary: Color::rgba(0.05, 0.05, 0.06, 1.0),   // #0D0D10
            bg_secondary: Color::rgba(0.09, 0.09, 0.11, 1.0), // #17171C
            bg_tertiary: Color::rgba(0.13, 0.13, 0.16, 1.0),  // #212129
            accent: Color::rgba(0.39, 0.35, 0.88, 1.0),       // #6459E1
            accent_hover: Color::rgba(0.45, 0.41, 0.92, 1.0),
            text_primary: Color::rgba(0.95, 0.95, 0.97, 1.0),
            text_secondary: Color::rgba(0.60, 0.60, 0.65, 1.0),
            border: Color::rgba(0.20, 0.20, 0.24, 1.0),
            success: Color::rgba(0.2, 0.8, 0.4, 1.0),
            warning: Color::rgba(0.9, 0.7, 0.2, 1.0),
            error: Color::rgba(0.9, 0.3, 0.3, 1.0),
        }
    }
}

pub const BORDER_RADIUS_LG: f32 = 16.0;
pub const BORDER_RADIUS_MD: f32 = 12.0;
pub const BORDER_RADIUS_SM: f32 = 8.0;

pub const SPACING_XS: f32 = 4.0;
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 16.0;
pub const SPACING_LG: f32 = 24.0;
pub const SPACING_XL: f32 = 32.0;
