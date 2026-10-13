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
    /// Primary button fill color
    pub accent_fill: Color32,
    /// Text color on accent-filled widgets
    pub on_accent: Color32,
}

impl ThemeColors {
    /// Get theme colors based on current UI theme (light/dark mode)
    pub fn get(ui: &Ui) -> Self {
        if ui.visuals().dark_mode { Self::dark() } else { Self::light() }
    }

    /// Light mode color palette
    pub const fn light() -> Self {
        Self {
            text: Color32::from_rgb(32, 36, 42),
            text_strong: Color32::from_rgb(0, 0, 0),
            text_weak: Color32::from_rgb(100, 108, 120),
            accent: Color32::from_rgb(0, 102, 204),
            success: Color32::from_rgb(30, 140, 70),
            warning: Color32::from_rgb(190, 120, 0),
            error: Color32::from_rgb(200, 45, 45),
            header: Color32::from_rgb(0, 90, 180),
            bg_light: Color32::from_rgb(255, 255, 255),
            bg_medium: Color32::from_rgb(244, 246, 249),
            bg_dark: Color32::from_rgb(230, 234, 239),
            line_small: Color32::from_rgb(222, 226, 232),
            line_medium: Color32::from_rgb(200, 206, 214),
            line_strong: Color32::from_rgb(165, 172, 182),
            accent_fill: Color32::from_rgb(0, 102, 204),
            on_accent: Color32::WHITE,
        }
    }

    /// Dark mode color palette
    pub const fn dark() -> Self {
        Self {
            text: Color32::from_rgb(220, 224, 230),
            text_strong: Color32::from_rgb(255, 255, 255),
            text_weak: Color32::from_rgb(140, 148, 160),
            accent: Color32::from_rgb(88, 166, 255),
            success: Color32::from_rgb(87, 200, 120),
            warning: Color32::from_rgb(240, 185, 70),
            error: Color32::from_rgb(240, 95, 95),
            header: Color32::from_rgb(88, 166, 255),
            bg_light: Color32::from_rgb(38, 42, 50),
            bg_medium: Color32::from_rgb(27, 30, 36),
            bg_dark: Color32::from_rgb(18, 20, 24),
            line_small: Color32::from_rgb(45, 50, 58),
            line_medium: Color32::from_rgb(60, 66, 76),
            line_strong: Color32::from_rgb(85, 92, 105),
            accent_fill: Color32::from_rgb(45, 115, 210),
            on_accent: Color32::WHITE,
        }
    }

    /// Apply the tramex theme to the egui context for both dark and light mode
    ///
    /// Should be called once at application startup.
    pub fn apply_style(ctx: &egui::Context) {
        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            let colors = match theme {
                egui::Theme::Dark => Self::dark(),
                egui::Theme::Light => Self::light(),
            };
            let dark_mode = theme == egui::Theme::Dark;

            let mut visuals = if dark_mode {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            };

            visuals.panel_fill = colors.bg_medium;
            visuals.window_fill = colors.bg_medium;
            visuals.extreme_bg_color = colors.bg_dark;
            visuals.faint_bg_color = if dark_mode { colors.bg_light } else { colors.bg_medium };
            visuals.code_bg_color = colors.bg_dark;

            visuals.window_stroke = egui::Stroke::new(1.0, colors.line_medium);
            visuals.window_corner_radius = egui::CornerRadius::same(10);
            visuals.menu_corner_radius = egui::CornerRadius::same(8);
            visuals.window_shadow = egui::Shadow {
                offset: [0, 4],
                blur: 16,
                spread: 0,
                color: Color32::from_black_alpha(if dark_mode { 60 } else { 25 }),
            };

            let [ar, ag, ab, _] = colors.accent.to_array();
            visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(ar, ag, ab, 90);
            visuals.selection.stroke = egui::Stroke::new(1.0, colors.accent);
            visuals.hyperlink_color = colors.accent;
            visuals.warn_fg_color = colors.warning;
            visuals.error_fg_color = colors.error;

            let widgets = &mut visuals.widgets;
            let widget_radius = egui::CornerRadius::same(6);
            for state in [
                &mut widgets.noninteractive,
                &mut widgets.inactive,
                &mut widgets.hovered,
                &mut widgets.active,
                &mut widgets.open,
            ] {
                state.corner_radius = widget_radius;
            }
            widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, colors.text);
            widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, colors.line_small);
            widgets.inactive.fg_stroke = egui::Stroke::new(1.0, colors.text);
            widgets.inactive.weak_bg_fill = colors.bg_light;
            widgets.inactive.bg_fill = colors.bg_light;
            let hovered_bg = if dark_mode { colors.line_medium } else { colors.bg_dark };
            widgets.hovered.weak_bg_fill = hovered_bg;
            widgets.hovered.bg_fill = hovered_bg;
            widgets.hovered.bg_stroke = egui::Stroke::new(1.0, colors.accent);
            widgets.hovered.fg_stroke = egui::Stroke::new(1.0, colors.text_strong);
            let active_bg = if dark_mode { colors.line_strong } else { colors.line_medium };
            widgets.active.weak_bg_fill = active_bg;
            widgets.active.bg_fill = active_bg;
            // `Visuals::strong_text_color()` resolves to `widgets.active.fg_stroke`
            widgets.active.fg_stroke = egui::Stroke::new(1.0, colors.text_strong);
            widgets.open.fg_stroke = egui::Stroke::new(1.0, colors.text);

            ctx.set_visuals_of(theme, visuals);
        }

        ctx.all_styles_mut(|style| {
            style.spacing.item_spacing = egui::vec2(6.0, 4.0);
            style.spacing.button_padding = egui::vec2(8.0, 3.0);
            style.spacing.interact_size.y = 20.0;
            style.spacing.window_margin = egui::Margin::same(8);
            style.spacing.menu_margin = egui::Margin::same(6);

            let text_styles = [
                (egui::TextStyle::Heading, 15.0, egui::FontFamily::Proportional),
                (egui::TextStyle::Body, 13.0, egui::FontFamily::Proportional),
                (egui::TextStyle::Button, 13.0, egui::FontFamily::Proportional),
                (egui::TextStyle::Monospace, 12.0, egui::FontFamily::Monospace),
                (egui::TextStyle::Small, 10.5, egui::FontFamily::Proportional),
            ];
            for (text_style, size, family) in text_styles {
                style.text_styles.insert(text_style, egui::FontId::new(size, family));
            }
        });
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
