use eframe::{egui, emath::Vec2};
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
        window(ctx, frame);
    }
}

pub fn window(ctx: &egui::Context, frame: &mut eframe::Frame) {
    atomix::window_frame(ctx, frame, "Atomix ECO Launcher", |_ui| {
        central_panel(ctx);
    });
}

pub fn central_panel(ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            progress_panel(ui);
        });
}

fn progress_panel(ui: &mut egui::Ui) {
    egui::TopBottomPanel::bottom("progress_panel")
        .resizable(false)
        .min_height(110.)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::LIGHT_GRAY)
                .outer_margin(egui::style::Margin::same(0.))
                .inner_margin(egui::style::Margin::same(15.))
                .rounding(egui::Rounding::same(25.)),
        )
        .show_inside(ui, |ui| {
            ui.centered_and_justified(|ui| {
                play_button_panel(ui);
                patch_progress_bar(ui);
            });
        });
}

fn play_button_panel(ui: &mut egui::Ui) {
    egui::SidePanel::right("progress_bar_panel")
        .min_width(200.)
        .resizable(false)
        .frame(
            egui::Frame::none()
                .inner_margin(egui::style::Margin {
                    left: 50.,
                    right: 10.,
                    top: 10.,
                    bottom: 10.,
                })
                .fill(egui::Color32::TRANSPARENT),
        )
        .show_inside(ui, |ui| {
            ui.centered_and_justified(|ui| {
                ui.style_mut().text_styles = [(
                    egui::TextStyle::Button,
                    egui::FontId::new(60.0, egui::FontFamily::Proportional),
                )]
                .into();
                ui.add(atomix::RoundButton::new("PLAY").rounding(25.));
            });
        });
}

fn patch_progress_bar(ui: &mut egui::Ui) {
    // Progress bar primary color
    ui.style_mut().visuals.selection.bg_fill = egui::Color32::from_rgb(0, 230, 100);
    // Progress bar secondary color
    ui.style_mut().visuals.extreme_bg_color = egui::Color32::GRAY;
    ui.add(atomix::ProgressBar::new(0.25).height(100.).rounding(25.));
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
