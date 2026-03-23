use eframe::egui;
use egui::Color32;

/// Convert a hex literal (0xRRGGBB) to Color32 at compile time.
macro_rules! hex {
    ($hex:expr) => {
        Color32::from_rgb(
            (($hex >> 16) & 0xFF) as u8,
            (($hex >> 8) & 0xFF) as u8,
            ($hex & 0xFF) as u8,
        )
    };
}

// Syntax highlighting colors
pub const COLOR_KEY: Color32 = hex!(0x82CEFF);
pub const COLOR_STRING: Color32 = hex!(0xF2AD76);
pub const COLOR_NUMBER: Color32 = hex!(0xA0E49B);
pub const COLOR_KEYWORD: Color32 = hex!(0x78B4F0);
pub const COLOR_PUNCTUATION: Color32 = hex!(0xB4B9C3);

// Generic button colors (matches input fields)
pub const BUTTON_FILL: Color32 = hex!(0x2f2f2f);
pub const BUTTON_HOVER_FILL: Color32 = hex!(0x3a3a3a);
pub const BUTTON_ACTIVE_FILL: Color32 = hex!(0x454545);
pub const BUTTON_CORNER_RADIUS: u8 = 6;

// Send button colors (lime green)
pub const SEND_BUTTON_FILL: Color32 = hex!(0x4CAF50);
pub const SEND_BUTTON_TEXT: Color32 = hex!(0x1f1f1f);

// Input field colors
pub const INPUT_BG: Color32 = hex!(0x2f2f2f);
pub const INPUT_STROKE_COLOR: Color32 = hex!(0x3f3f3f);

// App background
pub const PANEL_BG: Color32 = hex!(0x1f1f1f);
pub const MENU_BAR_BG: Color32 = hex!(0x1a1a1a);

// UI frame colors
pub const FRAME_FILL: Color32 = hex!(0x2f2f2f);
pub const FRAME_STROKE_COLOR: Color32 = hex!(0x3f3f3f);
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

// Toolbar offset for URL width calculation
pub const URL_WIDTH_OFFSET: f32 = 30.0;

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

/// Send button with lime green styling
pub fn send_button() -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new("\u{25B6}")
            .size(FONT_SIZE_ICON)
            .color(SEND_BUTTON_TEXT),
    )
    .fill(SEND_BUTTON_FILL)
}
