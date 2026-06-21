use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::highlight;
use crate::http::{self, Method, Response, SavedRequest};
use crate::settings::{Accent, Settings, Theme};
use crate::theme::{self, Palette};

/// A TextBuffer wrapper that allows selection/copy but silently ignores all edits.
struct ReadOnlyBuf<'a>(&'a str);

impl egui::TextBuffer for ReadOnlyBuf<'_> {
    fn is_mutable(&self) -> bool { false }
    fn as_str(&self) -> &str { self.0 }
    fn insert_text(&mut self, _text: &str, _char_index: usize) -> usize { 0 }
    fn delete_char_range(&mut self, _char_range: std::ops::Range<usize>) {}
}

// ── Directory tree ────────────────────────────────────────────────────────────

enum DirEntry {
    File { name: String, path: PathBuf },
    Dir  { name: String, path: PathBuf, children: Vec<DirEntry> },
}

impl DirEntry {
    fn scan(dir: &std::path::Path) -> Vec<DirEntry> {
        let mut entries: Vec<_> = match std::fs::read_dir(dir) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
            Err(_) => return Vec::new(),
        };
        entries.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));
        entries.into_iter().filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                Some(DirEntry::Dir { name, path: path.clone(), children: Self::scan(&path) })
            } else if path.extension().is_some_and(|ext| ext == "curl") {
                Some(DirEntry::File { name, path })
            } else {
                None
            }
        }).collect()
    }

    fn show(entries: &[DirEntry], ui: &mut egui::Ui, file_to_load: &mut Option<PathBuf>, current: &Option<PathBuf>, pal: &Palette) {
        ui.spacing_mut().item_spacing.y = 0.0;
        for entry in entries {
            match entry {
                DirEntry::Dir { name, path, children } => {
                    let id = ui.make_persistent_id(path.display().to_string());
                    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), theme::SIDEBAR_ROW_H),
                        egui::Sense::click(),
                    );
                    if response.clicked() {
                        state.toggle(ui);
                    }
                    if response.hovered() {
                        ui.painter().rect_filled(rect, egui::CornerRadius::same(0), pal.hover);
                    }
                    let arrow = if state.is_open() { "▾" } else { "▸" };
                    let arrow_galley = ui.painter().layout_no_wrap(
                        arrow.to_string(),
                        egui::FontId::proportional(theme::SIDEBAR_ARROW_FONT),
                        pal.dim,
                    );
                    ui.painter().galley(
                        egui::pos2(rect.min.x + theme::SIDEBAR_ARROW_PAD, rect.center().y - arrow_galley.size().y / 2.0),
                        arrow_galley,
                        pal.dim,
                    );
                    let name_galley = ui.painter().layout_no_wrap(
                        name.to_string(),
                        egui::FontId::proportional(theme::FONT_SIDEBAR),
                        pal.text,
                    );
                    ui.painter().galley(
                        egui::pos2(rect.min.x + theme::SIDEBAR_NAME_PAD, rect.center().y - name_galley.size().y / 2.0),
                        name_galley,
                        pal.text,
                    );
                    state.show_body_indented(&response, ui, |ui| {
                        Self::show(children, ui, file_to_load, current, pal);
                    });
                }
                DirEntry::File { name, path } => {
                    let selected = current.as_deref() == Some(path.as_path());
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), theme::SIDEBAR_ROW_H),
                        egui::Sense::click(),
                    );
                    if selected {
                        ui.painter().rect_filled(rect, egui::CornerRadius::same(0), pal.accent_sel);
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(rect.min, egui::vec2(theme::SIDEBAR_ACCENT_BAR, rect.height())),
                            egui::CornerRadius::ZERO,
                            pal.accent,
                        );
                    } else if response.hovered() {
                        ui.painter().rect_filled(rect, egui::CornerRadius::same(0), pal.hover);
                    }
                    let text_color = if selected { pal.accent } else { pal.text };
                    let galley = ui.painter().layout_no_wrap(
                        name.to_string(),
                        egui::FontId::proportional(theme::FONT_SIDEBAR),
                        text_color,
                    );
                    let text_pos = egui::pos2(
                        rect.min.x + theme::SIDEBAR_FILE_PAD,
                        rect.center().y - galley.size().y / 2.0,
                    );
                    ui.painter().galley(text_pos, galley, text_color);
                    if response.clicked() {
                        *file_to_load = Some(path.clone());
                    }
                }
            }
        }
    }

    fn has_match(entries: &[DirEntry], filter: &str) -> bool {
        let f = filter.to_lowercase();
        entries.iter().any(|e| match e {
            DirEntry::File { name, .. } => name.to_lowercase().contains(&f),
            DirEntry::Dir  { children, .. } => Self::has_match(children, filter),
        })
    }

    fn show_filtered(entries: &[DirEntry], ui: &mut egui::Ui, file_to_load: &mut Option<PathBuf>, filter: &str, current: &Option<PathBuf>, pal: &Palette) {
        let f = filter.to_lowercase();
        for entry in entries {
            match entry {
                DirEntry::Dir { name, path, children } => {
                    if Self::has_match(children, filter) {
                        egui::CollapsingHeader::new(name)
                            .id_salt(format!("f_{}", path.display()))
                            .default_open(true)
                            .show(ui, |ui| Self::show_filtered(children, ui, file_to_load, filter, current, pal));
                    }
                }
                DirEntry::File { name, path } => {
                    if name.to_lowercase().contains(&f) {
                        let selected = current.as_deref() == Some(path.as_path());
                        let label = egui::RichText::new(name)
                            .size(theme::FONT_SIDEBAR)
                            .color(if selected { pal.accent } else { pal.text });
                        if ui.selectable_label(false, label).clicked() {
                            *file_to_load = Some(path.clone());
                        }
                    }
                }
            }
        }
    }
}

