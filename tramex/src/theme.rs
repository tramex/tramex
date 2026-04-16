//! Theme colors module
//!
//! Centralized color definitions for light and dark modes.
//! Use `ThemeColors::get(ui)` to get theme-appropriate colors.

use egui::{Color32, Ui};

/// Theme-aware color palette
pub struct ThemeColors {
    /// Primary text color
    pub text: Color32,
    /// Strong/emphasized text color
    pub text_strong: Color32,
    /// Weak/secondary text color
    pub text_weak: Color32,
    /// Accent color for headers and highlights
    pub accent: Color32,
    /// Success/positive color
    pub success: Color32,
    /// Warning color
    pub warning: Color32,
    /// Error/negative color
    pub error: Color32,
    /// Panel/section header color
    pub header: Color32,
    /// Light background color
    pub bg_light: Color32,
    /// Medium background color
    pub bg_medium: Color32,
    /// Dark background color
    pub bg_dark: Color32,
    /// Small line color
    pub line_small: Color32,
    /// Medium line color
    pub line_medium: Color32,
    /// Strong line color
    pub line_strong: Color32,
}

impl ThemeColors {
    /// Get theme colors based on current UI theme (light/dark mode)
    pub fn get(ui: &Ui) -> Self {
        if ui.visuals().dark_mode { Self::dark() } else { Self::light() }
    }

    /// Light mode color palette
    pub const fn light() -> Self {
        Self {
            text: Color32::from_rgb(30, 30, 30),
            text_strong: Color32::from_rgb(0, 0, 0),
            text_weak: Color32::from_rgb(100, 100, 100),
            accent: Color32::from_rgb(0, 100, 150),
            success: Color32::from_rgb(40, 160, 40),
            warning: Color32::from_rgb(200, 140, 0),
            error: Color32::from_rgb(200, 50, 50),
            header: Color32::from_rgb(0, 100, 150),
            bg_light: Color32::from_rgb(240, 240, 240),
            bg_medium: Color32::from_rgb(220, 220, 220),
            bg_dark: Color32::from_rgb(200, 200, 200),
            line_small: Color32::from_rgb(200, 200, 200),
            line_medium: Color32::from_rgb(180, 180, 180),
            line_strong: Color32::from_rgb(150, 150, 150),
        }
    }

    /// Dark mode color palette
    pub const fn dark() -> Self {
        Self {
            text: Color32::from_rgb(220, 220, 220),
            text_strong: Color32::from_rgb(255, 255, 255),
            text_weak: Color32::from_rgb(160, 160, 160),
            accent: Color32::from_rgb(100, 180, 255),
            success: Color32::from_rgb(100, 220, 100),
            warning: Color32::from_rgb(255, 200, 80),
            error: Color32::from_rgb(255, 100, 100),
            header: Color32::from_rgb(100, 180, 255),
            bg_light: Color32::from_rgb(60, 60, 60),
            bg_medium: Color32::from_rgb(40, 40, 40),
            bg_dark: Color32::from_rgb(20, 20, 20),
            line_small: Color32::from_rgb(80, 80, 80),
            line_medium: Color32::from_rgb(100, 100, 100),
            line_strong: Color32::from_rgb(120, 120, 120),
        }
    }
}

/// Channel label colors - consistent across themes (colored backgrounds with dark text)
pub struct ChannelColors;

impl ChannelColors {
    /// Red - Broadcast channels
    pub const RED: Color32 = Color32::from_rgb(255, 84, 84);
    /// Blue - Common channels
    pub const BLUE: Color32 = Color32::from_rgb(68, 143, 255);
    /// Orange - Dedicated channels
    pub const ORANGE: Color32 = Color32::from_rgb(255, 181, 68);
    /// Green - Traffic channels
    pub const GREEN: Color32 = Color32::from_rgb(90, 235, 100);
    /// Text color on colored backgrounds (always dark for readability)
    pub const TEXT_ON_COLOR: Color32 = Color32::from_rgb(30, 30, 30);
}

/// Arrow colors for diagrams
pub struct ArrowColors;

impl ArrowColors {
    /// Active/highlighted arrow (green)
    pub const ACTIVE: Color32 = Color32::from_rgb(110, 255, 110);
    /// Secondary highlight (blue)
    pub const HIGHLIGHT: Color32 = Color32::from_rgb(68, 143, 255);
    /// Current selection (bright blue)
    pub const CURRENT: Color32 = Color32::from_rgb(50, 120, 220);
    /// Related items (lighter blue)
    pub const RELATED: Color32 = Color32::from_rgb(130, 180, 240);
}
