use eframe::{
    egui::{self, Ui},
    emath::Vec2,
};
mod atomix;

struct PatcherApp {}

impl PatcherApp {
    fn new() -> PatcherApp {
        PatcherApp {}
    }
}

impl eframe::App for PatcherApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        egui::Rgba::TRANSPARENT // Make sure we don't paint anything behind the rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        atomix::window_frame(ctx, frame, "Atomix ECO Launcher", |ui| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    // Progress bar primary color
                    ui.style_mut().visuals.selection.bg_fill = egui::Color32::from_rgb(0, 230, 100);
                    // Progress bar secondary color
                    ui.style_mut().visuals.extreme_bg_color = egui::Color32::GRAY;
                    egui::TopBottomPanel::bottom("progress_panel")
                        .resizable(false)
                        .min_height(125.)
                        .frame(
                            egui::Frame::none()
                                .fill(egui::Color32::LIGHT_GRAY)
                                .outer_margin(egui::style::Margin::same(0.))
                                .inner_margin(egui::style::Margin::same(15.))
                                .rounding(egui::Rounding::same(25.)),
                        )
                        .show_inside(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.add(atomix::ProgressBar::new(0.25).height(115.).rounding(25.))
                            });
                        });
                });
        });
    }
}

fn main() {
    let window_size = Some(Vec2 {
        x: 1000.0,
        y: 600.0,
    });
    eframe::run_native(
        "Atomix ECO Launcher",
        eframe::NativeOptions {
            initial_window_size: window_size,
            min_window_size: window_size,
            max_window_size: window_size,
            resizable: false,
            // Hide the OS-specific "chrome" around the window:
            decorated: false,
            // To have rounded corners we need transparency:
            transparent: true,
            ..eframe::NativeOptions::default()
        },
        Box::new(|_cc| Box::new(PatcherApp::new())),
    )
}
