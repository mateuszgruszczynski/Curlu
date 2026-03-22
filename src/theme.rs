use eframe::egui;
use egui::Color32;

// Syntax highlighting colors
pub const COLOR_KEY: Color32 = Color32::from_rgb(130, 206, 255);
pub const COLOR_STRING: Color32 = Color32::from_rgb(242, 173, 118);
pub const COLOR_NUMBER: Color32 = Color32::from_rgb(160, 228, 155);
pub const COLOR_KEYWORD: Color32 = Color32::from_rgb(120, 180, 240);
pub const COLOR_PUNCTUATION: Color32 = Color32::from_rgb(180, 185, 195);

// Widget colors
pub const BUTTON_FILL: Color32 = Color32::from_rgb(50, 55, 65);
pub const BUTTON_HOVER_FILL: Color32 = Color32::from_rgb(65, 72, 85);
pub const BUTTON_ACTIVE_FILL: Color32 = Color32::from_rgb(75, 83, 100);
pub const BUTTON_CORNER_RADIUS: u8 = 6;
pub const INPUT_BG: Color32 = Color32::from_rgb(28, 31, 38);
pub const INPUT_STROKE_COLOR: Color32 = Color32::from_rgb(68, 76, 92);

// App background
pub const PANEL_BG: Color32 = Color32::from_rgb(24, 27, 33);

// UI frame colors
pub const FRAME_FILL: Color32 = Color32::from_rgb(28, 31, 38);
pub const FRAME_STROKE_COLOR: Color32 = Color32::from_rgb(68, 76, 92);
pub const FRAME_STROKE_WIDTH: f32 = 1.0;
pub const FRAME_CORNER_RADIUS: u8 = 6;

// Font sizes
pub const FONT_SIZE_DEFAULT: f32 = 12.0;
pub const FONT_SIZE_ICON: f32 = 18.0;

// Element sizes
pub const BUTTON_SIZE: [f32; 2] = [50.0, 50.0];
pub const COMBOBOX_SIZE: [f32; 2] = [100.0, 50.0];
pub const URL_INPUT_HEIGHT: f32 = 50.0;
pub const URL_INPUT_MARGIN: egui::Margin = egui::Margin::symmetric(8, 14);

// Spacing
pub const ITEM_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);

// Window
pub const WINDOW_SIZE: [f32; 2] = [1000.0, 700.0];
pub const WINDOW_MIN_SIZE: [f32; 2] = [600.0, 400.0];

// Side panel
pub const SIDE_PANEL_MIN_WIDTH: f32 = 250.0;

// Scaling
pub const LINUX_SCALE: f32 = 1.0;

/// Apply theme to egui style
pub fn apply(style: &mut egui::Style) {
    style.spacing.item_spacing = ITEM_SPACING;

    let rounding = egui::CornerRadius::same(BUTTON_CORNER_RADIUS);

    // Inactive buttons
    style.visuals.widgets.inactive.weak_bg_fill = BUTTON_FILL;
    style.visuals.widgets.inactive.bg_fill = BUTTON_FILL;
    style.visuals.widgets.inactive.corner_radius = rounding;

    // Hovered buttons
    style.visuals.widgets.hovered.weak_bg_fill = BUTTON_HOVER_FILL;
    style.visuals.widgets.hovered.bg_fill = BUTTON_HOVER_FILL;
    style.visuals.widgets.hovered.corner_radius = rounding;

    // Active/pressed buttons
    style.visuals.widgets.active.weak_bg_fill = BUTTON_ACTIVE_FILL;
    style.visuals.widgets.active.bg_fill = BUTTON_ACTIVE_FILL;
    style.visuals.widgets.active.corner_radius = rounding;

    // Text input backgrounds
    style.visuals.extreme_bg_color = INPUT_BG;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, INPUT_STROKE_COLOR);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, COLOR_KEY);

    // App background
    style.visuals.panel_fill = PANEL_BG;
    style.visuals.window_fill = PANEL_BG;
}

/// Default font for text inputs
pub fn url_font() -> egui::FontId {
    egui::FontId::proportional(FONT_SIZE_DEFAULT)
}

/// Styled RichText with the default font size
pub fn text(s: &str) -> egui::RichText {
    egui::RichText::new(s).size(FONT_SIZE_DEFAULT)
}

/// Styled RichText for button icons
pub fn icon(s: &str) -> egui::RichText {
    egui::RichText::new(s).size(FONT_SIZE_ICON)
}
