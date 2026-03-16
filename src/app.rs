use eframe::egui;
use std::path::PathBuf;

use crate::highlight;
use crate::http::{self, Method, SavedRequest};
use crate::settings::Settings;


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
}

impl Default for App {
    fn default() -> Self {
        Self {
            method: Method::Get,
            url: String::from("https://httpbin.org/get"),
            request_headers: String::new(),
            request_body: String::new(),
            response_headers: String::new(),
            response_body: String::new(),
            settings: Settings::load(),
            show_side_panel: false,
            file_to_load: None,
        }
    }
}

impl App {
    fn send_request(&mut self) {
        let resp = http::send_request(
            self.method,
            &self.url,
            &self.request_headers,
            &self.request_body,
        );
        self.response_headers = resp.headers;
        self.response_body = resp.body;
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
            if path.extension().map_or(true, |ext| ext != "curl") {
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
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Some(saved) = SavedRequest::from_curl(&contents) {
                self.method = saved.method;
                self.url = saved.url;
                self.request_headers = saved.headers;
                self.request_body = saved.body;
            }
        }
    }

    fn show_dir_tree(
        ui: &mut egui::Ui,
        dir: &std::path::Path,
        file_to_load: &mut Option<PathBuf>,
    ) {
        let mut entries: Vec<_> = match std::fs::read_dir(dir) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
            Err(_) => return,
        };
        entries.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));

        for entry in entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();

            if path.is_dir() {
                egui::CollapsingHeader::new(&name)
                    .id_salt(path.display().to_string())
                    .show(ui, |ui| {
                        Self::show_dir_tree(ui, &path, file_to_load);
                    });
            } else if path.extension().is_some_and(|ext| ext == "curl") {
                if ui.selectable_label(false, &name).clicked() {
                    *file_to_load = Some(path);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.settings.default_directory =
                                    Some(path.to_string_lossy().into_owned());
                                self.settings.save();
                            }
                        }
                        if self.settings.default_directory.is_some() && ui.button("Clear").clicked() {
                            self.settings.default_directory = None;
                            self.settings.save();
                        }
                    });
                    ui.separator();
                    if let Some(dir) = &self.settings.default_directory {
                        let dir = PathBuf::from(dir);
                        egui::ScrollArea::both().show(ui, |ui| {
                            Self::show_dir_tree(ui, &dir, &mut self.file_to_load);
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

        egui::CentralPanel::default().show(ctx, |ui| {
            // Toolbar row: method + url + buttons
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

                if ui.button(egui::RichText::new("Send").size(18.0)).clicked() {
                    self.send_request();
                }
                if ui.button(egui::RichText::new("Save").size(18.0)).clicked() {
                    self.save_request();
                }
                if ui.button(egui::RichText::new("Load").size(18.0)).clicked() {
                    self.load_request();
                }
            });

            ui.add_space(4.0);

            // Two columns: request (left) and response (right)
            ui.columns(2, |cols| {
                // Helper: render a column with fixed-height headers + body filling the rest
                fn render_column(
                    ui: &mut egui::Ui,
                    headers_height: f32,
                    header_label: &str,
                    body_label: &str,
                    headers_text: &mut String,
                    headers_interactive: bool,
                    headers_hint: &str,
                    body_text: &mut String,
                    body_interactive: bool,
                    body_hint: &str,
                    header_scroll_id: &str,
                    body_scroll_id: &str,
                    header_layouter: &mut dyn FnMut(&egui::Ui, &str, f32) -> std::sync::Arc<egui::Galley>,
                    body_layouter: &mut dyn FnMut(&egui::Ui, &str, f32) -> std::sync::Arc<egui::Galley>,
                ) {
                    let visuals = ui.visuals().clone();
                    let outer_frame = egui::Frame::new()
                        .stroke(visuals.widgets.noninteractive.bg_stroke)
                        .corner_radius(visuals.widgets.noninteractive.corner_radius)
                        .fill(visuals.extreme_bg_color);

                    ui.label(egui::RichText::new(header_label).size(18.0));
                    let width = ui.available_width();
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, headers_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            outer_frame.show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .id_salt(header_scroll_id)
                                    .show(ui, |ui: &mut egui::Ui| {
                                        ui.add_sized(
                                            ui.available_size(),
                                            egui::TextEdit::multiline(headers_text)
                                                .font(egui::TextStyle::Monospace)
                                                .hint_text(headers_hint)
                                                .interactive(headers_interactive)
                                                .layouter(header_layouter)
                                                .frame(false),
                                        );
                                    });
                            });
                        },
                    );

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(body_label).size(18.0));

                    let body_height = ui.available_height();
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, body_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            outer_frame.show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .id_salt(body_scroll_id)
                                    .show(ui, |ui: &mut egui::Ui| {
                                        ui.add_sized(
                                            ui.available_size(),
                                            egui::TextEdit::multiline(body_text)
                                                .font(egui::TextStyle::Monospace)
                                                .hint_text(body_hint)
                                                .interactive(body_interactive)
                                                .layouter(body_layouter)
                                                .frame(false),
                                        );
                                    });
                            });
                        },
                    );
                }

                render_column(
                    &mut cols[0],
                    250.0,
                    "Request Headers",
                    "Request Body",
                    &mut self.request_headers,
                    true,
                    "Content-Type: application/json",
                    &mut self.request_body,
                    true,
                    "{\"key\": \"value\"}",
                    "req_headers_scroll",
                    "req_body_scroll",
                    &mut header_layouter.clone(),
                    &mut json_layouter.clone(),
                );

                render_column(
                    &mut cols[1],
                    250.0,
                    "Response Headers",
                    "Response Body",
                    &mut self.response_headers,
                    false,
                    "",
                    &mut self.response_body,
                    false,
                    "",
                    "resp_headers_scroll",
                    "resp_body_scroll",
                    &mut header_layouter.clone(),
                    &mut json_layouter.clone(),
                );
            });
        });

    }
}
