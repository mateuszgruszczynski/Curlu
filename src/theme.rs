use crate::settings::{Accent, Theme};
use eframe::egui;

// ── Sizing (not theme-dependent) ────────────────────────────────────────────

pub const LINUX_SCALE: f32 = 1.0;
pub const WINDOW_SIZE: [f32; 2] = [1200.0, 780.0];
pub const WINDOW_MIN_SIZE: [f32; 2] = [700.0, 480.0];
pub const SIDE_PANEL_MIN_WIDTH: f32 = 220.0;
/// Send button: [width, height]
pub const BUTTON_SIZE: [f32; 2] = [46.0, 36.0];
/// Method combo: [width, height]
pub const COMBOBOX_SIZE: [f32; 2] = [92.0, 36.0];
pub const URL_INPUT_HEIGHT: f32 = 36.0;
pub const URL_INPUT_MARGIN: egui::Margin = egui::Margin::symmetric(12, 9);
pub const ITEM_SPACING: egui::Vec2 = egui::vec2(7.0, 7.0);

// ── Font sizes ───────────────────────────────────────────────────────────────

pub const FONT_XS: f32 = 10.5;      // section-label caps, tiny annotations
pub const FONT_SM: f32 = 11.0;      // chips, format button, accent picker label
pub const FONT_MD: f32 = 11.5;      // sidebar action buttons, settings labels
pub const FONT_SIDEBAR: f32 = 12.5; // sidebar tree entries

// ── Sidebar tree layout ──────────────────────────────────────────────────────

pub const SIDEBAR_ROW_H: f32 = 22.0;
pub const SIDEBAR_ACCENT_BAR: f32 = 2.0;
pub const SIDEBAR_ARROW_FONT: f32 = 10.0;
pub const SIDEBAR_ARROW_PAD: f32 = 4.0;
pub const SIDEBAR_NAME_PAD: f32 = 18.0;
pub const SIDEBAR_FILE_PAD: f32 = 8.0;

// ── Spacing ──────────────────────────────────────────────────────────────────

pub const CHIP_GAP: f32 = 4.0;

// ── Palette ──────────────────────────────────────────────────────────────────

/// All resolved colors for one theme + accent combination.
#[derive(Clone)]
pub struct Palette {
    /// Darkest bg: title bar, status bar
    pub bg0: egui::Color32,
    /// Mid bg: toolbar, main area
    pub bg1: egui::Color32,
    /// Panel fill: text areas, inputs
    pub bg2: egui::Color32,
    /// Sidebar background
    pub side: egui::Color32,
    pub stroke: egui::Color32,
    pub stroke_soft: egui::Color32,
    pub text: egui::Color32,
    pub dim: egui::Color32,
    pub faint: egui::Color32,
    pub hover: egui::Color32,
    pub ok: egui::Color32,
    pub accent: egui::Color32,
    pub accent_hover: egui::Color32,
    pub accent_text: egui::Color32,
    /// ~15 % accent alpha — used for selected file highlight
    pub accent_sel: egui::Color32,
    /// ok color at ~17 % alpha — status pill background
    pub ok_soft: egui::Color32,
    pub syn_key: egui::Color32,
    pub syn_str: egui::Color32,
    pub syn_num: egui::Color32,
    pub syn_bool: egui::Color32,
    pub syn_null: egui::Color32,
    pub syn_punct: egui::Color32,
    pub syn_plain: egui::Color32,
}

const fn rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