// ── Column rendering ──────────────────────────────────────────────────────────

struct ColumnConfig<'a> {
    header_label: &'a str,
    body_label: &'a str,
    headers_text: &'a mut String,
    editable: bool,
    headers_hint: &'a str,
    body_text: &'a mut String,
    body_hint: &'a str,
    header_scroll_id: &'a str,
    body_scroll_id: &'a str,
    header_layouter: &'a mut dyn FnMut(&egui::Ui, &str, f32) -> std::sync::Arc<egui::Galley>,
    body_layouter: &'a mut dyn FnMut(&egui::Ui, &str, f32) -> std::sync::Arc<egui::Galley>,
    show_format_button: bool,
    json_path_filter: Option<&'a mut String>,
    /// (status, elapsed_ms, body_bytes) — shown as chips next to the header section label
    response_info: Option<(u16, f64, usize)>,
}

fn render_column(ui: &mut egui::Ui, headers_height: f32, mut cfg: ColumnConfig<'_>, pal: &Palette) {
    let frame = theme::panel_frame(pal);

    // ── Headers section ───────────────────────────────────────────────────
    theme::section_label_right(ui, cfg.header_label, pal, |ui| {
        if let Some((status, elapsed_ms, body_bytes)) = cfg.response_info {
            if status > 0 {
                // Right-to-left: first call → rightmost chip
                theme::chip_outline(ui, &theme::human_bytes(body_bytes), pal);
                ui.add_space(theme::CHIP_GAP);
                theme::chip_outline(ui, &format!("{:.1} ms", elapsed_ms), pal);
                ui.add_space(theme::CHIP_GAP);
                theme::chip_status(ui, status, pal);
            }
        }
    });

    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, headers_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            frame.show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(cfg.header_scroll_id)
                    .show(ui, |ui| {
                        if cfg.editable {
                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(cfg.headers_text)
                                    .font(egui::TextStyle::Monospace)
                                    .hint_text(cfg.headers_hint)
                                    .layouter(cfg.header_layouter)
                                    .frame(false),
                            );
                        } else {
                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(&mut ReadOnlyBuf(cfg.headers_text))
                                    .font(egui::TextStyle::Monospace)
                                    .layouter(cfg.header_layouter)
                                    .frame(false),
                            );
                        }
                    });
            });
        },
    );

    // ── Body section ──────────────────────────────────────────────────────
    ui.add_space(theme::CHIP_GAP);

    // Snapshot path string before any mutable borrow (clones once, no borrow held)
    let path_clone: Option<String> = cfg.json_path_filter.as_ref().map(|p| (**p).clone());

    // Body section label with right-aligned toolbar content
    let body_label = cfg.body_label;
    let show_format = cfg.show_format_button;
    theme::section_label_right(ui, body_label, pal, |ui| {
        // Layout is right-to-left here; items added first appear rightmost
        if let Some(path_filter) = cfg.json_path_filter.as_mut() {
            ui.add(
                egui::TextEdit::singleline(*path_filter)
                    .hint_text("e.g. data.items[0]")
                    .desired_width(180.0)
                    .font(egui::TextStyle::Monospace),
            );
            ui.label(egui::RichText::new("JSON path:").size(theme::FONT_XS).color(pal.faint));
        }
        if show_format {
            if theme::format_button(ui, pal).on_hover_text("Format JSON").clicked() {
                if let Some(fmt) = http::pretty_print_json(cfg.body_text) {
                    *cfg.body_text = fmt;
                }
            }
        }
    });

    let filter_active = path_clone.as_deref().is_some_and(|p| !p.is_empty());
    let display_body: String = if filter_active {
        let p = path_clone.as_deref().unwrap();
        http::apply_json_filter(cfg.body_text, p).unwrap_or_else(|| String::from("<no match>"))
    } else {
        cfg.body_text.clone()
    };

    let body_height = ui.available_height();
    ui.allocate_ui_with_layout(
        egui::vec2(width, body_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            frame.show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(cfg.body_scroll_id)
                    .show(ui, |ui| {
                        if cfg.editable && !filter_active {
                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(cfg.body_text)
                                    .font(egui::TextStyle::Monospace)
                                    .hint_text(cfg.body_hint)
                                    .layouter(cfg.body_layouter)
                                    .frame(false),
                            );
                        } else {
                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(&mut ReadOnlyBuf(&display_body))
                                    .font(egui::TextStyle::Monospace)
                                    .layouter(cfg.body_layouter)
                                    .frame(false),
                            );
                        }
                    });
            });
        },
    );
}

