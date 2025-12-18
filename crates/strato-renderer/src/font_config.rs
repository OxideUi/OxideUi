use cosmic_text::{fontdb, FontSystem as CosmicFontSystem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use sys_locale::get_locale;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub safe_fonts: HashMap<String, Vec<String>>,
    pub fallback_fonts: Vec<String>,
}

impl Default for FontConfig {
    fn default() -> Self {
        let mut safe_fonts = HashMap::new();

        // Windows safe fonts
        safe_fonts.insert(
            "windows".to_string(),
            vec![
                "segoeui.ttf".to_string(),
                "arial.ttf".to_string(),
                "tahoma.ttf".to_string(),
                "calibri.ttf".to_string(),
                "verdana.ttf".to_string(),
            ],
        );

        // macOS safe fonts
        safe_fonts.insert(
            "macos".to_string(),
            vec![
                "SF-Pro-Display-Regular.otf".to_string(),
                "HelveticaNeue.ttc".to_string(),
                "Arial.ttf".to_string(),
                "Helvetica.ttc".to_string(),
            ],
        );

        // Linux safe fonts
        safe_fonts.insert(
            "linux".to_string(),
            vec![
                "DejaVuSans.ttf".to_string(),
                "LiberationSans-Regular.ttf".to_string(),
                "NotoSans-Regular.ttf".to_string(),
                "Ubuntu-R.ttf".to_string(),
            ],
        );

        Self {
            safe_fonts,
            fallback_fonts: vec![
                "sans-serif".to_string(),
                "Arial".to_string(),
                "Helvetica".to_string(),
            ],
        }
    }
}

impl FontConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: FontConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_platform_fonts(&self) -> Vec<String> {
        let platform = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };

        self.safe_fonts
            .get(platform)
            .cloned()
            .unwrap_or_else(|| self.fallback_fonts.clone())
    }
}

/// Creates a safe font system that loads only specific fonts to avoid problematic system fonts
pub fn create_safe_font_system() -> CosmicFontSystem {
    let config = FontConfig::load_from_file("safe_fonts.json").unwrap_or_else(|_| {
        log::warn!("Could not load safe_fonts.json, using default configuration");
        FontConfig::default()
    });

    let mut db = fontdb::Database::new();

    if cfg!(target_os = "windows") {
        // Load only safe fonts on Windows to avoid problematic fonts like mstmc.ttf
        let safe_fonts = config.get_platform_fonts();
        let system_fonts_dir = std::env::var("WINDIR")
            .map(|windir| format!("{}\\Fonts", windir))
            .unwrap_or_else(|_| "C:\\Windows\\Fonts".to_string());

        for font_name in safe_fonts {
            let font_path = format!("{}\\{}", system_fonts_dir, font_name);
            if std::path::Path::new(&font_path).exists() {
                db.load_font_file(&font_path).ok();
            }
        }

        // If no fonts were loaded, fall back to a minimal set
        if db.faces().count() == 0 {
            log::warn!("No safe fonts found, loading minimal fallback fonts");
            db.load_font_file(format!("{}\\arial.ttf", system_fonts_dir))
                .ok();
            db.load_font_file(format!("{}\\tahoma.ttf", system_fonts_dir))
                .ok();
        }
    } else {
        // On macOS and Linux, load system fonts normally
        db.load_system_fonts();

        // Explicitly load emoji font on macOS to ensure it's available
        if cfg!(target_os = "macos") {
            let emoji_path = "/System/Library/Fonts/Apple Color Emoji.ttc";
            if std::path::Path::new(emoji_path).exists() {
                db.load_font_file(emoji_path).ok();
            }
        }
    }

    let locale = get_locale().unwrap_or_else(|| "en-US".to_string());
    CosmicFontSystem::new_with_locale_and_db(locale, db)
}