pub fn palette(theme: Theme, accent: Accent) -> Palette {
    let (acc, acc_hov, acc_txt) = match accent {
        Accent::Blue  => (rgb( 59,130,246), rgb( 90,150,248), rgb(255,255,255)),
        Accent::Green => (rgb( 47,170, 87), rgb( 60,192,104), rgb(255,255,255)),
        Accent::Amber => (rgb(223,142, 38), rgb(239,162, 61), rgb( 36, 22,  6)),
    };
    let acc_sel = egui::Color32::from_rgba_unmultiplied(acc.r(), acc.g(), acc.b(), 38);

    match theme {
        Theme::Dark => Palette {
            bg0:         rgb( 21, 23, 28),
            bg1:         rgb( 27, 30, 36),
            bg2:         rgb( 20, 22, 27),
            side:        rgb( 24, 27, 32),
            stroke:      rgb( 43, 49, 59),
            stroke_soft: rgb( 35, 39, 47),
            text:        rgb(214,217,224),
            dim:         rgb(138,144,156),
            faint:       rgb( 91, 97,109),
            hover:       egui::Color32::from_rgba_unmultiplied(200,200,200, 13),
            ok:          rgb( 76,195,138),
            ok_soft:     egui::Color32::from_rgba_unmultiplied( 76,195,138, 43),
            accent: acc, accent_hover: acc_hov, accent_text: acc_txt, accent_sel: acc_sel,
            syn_key:     rgb(122,162,247),
            syn_str:     rgb(158,206,106),
            syn_num:     rgb(224,175,104),
            syn_bool:    rgb(187,154,247),
            syn_null:    rgb(187,154,247),
            syn_punct:   rgb(107,114,128),
            syn_plain:   rgb(138,144,156),
        },
        Theme::Light => Palette {
            bg0:         rgb(232,234,237),
            bg1:         rgb(245,246,248),
            bg2:         rgb(255,255,255),
            side:        rgb(238,240,243),
            stroke:      rgb(215,219,225),
            stroke_soft: rgb(228,231,235),
            text:        rgb( 35, 39, 46),
            dim:         rgb(100,107,118),
            faint:       rgb(154,161,171),
            hover:       egui::Color32::from_rgba_unmultiplied(0, 0, 0, 13),
            ok:          rgb( 46,158, 99),
            ok_soft:     egui::Color32::from_rgba_unmultiplied( 46,158, 99, 43),
            accent: acc, accent_hover: acc_hov, accent_text: acc_txt, accent_sel: acc_sel,
            syn_key:     rgb( 47,111,179),
            syn_str:     rgb( 47,133, 90),
            syn_num:     rgb(176,111, 26),
            syn_bool:    rgb(124, 77,219),
            syn_null:    rgb(124, 77,219),
            syn_punct:   rgb(139,148,160),
            syn_plain:   rgb(100,107,118),
        },
    }
}

// ── Style application ────────────────────────────────────────────────────────

