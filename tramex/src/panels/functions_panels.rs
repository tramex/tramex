//! Panels functions

use egui::{Color32, TextFormat};

/// Custom label color
pub enum CustomLabelColor {
    /// Red color
    Red,

    /// Blue color
    Blue,

    /// Orange color
    Orange,

    /// Green color
    Green,

    /// White color
    White,
}

/// Print a label on the grid
pub fn make_label_equal(ui: &mut egui::Ui, label: &str, state: &str, color: CustomLabelColor) {
    make_label(ui, label, label == state, color);
}

/// Create a label with a background color
pub fn make_label(ui: &mut egui::Ui, label: &str, show: bool, color: CustomLabelColor) {
    use egui::text::LayoutJob;
    let mut job = LayoutJob::default();
    let (default_color, _strong_color) = (Color32::BLACK, Color32::BLACK);
    let background = if show {
        match color {
            CustomLabelColor::Red => Color32::from_rgb(255, 84, 84),
            CustomLabelColor::Blue => Color32::from_rgb(68, 143, 255),
            CustomLabelColor::Orange => Color32::from_rgb(255, 181, 68),
            CustomLabelColor::Green => Color32::from_rgb(90, 235, 100),
            CustomLabelColor::White => Color32::from_rgb(255, 255, 255),
        }
    } else {
        Color32::from_rgb(255, 255, 255)
    };

    job.append(
        label,
        0.0,
        TextFormat {
            color: default_color,
            background,
            ..Default::default()
        },
    );
    ui.vertical_centered(|ui| {
        ui.label(job);
    });
}

/// Arrow direction
#[derive(Debug)]
pub enum ArrowDirection {
    /// Up arrow
    Up,

    /// Down arrow
    Down,
}

/// Arrow color
#[derive(Debug)]
pub enum ArrowColor {
    /// Green arrow
    Green,

    /// Blue arrow
    Blue,

    /// Black arrow
    Black,
}

/// Create an arrow
pub fn make_arrow(ui: &mut egui::Ui, direction: ArrowDirection, color: ArrowColor, font_id: &egui::FontId) {
    // ↑↓
    // ⇑⇓
    // ⇡⇣ chosen
    // ⮉⮋
    // ⬆⬇
    // ⇧⇩
    let content = match direction {
        ArrowDirection::Down => "⇣",
        ArrowDirection::Up => "⇡",
    };
    let current_color = match color {
        ArrowColor::Green => Color32::from_rgb(110, 255, 110),
        ArrowColor::Blue => Color32::from_rgb(68, 143, 255),
        ArrowColor::Black => Color32::from_rgb(0, 0, 0),
    };

    ui.label(egui::RichText::new(content).color(current_color).font(font_id.clone()));
}
