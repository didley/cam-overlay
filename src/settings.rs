use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub zoom_level: i32,
    pub shape: String,
    pub flipped: bool,
    pub window_width: u32,
    pub window_height: u32,
    pub fit_mode: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            zoom_level: 1,
            shape: "circle".to_string(),
            flipped: false,
            window_width: 320,
            window_height: 320,
            fit_mode: "cover".to_string(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        confy::load("cam-overlay", None).unwrap_or_default()
    }

    pub fn save(&self) {
        if let Err(e) = confy::store("cam-overlay", None, self) {
            eprintln!("Failed to save settings: {e}");
        }
    }
}
