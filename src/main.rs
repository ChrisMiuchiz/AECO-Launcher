use eframe::{egui, emath::Vec2};

struct PatcherApp {}

impl PatcherApp {
    fn new() -> PatcherApp {
        PatcherApp {}
    }
}

impl eframe::App for PatcherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |_ui| {});
    }
}

fn main() {
    let window_size = Some(Vec2 { x: 1000.0, y: 600.0 });
    eframe::run_native(
        "Atomix ECO Launcher",
        eframe::NativeOptions {
            initial_window_size: window_size,
            min_window_size: window_size,
            max_window_size: window_size,
            resizable: false,
            ..eframe::NativeOptions::default()
        },
        Box::new(|_cc| Box::new(PatcherApp::new())),
    )
}
