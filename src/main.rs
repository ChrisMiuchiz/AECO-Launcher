use eframe::{egui, emath::Vec2};
mod atomix;

fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

struct PatcherApp {
    background_handle: Option<egui::TextureHandle>,
    link_bar_color: egui::Color32,
}

impl PatcherApp {
    fn new() -> PatcherApp {
        PatcherApp {
            background_handle: None,
            link_bar_color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 240),
        }
    }

    pub fn load_background(&mut self, ui: &mut egui::Ui) -> egui::TextureHandle {
        if let Some(handle) = &self.background_handle {
            return handle.clone();
        }
        let bg_img = load_image_from_memory(include_bytes!("../assets/top_bg.png"))
            .expect("Background texture should be valid");
        let handle = ui
            .ctx()
            .load_texture("background", bg_img, egui::TextureFilter::Linear);
        self.background_handle = Some(handle);
        self.background_handle
            .as_ref()
            .expect("Background was just loaded")
            .clone()
    }

    pub fn window(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        atomix::window_frame(ctx, frame, "Atomix ECO Launcher", |ui| {
            self.background(ui);
            self.central_panel(ui);
        });
    }

    fn background(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let bg_handle = self.load_background(ui);

        let max_rect = ui.max_rect();
        let top = max_rect.top();
        let left = max_rect.left();
        let right = max_rect.right();
        let bottom = max_rect.bottom();
        let bg_rect = egui::Rect {
            min: egui::Pos2 { x: left, y: top },
            max: egui::Pos2 {
                x: right,
                y: bottom - 50.,
            },
        };

        egui::Image::new(&bg_handle, size).paint_at(ui, bg_rect);
    }

    pub fn central_panel(&mut self, ui: &mut egui::Ui) {
        self.progress_panel(ui);
        self.links_panel(ui);
    }

    fn links_panel(&self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::bottom("links_panel")
            .frame(
                egui::Frame::none()
                    .fill(self.link_bar_color)
                    .inner_margin(15.),
            )
            .show_inside(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.style_mut().text_styles = [(
                        egui::TextStyle::Button,
                        egui::FontId::new(32.0, egui::FontFamily::Proportional),
                    )]
                    .into();
                    ui.style_mut().visuals.widgets.noninteractive.bg_stroke =
                        egui::Stroke::new(1., egui::Color32::GRAY);
                    ui.add(egui::Button::new("Control Panel").fill(egui::Color32::TRANSPARENT));
                    ui.separator();
                    ui.add(egui::Button::new("Register").fill(egui::Color32::TRANSPARENT));
                });
            });
    }

    fn progress_panel(&mut self, ui: &mut egui::Ui) {
        // Make progress panel touch above link panel
        ui.spacing_mut().item_spacing = Vec2 { x: 0., y: -0.444 };
        let rounding = 25.;
        egui::TopBottomPanel::bottom("progress_panel_outer")
            .resizable(false)
            .min_height(110.)
            .frame(
                egui::Frame::none()
                    .fill(self.link_bar_color)
                    .rounding(egui::Rounding {
                        nw: 0.,
                        ne: 0.,
                        sw: rounding,
                        se: rounding,
                    }),
            )
            .show_inside(ui, |ui| {
                egui::TopBottomPanel::bottom("progress_panel_inner")
                    .resizable(false)
                    .min_height(110.)
                    .frame(
                        egui::Frame::none()
                            .fill(egui::Color32::LIGHT_GRAY)
                            .outer_margin(egui::style::Margin::same(0.))
                            .inner_margin(egui::style::Margin::same(15.))
                            .rounding(egui::Rounding::same(rounding)),
                    )
                    .show_inside(ui, |ui| {
                        ui.centered_and_justified(|ui| {
                            self.play_button_panel(ui);
                            self.patch_progress_bar(ui);
                        });
                    });
            });
    }

    fn play_button_panel(&mut self, ui: &mut egui::Ui) {
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

    fn patch_progress_bar(&mut self, ui: &mut egui::Ui) {
        // Progress bar primary color
        ui.style_mut().visuals.selection.bg_fill = egui::Color32::from_rgb(0, 230, 100);
        // Progress bar secondary color
        ui.style_mut().visuals.extreme_bg_color = egui::Color32::GRAY;
        ui.add(atomix::ProgressBar::new(0.25).height(100.).rounding(25.));
    }
}

impl eframe::App for PatcherApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        egui::Rgba::TRANSPARENT // Make sure we don't paint anything behind the rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.window(ctx, frame);
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
