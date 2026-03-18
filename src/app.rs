use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::highlight;
use crate::http::{self, Method, Response, SavedRequest};
use crate::settings::Settings;
use crate::theme;

/// A TextBuffer wrapper that allows selection/copy but silently ignores all edits.
struct ReadOnlyBuf<'a>(&'a str);

impl egui::TextBuffer for ReadOnlyBuf<'_> {
    fn is_mutable(&self) -> bool { false }
    fn as_str(&self) -> &str { self.0 }
    fn insert_text(&mut self, _text: &str, _char_index: usize) -> usize { 0 }
    fn delete_char_range(&mut self, _char_range: std::ops::Range<usize>) {}
}

enum DirEntry {
    File { name: String, path: PathBuf },
    Dir { name: String, path: PathBuf, children: Vec<DirEntry> },
}

impl DirEntry {
    fn scan(dir: &std::path::Path) -> Vec<DirEntry> {
        let mut entries: Vec<_> = match std::fs::read_dir(dir) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
            Err(_) => return Vec::new(),
        };
        entries.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));

        entries
            .into_iter()
            .filter_map(|entry| {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();

                if path.is_dir() {
                    let children = Self::scan(&path);
                    Some(DirEntry::Dir { name, path, children })
                } else if path.extension().is_some_and(|ext| ext == "curl") {
                    Some(DirEntry::File { name, path })
                } else {
                    None
                }
            })
            .collect()
    }

    fn show(entries: &[DirEntry], ui: &mut egui::Ui, file_to_load: &mut Option<PathBuf>) {
        for entry in entries {
            match entry {
                DirEntry::Dir { name, path, children } => {
                    egui::CollapsingHeader::new(name)
                        .id_salt(path.display().to_string())
                        .show(ui, |ui| {
                            Self::show(children, ui, file_to_load);
                        });
                }
                DirEntry::File { name, path } => {
                    if ui.selectable_label(false, name).clicked() {
                        *file_to_load = Some(path.clone());
                    }
                }
            }
        }
    }
}

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
}