// ── App struct ────────────────────────────────────────────────────────────────

pub struct App {
    method: Method,
    url: String,
    request_headers: String,
    request_body: String,
    response_headers: String,
    response_body: String,
    settings: Settings,
    show_side_panel: bool,
    file_to_load: Option<PathBuf>,
    pending_response: Option<mpsc::Receiver<Response>>,
    dir_tree: Vec<DirEntry>,
    dir_tree_path: Option<String>,
    show_curl_window: bool,
    curl_text: String,
    current_file: Option<PathBuf>,
    show_save_confirm: bool,
    show_import_curl_window: bool,
    import_curl_text: String,
    import_curl_error: bool,
    file_browser_filter: String,
    file_browser_filter_applied: String,
    file_browser_filter_last_changed: Option<std::time::Instant>,
    response_json_path: String,
    last_status: u16,
    last_elapsed_ms: f64,
    last_body_bytes: usize,
    show_settings: bool,
}

impl Default for App {
    fn default() -> Self {
        let settings = Settings::load();
        let dir_tree_path = settings.default_directory.clone();
        let dir_tree = dir_tree_path
            .as_ref()
            .map(|d| DirEntry::scan(d.as_ref()))
            .unwrap_or_default();

        Self {
            method: Method::Get,
            url: String::from("https://httpbin.org/get"),
            request_headers: String::new(),
            request_body: String::new(),
            response_headers: String::new(),
            response_body: String::new(),
            settings,
            show_side_panel: false,
            file_to_load: None,
            pending_response: None,
            dir_tree,
            dir_tree_path,
            show_curl_window: false,
            curl_text: String::new(),
            current_file: None,
            show_save_confirm: false,
            show_import_curl_window: false,
            import_curl_text: String::new(),
            import_curl_error: false,
            file_browser_filter: String::new(),
            file_browser_filter_applied: String::new(),
            file_browser_filter_last_changed: None,
            response_json_path: String::new(),
            last_status: 0,
            last_elapsed_ms: 0.0,
            last_body_bytes: 0,
            show_settings: false,
        }
    }
}

impl App {
    fn send_request(&mut self, ctx: &egui::Context) {
        let method  = self.method;
        let url     = self.url.clone();
        let headers = self.request_headers.clone();
        let body    = self.request_body.clone();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();

        self.response_headers = String::from("Sending…");
        self.response_body.clear();
        self.pending_response = Some(rx);

        std::thread::spawn(move || {
            let resp = http::send_request(method, &url, &headers, &body);
            let _ = tx.send(resp);
            ctx.request_repaint();
        });
    }

    fn refresh_dir_tree(&mut self) {
        self.dir_tree = self.settings.default_directory
            .as_ref()
            .map(|d| DirEntry::scan(d.as_ref()))
            .unwrap_or_default();
        self.dir_tree_path = self.settings.default_directory.clone();
    }

    fn file_dialog(&self) -> rfd::FileDialog {
        let dialog = rfd::FileDialog::new().add_filter("curl", &["curl"]);
        if let Some(dir) = &self.settings.default_directory {
            dialog.set_directory(dir)
        } else {
            dialog
        }
    }

    fn load_request(&mut self) {
        if let Some(path) = self.file_dialog().pick_file() {
            self.load_request_from_path(&path);
        }
    }

