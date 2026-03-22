mod app;
mod highlight;
mod http;
mod settings;
mod theme;

use eframe::egui;

fn load_icon() -> egui::IconData {
    let img = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("Failed to load icon")
        .into_rgba8();
    let (w, h) = img.dimensions();
    egui::IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(theme::WINDOW_SIZE)
            .with_min_inner_size(theme::WINDOW_MIN_SIZE)
            .with_icon(load_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "Curlu - Curl UI",
        options,
        Box::new(|cc| {
            if cfg!(target_os = "linux") {
                cc.egui_ctx.set_pixels_per_point(theme::LINUX_SCALE);
            }
            Ok(Box::new(app::App::default()))
        }),
    )
}