fn render_column(ui: &mut egui::Ui, headers_height: f32, cfg: ColumnConfig<'_>) {
    let outer_frame = egui::Frame::new()
        .stroke(egui::Stroke::new(theme::FRAME_STROKE_WIDTH, theme::FRAME_STROKE_COLOR))
        .corner_radius(theme::FRAME_CORNER_RADIUS)
        .fill(theme::FRAME_FILL);

    ui.label(theme::text(cfg.header_label));
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, headers_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            outer_frame.show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(cfg.header_scroll_id)
                    .show(ui, |ui: &mut egui::Ui| {
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

    ui.add_space(4.0);
    ui.label(theme::text(cfg.body_label));

    let body_height = ui.available_height();
    ui.allocate_ui_with_layout(
        egui::vec2(width, body_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            outer_frame.show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(cfg.body_scroll_id)
                    .show(ui, |ui: &mut egui::Ui| {
                        if cfg.editable {
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
                                egui::TextEdit::multiline(&mut ReadOnlyBuf(cfg.body_text))
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
        }
    }
}

impl App {
    fn send_request(&mut self, ctx: &egui::Context) {
        let method = self.method;
        let url = self.url.clone();
        let headers = self.request_headers.clone();
        let body = self.request_body.clone();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();

        self.response_headers = String::from("Sending...");
        self.response_body.clear();
        self.pending_response = Some(rx);

        std::thread::spawn(move || {
            let resp = http::send_request(method, &url, &headers, &body);
            let _ = tx.send(resp);
            ctx.request_repaint();
        });
    }

    fn refresh_dir_tree(&mut self) {
        self.dir_tree = self
            .settings
            .default_directory
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

    fn save_request(&self) {
        let saved = SavedRequest {
            method: self.method,
            url: self.url.clone(),
            headers: self.request_headers.clone(),
            body: self.request_body.clone(),
        };
        if let Some(mut path) = self.file_dialog().set_file_name("request.curl").save_file() {
            if path.extension().is_none_or(|ext| ext != "curl") {
                path.set_extension("curl");
            }
            let _ = std::fs::write(path, saved.to_curl());
        }
    }

    fn load_request(&mut self) {
        if let Some(path) = self.file_dialog().pick_file() {
            self.load_request_from_path(&path);
        }
    }

    fn load_request_from_path(&mut self, path: &std::path::Path) {
        if let Ok(contents) = std::fs::read_to_string(path)
            && let Some(saved) = SavedRequest::from_curl(&contents) {
                self.method = saved.method;
                self.url = saved.url;
                self.request_headers = saved.headers;
                self.request_body = saved.body;
            }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        theme::apply(&mut style);
        ctx.set_style(style);

        if cfg!(target_os = "linux") {
            ctx.set_pixels_per_point(theme::LINUX_SCALE);
        }

        // Check for completed async response
        if let Some(rx) = &self.pending_response
            && let Ok(resp) = rx.try_recv() {
                self.response_headers = resp.headers;
                self.response_body = resp.body;
                self.pending_response = None;
            }

        // // Refresh dir tree if settings directory changed
        // if self.dir_tree_path != self.settings.default_directory {
        //     self.refresh_dir_tree();
        // }

        if self.show_side_panel {
            egui::SidePanel::left("file_browser")
                .resizable(true)
                .min_width(theme::SIDE_PANEL_MIN_WIDTH)
                .default_width(theme::SIDE_PANEL_MIN_WIDTH)
                .show(ctx, |ui| {
                    // ui.label("Directory:");
                    // ui.horizontal(|ui| {
                    //     let display = self
                    //         .settings
                    //         .default_directory
                    //         .as_deref()
                    //         .unwrap_or("(not set)");
                    //     ui.label(display);
                    // });
                    ui.horizontal(|ui| {
                        if ui.button("Change directory").clicked()
                            && let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.settings.default_directory =
                                    Some(path.to_string_lossy().into_owned());
                                self.settings.save();
                                self.refresh_dir_tree();
                            }
                        // if self.settings.default_directory.is_some() && ui.button("Clear").clicked() {
                        //     self.settings.default_directory = None;
                        //     self.settings.save();
                        //     self.refresh_dir_tree();
                        // }
                        if self.settings.default_directory.is_some() && ui.button("Refresh").clicked() {
                            self.refresh_dir_tree();
                        }
                    });
                    ui.separator();
                    if !self.dir_tree.is_empty() {
                        egui::ScrollArea::both().show(ui, |ui| {
                            DirEntry::show(&self.dir_tree, ui, &mut self.file_to_load);
                        });
                    }
                });
        }

        if let Some(path) = self.file_to_load.take() {
            self.load_request_from_path(&path);
        }

        let header_layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let font_id = egui::TextStyle::Monospace.resolve(ui.style());
            let mut job = highlight::headers(text, font_id);
            job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(job))
        };
        let json_layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let font_id = egui::TextStyle::Monospace.resolve(ui.style());
            let mut job = highlight::json(text, font_id);
            job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(job))
        };

        // let is_sending = self.pending_response.is_some();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add_sized(theme::BUTTON_SIZE, egui::Button::new(theme::text("HIST"))).clicked() {
                    self.show_side_panel = !self.show_side_panel;
                }

                ui.allocate_ui_with_layout(
                    egui::vec2(theme::COMBOBOX_SIZE[0], theme::COMBOBOX_SIZE[1]),
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        egui::ComboBox::from_id_salt("method")
                            .width(theme::COMBOBOX_SIZE[0])
                            .selected_text(theme::text(self.method.as_str()))
                            .show_ui(ui, |ui| {
                                for m in Method::ALL {
                                    ui.selectable_value(&mut self.method, *m, theme::text(m.as_str()));
                                }
                            });
                    },
                );

                ui.add_sized(
                    [ui.available_width() - theme::URL_WIDTH_OFFSET, theme::URL_INPUT_HEIGHT],
                    egui::TextEdit::singleline(&mut self.url)
                        .font(theme::url_font())
                        .margin(theme::URL_INPUT_MARGIN),
                );


                if ui.add_sized(theme::BUTTON_SIZE, egui::Button::new(theme::text("Send"))).clicked() {
                    self.send_request(ctx);
                }
                if ui.add_sized(theme::BUTTON_SIZE, egui::Button::new(theme::text("Save"))).clicked() {
                    self.save_request();
                }
                if ui.add_sized(theme::BUTTON_SIZE, egui::Button::new(theme::text("Load"))).clicked() {
                    self.load_request();
                }
                if ui.add_sized(theme::BUTTON_SIZE, egui::Button::new(theme::text("Show"))).clicked() {
                    let saved = SavedRequest {
                        method: self.method,
                        url: self.url.clone(),
                        headers: self.request_headers.clone(),
                        body: self.request_body.clone(),
                    };
                    self.curl_text = saved.to_curl();
                    self.show_curl_window = true;
                }
            });

            ui.add_space(4.0);

            ui.columns(2, |cols| {
                render_column(&mut cols[0], 250.0, ColumnConfig {
                    header_label: "Request Headers",
                    body_label: "Request Body",
                    headers_text: &mut self.request_headers,
                    editable: true,
                    headers_hint: "Content-Type: application/json",
                    body_text: &mut self.request_body,
                    body_hint: "{\"key\": \"value\"}",
                    header_scroll_id: "req_headers_scroll",
                    body_scroll_id: "req_body_scroll",
                    header_layouter: &mut header_layouter.clone(),
                    body_layouter: &mut json_layouter.clone(),
                });

                render_column(&mut cols[1], 250.0, ColumnConfig {
                    header_label: "Response Headers",
                    body_label: "Response Body",
                    headers_text: &mut self.response_headers,
                    editable: false,
                    headers_hint: "",
                    body_text: &mut self.response_body,
                    body_hint: "",
                    header_scroll_id: "resp_headers_scroll",
                    body_scroll_id: "resp_body_scroll",
                    header_layouter: &mut header_layouter.clone(),
                    body_layouter: &mut json_layouter.clone(),
                });
            });
        });

        if self.show_curl_window {
            let screen = ctx.screen_rect();
            egui::Window::new("Curl Command")
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
    }
}