pub fn apply(style: &mut egui::Style, pal: &Palette) {
    style.spacing.item_spacing = ITEM_SPACING;

    let cr4  = egui::CornerRadius::same(4);
    // Hover highlights use 2px radius — consistent with file-selection rectangle highlight
    let cr2  = egui::CornerRadius::same(2);

    // Do NOT set override_text_color — let fg_stroke drive text per widget state.
    // This prevents Amber accent's dark accent_text from leaking into label colors.
    style.visuals.panel_fill           = pal.bg1;
    style.visuals.window_fill          = pal.bg1;
    style.visuals.extreme_bg_color     = pal.bg2;
    style.visuals.window_stroke        = egui::Stroke::new(1.0, pal.stroke);
    style.visuals.window_corner_radius = cr4;
    style.visuals.window_shadow        = egui::Shadow::NONE;
    style.visuals.popup_shadow         = egui::Shadow::NONE;

    // noninteractive — static labels / frames
    style.visuals.widgets.noninteractive.bg_fill       = pal.bg2;
    style.visuals.widgets.noninteractive.weak_bg_fill  = pal.bg1;
    style.visuals.widgets.noninteractive.bg_stroke     = egui::Stroke::new(1.0, pal.stroke);
    style.visuals.widgets.noninteractive.fg_stroke     = egui::Stroke::new(1.0, pal.text);
    style.visuals.widgets.noninteractive.corner_radius = cr4;

    // inactive — buttons/combos at rest
    style.visuals.widgets.inactive.bg_fill       = egui::Color32::TRANSPARENT;
    style.visuals.widgets.inactive.weak_bg_fill  = egui::Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_stroke     = egui::Stroke::NONE;
    style.visuals.widgets.inactive.fg_stroke     = egui::Stroke::new(1.0, pal.text);
    style.visuals.widgets.inactive.corner_radius = cr4;

    // hovered — rectangular-ish highlight, consistent with file-selection style
    style.visuals.widgets.hovered.bg_fill       = pal.hover;
    style.visuals.widgets.hovered.weak_bg_fill  = pal.hover;
    style.visuals.widgets.hovered.bg_stroke     = egui::Stroke::NONE;
    style.visuals.widgets.hovered.fg_stroke     = egui::Stroke::new(1.0, pal.text);
    style.visuals.widgets.hovered.corner_radius = cr2;

    // active / pressed — accent fill; text stays readable in all themes/accents
    style.visuals.widgets.active.bg_fill       = pal.accent;
    style.visuals.widgets.active.weak_bg_fill  = pal.accent;
    style.visuals.widgets.active.bg_stroke     = egui::Stroke::new(1.5, pal.accent);
    style.visuals.widgets.active.fg_stroke     = egui::Stroke::new(1.0, pal.text);
    style.visuals.widgets.active.corner_radius = cr2;

    // open — combo dropdown open / text-edit focused
    style.visuals.widgets.open.bg_fill       = pal.bg2;
    style.visuals.widgets.open.weak_bg_fill  = pal.bg2;
    style.visuals.widgets.open.bg_stroke     = egui::Stroke::new(1.5, pal.accent);
    style.visuals.widgets.open.fg_stroke     = egui::Stroke::new(1.0, pal.text);
    style.visuals.widgets.open.corner_radius = cr4;

    // text selection
    style.visuals.selection.bg_fill = pal.accent_sel;
    style.visuals.selection.stroke  = egui::Stroke::new(1.0, pal.accent);

    // No hover-expansion — flat, stable edges per style guide
    style.visuals.widgets.noninteractive.expansion = 0.0;
    style.visuals.widgets.inactive.expansion       = 0.0;
    style.visuals.widgets.hovered.expansion        = 0.0;
    style.visuals.widgets.active.expansion         = 0.0;
    style.visuals.widgets.open.expansion           = 0.0;

    // Thin scroll bar (~6 px)
    style.spacing.scroll.bar_width = 6.0;
}

// ── Widget helpers ───────────────────────────────────────────────────────────

pub fn url_font() -> egui::FontId {
    egui::FontId::monospace(13.0)
}

/// Send button — filled with accent color.
pub fn send_button(pal: &Palette) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new("\u{25B6}")
            .size(14.0)
            .color(pal.accent_text),
    )
    .fill(pal.accent)
    .corner_radius(egui::CornerRadius::same(4))
}

/// Panel frame: bg2 fill, hairline stroke, 5 px corners.
pub fn panel_frame(pal: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(pal.bg2)
        .stroke(egui::Stroke::new(1.0, pal.stroke))
        .corner_radius(egui::CornerRadius::same(5))
}

// ── Section labels ───────────────────────────────────────────────────────────

/// 2 px accent bar + uppercase label (dim colour).
pub fn section_label(ui: &mut egui::Ui, label: &str, pal: &Palette) {
    section_label_right(ui, label, pal, |_| {});
}

/// Section label with right-aligned extra content drawn inside a closure.
pub fn section_label_right(
    ui: &mut egui::Ui,
    label: &str,
    pal: &Palette,
    right_content: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(2.0, 11.0), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, egui::CornerRadius::same(1), pal.accent);
        }
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(label)
                .size(FONT_XS)
                .strong()
                .color(pal.dim),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            right_content(ui);
        });
    });
}

// ── Status helpers ───────────────────────────────────────────────────────────

pub fn status_color(status: u16, pal: &Palette) -> egui::Color32 {
    match status {
        200..=299 => pal.ok,
        300..=399 => rgb(224,175,104),
        400..=599 => rgb(229, 86, 74),
        _ => pal.dim,
    }
}

