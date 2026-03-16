use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::highlight;
use crate::http::{self, Method, Response, SavedRequest};
use crate::settings::Settings;

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
    headers_interactive: bool,
    headers_hint: &'a str,
    body_text: &'a mut String,
    body_interactive: bool,
    body_hint: &'a str,
    header_scroll_id: &'a str,
    body_scroll_id: &'a str,
    header_layouter: &'a mut dyn FnMut(&egui::Ui, &str, f32) -> std::sync::Arc<egui::Galley>,
    body_layouter: &'a mut dyn FnMut(&egui::Ui, &str, f32) -> std::sync::Arc<egui::Galley>,
}

fn render_column(ui: &mut egui::Ui, headers_height: f32, cfg: ColumnConfig<'_>) {
    let visuals = ui.visuals().clone();
    let outer_frame = egui::Frame::new()
        .stroke(visuals.widgets.noninteractive.bg_stroke)
        .corner_radius(visuals.widgets.noninteractive.corner_radius)
        .fill(visuals.extreme_bg_color);

    ui.label(egui::RichText::new(cfg.header_label).size(18.0));
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, headers_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            outer_frame.show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(cfg.header_scroll_id)
                    .show(ui, |ui: &mut egui::Ui| {
                        ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(cfg.headers_text)
                                .font(egui::TextStyle::Monospace)
                                .hint_text(cfg.headers_hint)
                                .interactive(cfg.headers_interactive)
                                .layouter(cfg.header_layouter)
                                .frame(false),
                        );
                    });
            });
        },
    );

    ui.add_space(4.0);
    ui.label(egui::RichText::new(cfg.body_label).size(18.0));

    let body_height = ui.available_height();
    ui.allocate_ui_with_layout(
        egui::vec2(width, body_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            outer_frame.show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(cfg.body_scroll_id)
                    .show(ui, |ui: &mut egui::Ui| {
                        ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(cfg.body_text)
                                .font(egui::TextStyle::Monospace)
                                .hint_text(cfg.body_hint)
                                .interactive(cfg.body_interactive)
                                .layouter(cfg.body_layouter)
                                .frame(false),
                        );
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
        // Check for completed async response
        if let Some(rx) = &self.pending_response
            && let Ok(resp) = rx.try_recv() {
                self.response_headers = resp.headers;
                self.response_body = resp.body;
                self.pending_response = None;
            }

        // Refresh dir tree if settings directory changed
        if self.dir_tree_path != self.settings.default_directory {
            self.refresh_dir_tree();
        }

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.show_side_panel, egui::RichText::new("Saved Requests").size(18.0)).clicked() {
                    self.show_side_panel = !self.show_side_panel;
                }
            });
        });

        if self.show_side_panel {
            egui::SidePanel::left("file_browser")
                .resizable(true)
                .min_width(250.0)
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.label("Directory:");
                    ui.horizontal(|ui| {
                        let display = self
                            .settings
                            .default_directory
                            .as_deref()
                            .unwrap_or("(not set)");
                        ui.label(display);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Browse...").clicked()
                            && let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.settings.default_directory =
                                    Some(path.to_string_lossy().into_owned());
                                self.settings.save();
                                self.refresh_dir_tree();
                            }
                        if self.settings.default_directory.is_some() && ui.button("Clear").clicked() {
                            self.settings.default_directory = None;
                            self.settings.save();
                            self.refresh_dir_tree();
                        }
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

        let is_sending = self.pending_response.is_some();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("method")
                    .width(150.0)
                    .selected_text(egui::RichText::new(self.method.as_str()).size(18.0))
                    .show_ui(ui, |ui| {
                        for m in Method::ALL {
                            ui.selectable_value(&mut self.method, *m, egui::RichText::new(m.as_str()).size(21.0));
                        }
                    });

                let url_font = egui::FontId::proportional(18.0);
                ui.add(egui::TextEdit::singleline(&mut self.url).desired_width(ui.available_width() - 180.0).font(url_font));

                let send_label = if is_sending { "Sending..." } else { "Send" };
                if ui.add_enabled(!is_sending, egui::Button::new(egui::RichText::new(send_label).size(18.0))).clicked() {
                    self.send_request(ctx);
                }
                if ui.button(egui::RichText::new("Save").size(18.0)).clicked() {
                    self.save_request();
                }
                if ui.button(egui::RichText::new("Load").size(18.0)).clicked() {
                    self.load_request();
                }
            });

            ui.add_space(4.0);

            ui.columns(2, |cols| {
                render_column(&mut cols[0], 250.0, ColumnConfig {
                    header_label: "Request Headers",
                    body_label: "Request Body",
                    headers_text: &mut self.request_headers,
                    headers_interactive: true,
                    headers_hint: "Content-Type: application/json",
                    body_text: &mut self.request_body,
                    body_interactive: true,
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
                    headers_interactive: false,
                    headers_hint: "",
                    body_text: &mut self.response_body,
                    body_interactive: false,
                    body_hint: "",
                    header_scroll_id: "resp_headers_scroll",
                    body_scroll_id: "resp_body_scroll",
                    header_layouter: &mut header_layouter.clone(),
                    body_layouter: &mut json_layouter.clone(),
                });
            });
        });
    }
}