    fn load_request_from_path(&mut self, path: &std::path::Path) {
        if let Ok(contents) = std::fs::read_to_string(path)
            && let Some(saved) = SavedRequest::from_curl(&contents)
        {
            self.method = saved.method;
            self.url    = saved.url;
            self.request_headers = saved.headers;
            self.request_body    = http::pretty_print_json(&saved.body).unwrap_or(saved.body);
            self.current_file    = Some(path.to_path_buf());
        }
    }

    fn save_to_current_file(&self) {
        if let Some(path) = &self.current_file {
            let _ = std::fs::write(path, self.build_saved_request().to_curl());
        }
    }

    fn save_as(&mut self) {
        let saved = self.build_saved_request();
        if let Some(mut path) = self.file_dialog().set_file_name("request.curl").save_file() {
            if path.extension().is_none_or(|ext| ext != "curl") {
                path.set_extension("curl");
            }
            let _ = std::fs::write(&path, saved.to_curl());
            self.current_file = Some(path);
        }
    }

    fn new_request(&mut self) {
        self.method          = Method::Get;
        self.url             = String::new();
        self.request_headers = String::new();
        self.request_body    = String::new();
        self.response_headers = String::new();
        self.response_body   = String::new();
        self.current_file    = None;
        self.last_status     = 0;
        self.last_elapsed_ms = 0.0;
        self.last_body_bytes = 0;
    }