pub fn status_label(status: u16) -> String {
    let text = match status {
        200 => "OK",          201 => "Created",      204 => "No Content",
        301 => "Moved",       302 => "Found",         304 => "Not Modified",
        400 => "Bad Request", 401 => "Unauthorized",  403 => "Forbidden",
        404 => "Not Found",   405 => "Method Not Allowed",
        500 => "Server Error",502 => "Bad Gateway",   503 => "Unavailable",
        _ => "",
    };
    if text.is_empty() { format!("{status}") } else { format!("{status} {text}") }
}

pub fn human_bytes(n: usize) -> String {
    if n < 1024 { format!("{n} B") }
    else if n < 1024 * 1024 { format!("{:.1} KB", n as f64 / 1024.0) }
    else { format!("{:.1} MB", n as f64 / (1024.0 * 1024.0)) }
}

// ── Chip widgets (§9) ────────────────────────────────────────────────────────

/// Status pill: filled background (ok-soft / error-soft), dot, monospace label.
pub fn chip_status(ui: &mut egui::Ui, status: u16, pal: &Palette) {
    let (text_col, fill_col) = match status {
        200..=299 => (pal.ok, pal.ok_soft),
        300..=399 => {
            let c = rgb(224, 175, 104);
            (c, egui::Color32::from_rgba_unmultiplied(224, 175, 104, 43))
        }
        400..=599 => {
            let c = rgb(229, 86, 74);
            (c, egui::Color32::from_rgba_unmultiplied(229, 86, 74, 43))
        }
        _ => (pal.dim, egui::Color32::TRANSPARENT),
    };
    egui::Frame::new()
        .fill(fill_col)
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(7, 1))
        .show(ui, |ui| {
                            ui.label(
                    egui::RichText::new(status_label(status))
                        .size(FONT_SM).monospace().color(text_col),
                );
            // ui.horizontal(|ui| {
            //     // ui.spacing_mut().item_spacing.x = 5.0;
            //     // let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
            //     // if ui.is_rect_visible(dot_rect) {
            //     //     ui.painter().circle_filled(dot_rect.center(), 3.0, text_col);
            //     // }

            // });
        });
}

/// Outline chip: transparent fill, 1px stroke, dim monospace text.
pub fn chip_outline(ui: &mut egui::Ui, text: &str, pal: &Palette) {
    egui::Frame::new()
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::new(1.0, pal.stroke))
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(7, 1))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text).size(FONT_SM).monospace().color(pal.dim),
            );
        });
}

/// Secondary outline button for `{ }` prettify (§2c): stroke border, no fill,
/// dim text at rest — stroke + text switch to accent on hover.
pub fn format_button(ui: &mut egui::Ui, pal: &Palette) -> egui::Response {
    ui.scope(|ui| {
        let cr4 = egui::CornerRadius::same(4);
        ui.visuals_mut().widgets.inactive.bg_fill       = egui::Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.weak_bg_fill  = egui::Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.bg_stroke     = egui::Stroke::new(1.0, pal.stroke);
        ui.visuals_mut().widgets.inactive.fg_stroke     = egui::Stroke::new(1.0, pal.dim);
        ui.visuals_mut().widgets.inactive.corner_radius = cr4;
        ui.visuals_mut().widgets.hovered.bg_fill        = egui::Color32::TRANSPARENT;
        ui.visuals_mut().widgets.hovered.weak_bg_fill   = egui::Color32::TRANSPARENT;
        ui.visuals_mut().widgets.hovered.bg_stroke      = egui::Stroke::new(1.0, pal.accent);
        ui.visuals_mut().widgets.hovered.fg_stroke      = egui::Stroke::new(1.0, pal.accent);
        ui.visuals_mut().widgets.hovered.corner_radius  = cr4;
        ui.add(
            egui::Button::new(egui::RichText::new("{ }").size(FONT_SM))
                .corner_radius(cr4),
        )
    })
    .inner
}
