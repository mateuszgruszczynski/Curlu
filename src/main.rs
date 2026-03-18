mod app;
mod highlight;
mod http;
mod settings;
mod theme;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(theme::WINDOW_SIZE)
            .with_min_inner_size(theme::WINDOW_MIN_SIZE),
        ..Default::default()
    };
    eframe::run_native(
        "Curlu - Curl UI",
        options,
        Box::new(|cc| {
            let scale = if cfg!(target_os = "linux") {
                theme::LINUX_SCALE
            } else {
                cc.egui_ctx.pixels_per_point()
            };
            cc.egui_ctx.set_pixels_per_point(scale);
            Ok(Box::new(app::App::default()))
        }),
    )
}
