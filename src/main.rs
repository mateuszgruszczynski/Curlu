mod app;
mod highlight;
mod http;
mod settings;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Curlu - Curl UI",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::default()))),
    )
}