    fn build_saved_request(&self) -> SavedRequest {
        SavedRequest {
            method:  self.method,
            url:     self.url.clone(),
            headers: self.request_headers.clone(),
            body:    self.request_body.clone(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Theme ─────────────────────────────────────────────────────────
        let pal = theme::palette(self.settings.theme, self.settings.accent);
        let mut style = (*ctx.style()).clone();
        theme::apply(&mut style, &pal);
        ctx.set_style(style);

        if cfg!(target_os = "linux") {
            ctx.set_pixels_per_point(theme::LINUX_SCALE);
        }

        // Window title
        let title = match &self.current_file {
            Some(p) => format!("CurlU — [{}]", p.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default()),
            None    => String::from("CurlU"),
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));

        // ── Async response ────────────────────────────────────────────────
        if let Some(rx) = &self.pending_response
            && let Ok(resp) = rx.try_recv()
        {
            self.response_headers = resp.headers;
            self.response_body    = resp.body;
            self.last_status      = resp.status;
            self.last_elapsed_ms  = resp.elapsed_ms;
            self.last_body_bytes  = resp.body_bytes;
            self.pending_response = None;
        }

        // ── File browser filter debounce (300 ms) ─────────────────────────
        if let Some(changed_at) = self.file_browser_filter_last_changed {
            let elapsed = changed_at.elapsed();
            if elapsed >= std::time::Duration::from_millis(300) {
                self.file_browser_filter_applied = self.file_browser_filter.clone();
                self.file_browser_filter_last_changed = None;
            } else {
                let remaining = 300u64.saturating_sub(elapsed.as_millis() as u64);
                ctx.request_repaint_after(std::time::Duration::from_millis(remaining));
            }
        }

        // ── Toolbar ───────────────────────────────────────────────────────
        egui::TopBottomPanel::top("toolbar")
            .frame(egui::Frame::new()
                .inner_margin(egui::Margin { left: 8, right: 8, top: 8, bottom: 3 })
                .fill(pal.bg1)
                .stroke(egui::Stroke::new(1.0, pal.stroke)))
            .exact_height(32.0)
            .show(ctx, |ui| {
                ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                ui.visuals_mut().widgets.hovered.weak_bg_fill  = pal.hover;
                ui.visuals_mut().widgets.hovered.bg_stroke     = egui::Stroke::NONE;

                egui::menu::bar(ui, |ui| {
                    if ui.button("File Browser").clicked() { self.show_side_panel = !self.show_side_panel; }
                    ui.separator();
                    if ui.button("New Request").clicked() { self.new_request(); }
                    if ui.button("Open…").clicked()      { self.load_request(); }
                    let save_enabled = self.current_file.is_some();
                    if ui.add_enabled(save_enabled, egui::Button::new("Save")).clicked() {
                        self.show_save_confirm = true;
                    }
                    if ui.button("Save As…").clicked() { self.save_as(); }
                    ui.separator();
                    if ui.button("Show as curl").clicked() {
                        self.curl_text = self.build_saved_request().to_curl();
                        self.show_curl_window = true;
                    }
                    if ui.button("Import curl…").clicked() {
                        self.import_curl_text.clear();
                        self.import_curl_error = false;
                        self.show_import_curl_window = true;
                    }
                    ui.separator();
                    if ui.button("Settings").clicked() {
                        self.show_settings = !self.show_settings;
                    }
                });
            });

        // ── Status bar ────────────────────────────────────────────────────
        // egui::TopBottomPanel::bottom("status_bar")
        //     .frame(egui::Frame::new()
        //         .inner_margin(egui::Margin::symmetric(12, 0))
        //         .fill(pal.bg0)
        //         .stroke(egui::Stroke::new(1.0, pal.stroke)))
        //     .exact_height(22.0)
        //     .show(ctx, |ui| {
        //         ui.horizontal_centered(|ui| {
        //             let is_sending = self.pending_response.is_some();
        //             let dot_color  = if is_sending { pal.accent } else { pal.ok };
        //             ui.colored_label(dot_color, "●");
        //             ui.label(
        //                 egui::RichText::new(if is_sending { "Sending…" } else { "Ready" })
        //                     .size(theme::FONT_SM)
        //                     .color(pal.dim),
        //             );
        //             if !self.url.trim().is_empty() {
        //                 ui.label(egui::RichText::new("·").size(theme::FONT_SM).color(pal.faint));
        //                 let host = extract_host(&self.url);
        //                 ui.label(
        //                     egui::RichText::new(format!("{} {host}", self.method.as_str()))
        //                         .size(theme::FONT_SM)
        //                         .monospace()
        //                         .color(pal.dim),
        //                 );
        //             }
        //         });
        //     });

        // ── Sidebar ───────────────────────────────────────────────────────
        if self.show_side_panel {
            egui::SidePanel::left("file_browser")
                .resizable(true)
                .min_width(theme::SIDE_PANEL_MIN_WIDTH)
                .default_width(theme::SIDE_PANEL_MIN_WIDTH)
                .frame(egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(0, 0))
                    .fill(pal.side)
                    .stroke(egui::Stroke::new(1.0, pal.stroke)))
                .show(ctx, |ui| {
                    // Top controls: path label left, icon buttons right
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), 30.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.add_space(8.0);
                            if let Some(dir) = &self.settings.default_directory {
                                let short = dir.split('/').last().map(|s| format!("~/{s}")).unwrap_or_default();
                                ui.label(egui::RichText::new(short).size(theme::FONT_XS).monospace().color(pal.faint));
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_space(6.0);
                                ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                                ui.visuals_mut().widgets.hovered.weak_bg_fill  = pal.hover;
                                ui.visuals_mut().widgets.hovered.bg_stroke     = egui::Stroke::NONE;
                                if ui.add(egui::Button::new(
                                    egui::RichText::new("⊕").size(theme::FONT_MD).color(pal.dim)
                                ).frame(true)).on_hover_text("Change directory").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.settings.default_directory = Some(path.to_string_lossy().into_owned());
                                        self.settings.save();
                                        self.refresh_dir_tree();
                                    }
                                }
                                if self.settings.default_directory.is_some() {
                                    if ui.add(egui::Button::new(
                                        egui::RichText::new("↺").size(theme::FONT_MD).color(pal.dim)
                                    ).frame(true)).on_hover_text("Refresh").clicked() {
                                        self.refresh_dir_tree();
                                    }
                                }
                            });
                        },
                    );

                    ui.add(egui::Separator::default().spacing(0.0));

                    // Filter input with 10px padding on each side
                    ui.visuals_mut().widgets.inactive.bg_fill      = pal.bg2;
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = pal.bg2;
                    ui.visuals_mut().widgets.inactive.bg_stroke    = egui::Stroke::new(1.0, pal.stroke);
                    ui.visuals_mut().widgets.hovered.bg_fill       = pal.bg2;
                    ui.visuals_mut().widgets.hovered.weak_bg_fill  = pal.bg2;
                    ui.visuals_mut().widgets.hovered.bg_stroke     = egui::Stroke::new(1.0, pal.stroke);
                    egui::Frame::none()
                        .inner_margin(egui::Margin { left: 10, right: 10, top: 4, bottom: 4 })
                        .show(ui, |ui| {
                            let filter_resp = ui.add_sized(
                                [ui.available_width(), 26.0],
                                egui::TextEdit::singleline(&mut self.file_browser_filter)
                                    .hint_text("Filter…")
                                    .margin(theme::URL_INPUT_MARGIN)
                                    .desired_rows(1)
                                    .font(egui::TextStyle::Monospace),
                            );
                            if filter_resp.changed() {
                                self.file_browser_filter_last_changed = Some(std::time::Instant::now());
                            }
                        });
                    ui.add(egui::Separator::default().spacing(0.0));

                    // Reserve footer height before the scroll area so it stays visible
                    let count = count_files(&self.dir_tree);
                    let footer_h = if count > 0 { 25.0 } else { 0.0 };
                    let scroll_h = (ui.available_height() - footer_h).max(0.0);

                    // File tree — vertical scroll only to prevent panel width growth
                    egui::ScrollArea::vertical()
                        .max_height(scroll_h)
                        .show(ui, |ui| {
                            ui.add_space(5.0);
                            let filter = self.file_browser_filter_applied.as_str();
                            if filter.is_empty() {
                                DirEntry::show(&self.dir_tree, ui, &mut self.file_to_load, &self.current_file, &pal);
                            } else {
                                DirEntry::show_filtered(&self.dir_tree, ui, &mut self.file_to_load, filter, &self.current_file, &pal);
                            }
                        });

                    // ui.add(egui::Separator::default().spacing(0.0));

                    // Footer: file count
                    // if count > 0 {
                    //     ui.add(egui::Separator::default().spacing(0.0));
                    //     ui.horizontal(|ui| {
                    //         ui.add_space(11.0);
                    //         ui.label(egui::RichText::new(format!("{count} requests")).size(11.0).color(pal.faint));
                    //     });
                    // }
                });
        }

        if let Some(path) = self.file_to_load.take() {
            self.load_request_from_path(&path);
        }

        // ── Layouters ─────────────────────────────────────────────────────
        let header_layouter = {
            let syn_key   = pal.syn_key;
            let syn_plain = pal.syn_plain;
            let syn_punct = pal.syn_punct;
            move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                let mut job = highlight::headers_colored(text, font_id, syn_key, syn_plain, syn_punct);
                job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(job))
            }
        };
        let json_layouter = {
            let pal2 = pal.clone();
            move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                let mut job = highlight::json_colored(text, font_id, &pal2);
                job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(job))
            }
        };

        // ── Central panel ─────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::new()
                .inner_margin(egui::Margin::same(11))
                .fill(pal.bg1))
            .show(ctx, |ui| {
                // Request bar
                ui.horizontal(|ui| {
                    // Method combo
                    ui.visuals_mut().widgets.inactive.bg_fill      = pal.bg2;
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = pal.bg2;
                    ui.visuals_mut().widgets.inactive.bg_stroke    = egui::Stroke::new(1.0, pal.stroke);
                    ui.visuals_mut().widgets.hovered.bg_fill       = pal.bg2;
                    ui.visuals_mut().widgets.hovered.weak_bg_fill  = pal.bg2;
                    ui.visuals_mut().widgets.hovered.bg_stroke     = egui::Stroke::new(1.0, pal.stroke);
                    ui.allocate_ui_with_layout(
                        egui::vec2(theme::COMBOBOX_SIZE[0], theme::COMBOBOX_SIZE[1]),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.spacing_mut().button_padding = egui::vec2(12.0, 9.0);
                            egui::ComboBox::from_id_salt("method")
                                .width(theme::COMBOBOX_SIZE[0])
                                .selected_text(
                                    egui::RichText::new(self.method.as_str())
                                        .monospace()
                                        .strong()
                                        .color(pal.text),
                                )
                                .show_ui(ui, |ui| {
                                    for m in Method::ALL {
                                        ui.selectable_value(&mut self.method, *m,
                                            egui::RichText::new(m.as_str()).monospace());
                                    }
                                });
                        },
                    );

                    // URL input (accent border when focused)
                    ui.visuals_mut().widgets.inactive.bg_fill      = pal.bg2;
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = pal.bg2;
                    ui.visuals_mut().widgets.inactive.bg_stroke    = egui::Stroke::new(1.0, pal.stroke);
                    ui.visuals_mut().widgets.hovered.bg_fill       = pal.bg2;
                    ui.visuals_mut().widgets.hovered.weak_bg_fill  = pal.bg2;
                    ui.visuals_mut().widgets.hovered.bg_stroke     = egui::Stroke::new(1.0, pal.stroke);
                    ui.visuals_mut().widgets.open.bg_stroke        = egui::Stroke::new(1.5, pal.accent);
                    let url_response = ui.add_sized(
                        [ui.available_width() - theme::BUTTON_SIZE[0] - theme::ITEM_SPACING.x, theme::URL_INPUT_HEIGHT],
                        egui::TextEdit::multiline(&mut self.url)
                            .font(theme::url_font())
                            .margin(theme::URL_INPUT_MARGIN)
                            .desired_rows(1)
                            .desired_width(f32::INFINITY),
                    );
                    if url_response.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.url = self.url.trim().to_string();
                        self.send_request(ctx);
                    }

                    // Send button
                    if ui.add_sized(theme::BUTTON_SIZE, theme::send_button(&pal))
                        .on_hover_text("Send request (Enter)")
                        .clicked()
                    {
                        self.send_request(ctx);
                    }
                });

                ui.add_space(10.0);

                // Two-column layout
                ui.columns(2, |cols| {
                    let hl = header_layouter.clone();
                    let jl = json_layouter.clone();
                    render_column(&mut cols[0], 196.0, ColumnConfig {
                        header_label: "REQUEST HEADERS",
                        body_label: "REQUEST BODY",
                        headers_text: &mut self.request_headers,
                        editable: true,
                        headers_hint: "Content-Type: application/json",
                        body_text: &mut self.request_body,
                        body_hint: "{ \"key\": \"value\" }",
                        header_scroll_id: "req_headers_scroll",
                        body_scroll_id: "req_body_scroll",
                        header_layouter: &mut { let mut f = hl; move |ui, s, w| f(ui, s, w) },
                        body_layouter:   &mut { let mut f = jl; move |ui, s, w| f(ui, s, w) },
                        show_format_button: true,
                        json_path_filter: None,
                        response_info: None,
                    }, &pal);

                    let hl2 = header_layouter.clone();
                    let jl2 = json_layouter.clone();
                    render_column(&mut cols[1], 196.0, ColumnConfig {
                        header_label: "RESPONSE HEADERS",
                        body_label: "RESPONSE BODY",
                        headers_text: &mut self.response_headers,
                        editable: false,
                        headers_hint: "",
                        body_text: &mut self.response_body,
                        body_hint: "",
                        header_scroll_id: "resp_headers_scroll",
                        body_scroll_id: "resp_body_scroll",
                        header_layouter: &mut { let mut f = hl2; move |ui, s, w| f(ui, s, w) },
                        body_layouter:   &mut { let mut f = jl2; move |ui, s, w| f(ui, s, w) },
                        show_format_button: false,
                        json_path_filter: Some(&mut self.response_json_path),
                        response_info: Some((self.last_status, self.last_elapsed_ms, self.last_body_bytes)),
                    }, &pal);
                });
            });

        // ── Show-as-curl window ───────────────────────────────────────────
        if self.show_curl_window {
            let screen = ctx.screen_rect();
            egui::Window::new("Curl")
                .open(&mut self.show_curl_window)
                .resizable(true)
                .default_width(500.0)
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(screen.center())
                .show(ctx, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut ReadOnlyBuf(&self.curl_text))
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY),
                    );
                    if ui.button("Copy to clipboard").clicked() {
                        ui.ctx().copy_text(self.curl_text.clone());
                    }
                });
        }

        // ── Import-curl window ────────────────────────────────────────────
        if self.show_import_curl_window {
            let screen = ctx.screen_rect();
            let mut close    = false;
            let mut do_import = false;
            egui::Window::new("Import curl")
                .resizable(true)
                .default_width(500.0)
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(screen.center())
                .show(ctx, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.import_curl_text)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("Paste curl command…")
                            .desired_width(f32::INFINITY)
                            .desired_rows(6),
                    );
                    if self.import_curl_error {
                        ui.colored_label(egui::Color32::from_rgb(220, 80, 80), "Could not parse curl command.");
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Import").clicked() { do_import = true; }
                        if ui.button("Cancel").clicked() { close    = true; }
                    });
                });
            if do_import {
                if let Some(saved) = http::SavedRequest::from_curl(&self.import_curl_text) {
                    self.method          = saved.method;
                    self.url             = saved.url;
                    self.request_headers = saved.headers;
                    self.request_body    = http::pretty_print_json(&saved.body).unwrap_or(saved.body);
                    self.show_import_curl_window = false;
                    self.import_curl_error       = false;
                } else {
                    self.import_curl_error = true;
                }
            }
            if close {
                self.show_import_curl_window = false;
                self.import_curl_error       = false;
            }
        }

        // ── Save-confirm window ───────────────────────────────────────────
        if self.show_save_confirm {
            let screen = ctx.screen_rect();
            egui::Window::new("Overwrite file?")
                .collapsible(false)
                .resizable(false)
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(screen.center())
                .show(ctx, |ui| {
                    let file_name = self.current_file.as_ref()
                        .and_then(|p| p.file_name())
                        .map(|f| f.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    ui.label(format!("Overwrite \"{file_name}\"?"));
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.save_to_current_file();
                            self.show_save_confirm = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_save_confirm = false;
                        }
                    });
                });
        }

        // ── Settings window ───────────────────────────────────────────────
        if self.show_settings {
            let screen = ctx.screen_rect();
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .min_width(260.0)
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(screen.center())
                .open(&mut self.show_settings)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Theme").size(theme::FONT_MD).color(pal.dim).strong());
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        for (theme_opt, label, preview_bg, preview_text) in [
                            (Theme::Dark,  "Dark",  pal.bg2,                          pal.text),
                            (Theme::Light, "Light", egui::Color32::from_rgb(255,255,255), egui::Color32::from_rgb(35,39,46)),
                        ] {
                            let selected = self.settings.theme == theme_opt;
                            let border   = if selected { pal.accent } else { pal.stroke };
                            let bw       = if selected { 2.0_f32 } else { 1.0_f32 };
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(80.0, 52.0), egui::Sense::click()
                            );
                            if ui.is_rect_visible(rect) {
                                // Card background
                                ui.painter().rect(rect, egui::CornerRadius::same(6), preview_bg, egui::Stroke::new(bw, border), egui::StrokeKind::Outside);
                                // Mini window chrome strip
                                let chrome = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 10.0));
                                let chrome_bg = if matches!(theme_opt, Theme::Dark) {
                                    egui::Color32::from_rgb(21,23,28)
                                } else {
                                    egui::Color32::from_rgb(232,234,237)
                                };
                                ui.painter().rect(chrome, egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 }, chrome_bg, egui::Stroke::NONE, egui::StrokeKind::Outside);
                                // Label text
                                ui.painter().text(
                                    rect.center() + egui::vec2(0.0, 5.0),
                                    egui::Align2::CENTER_CENTER,
                                    label,
                                    egui::FontId::proportional(theme::FONT_MD),
                                    preview_text,
                                );
                            }
                            if resp.clicked() {
                                self.settings.theme = theme_opt;
                                self.settings.save();
                            }
                            ui.add_space(8.0);
                        }
                    });

                    ui.add_space(14.0);
                    ui.label(egui::RichText::new("Accent color").size(theme::FONT_MD).color(pal.dim).strong());
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        for (accent_opt, label, ar, ag, ab) in [
                            (Accent::Blue,  "Blue",  59_u8, 130_u8, 246_u8),
                            (Accent::Green, "Green", 47,    170,     87),
                            (Accent::Amber, "Amber", 223,   142,     38),
                        ] {
                            let selected    = self.settings.accent == accent_opt;
                            let accent_col  = egui::Color32::from_rgb(ar, ag, ab);
                            let border      = if selected { accent_col } else { pal.stroke };
                            let bw          = if selected { 2.0_f32 } else { 1.0_f32 };
                            let fill        = if selected { accent_col } else { pal.bg2 };
                            let check_color = pal.accent_text;

                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(52.0, 52.0), egui::Sense::click()
                            );
                            if ui.is_rect_visible(rect) {
                                ui.painter().rect(rect, egui::CornerRadius::same(8), fill, egui::Stroke::new(bw, border), egui::StrokeKind::Outside);
                                if selected {
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "✓",
                                        egui::FontId::proportional(18.0),
                                        check_color,
                                    );
                                } else {
                                    ui.painter().circle_filled(rect.center(), 8.0, accent_col);
                                }
                                ui.painter().text(
                                    rect.center_bottom() + egui::vec2(0.0, 8.0),
                                    egui::Align2::CENTER_TOP,
                                    label,
                                    egui::FontId::proportional(theme::FONT_SM),
                                    if selected { pal.accent } else { pal.dim },
                                );
                            }
                            if resp.clicked() {
                                self.settings.accent = accent_opt;
                                self.settings.save();
                            }
                            ui.add_space(12.0);
                        }
                    });
                    ui.add_space(16.0); // space for labels below the squares
                });
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn extract_host(url: &str) -> &str {
    let s = url.trim().trim_start_matches("https://").trim_start_matches("http://");
    s.split('/').next().unwrap_or(s)
}

fn count_files(entries: &[DirEntry]) -> usize {
    entries.iter().map(|e| match e {
        DirEntry::File { .. }        => 1,
        DirEntry::Dir { children, .. } => count_files(children),
    }).sum()
}
