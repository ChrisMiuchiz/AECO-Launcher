use crate::message::{GUIMessage, PatchMessage, PatchStatus};
use crate::version::version_summary;
use eframe::{egui, emath::Vec2};
use std::sync::mpsc::{Receiver, Sender};
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

enum ProgressBarState {
    Downloading(String, f32),
    Connecting(String),
    Error(String),
}

enum PlayButtonState {
    Disabled,
    Play,
    Retry,
}

impl ProgressBarState {
    pub fn background_color(&self) -> egui::Color32 {
        egui::Color32::GRAY
    }

    pub fn foreground_color(&self) -> egui::Color32 {
        match &self {
            ProgressBarState::Downloading(_, _) => egui::Color32::from_rgb(0x4e, 0x80, 0x4e),
            ProgressBarState::Connecting(_) => egui::Color32::from_rgb(0xF0, 0xD0, 0x90),
            ProgressBarState::Error(_) => egui::Color32::from_rgb(0xD0, 0x80, 0x80),
        }
    }

    pub fn text_color(&self) -> egui::Color32 {
        match &self {
            ProgressBarState::Downloading(_, _) => egui::Color32::WHITE,
            ProgressBarState::Connecting(_) => egui::Color32::DARK_GRAY,
            ProgressBarState::Error(_) => egui::Color32::WHITE,
        }
    }

    pub fn amount(&self) -> f32 {
        match &self {
            ProgressBarState::Downloading(_, amount) => *amount,
            ProgressBarState::Connecting(_) => 1.,
            ProgressBarState::Error(_) => 1.,
        }
    }

    pub fn text(&self) -> &String {
        match &self {
            ProgressBarState::Downloading(s, _) => s,
            ProgressBarState::Connecting(s) => s,
            ProgressBarState::Error(s) => s,
        }
    }
}

pub struct PatcherUI {
    tx: Sender<GUIMessage>,
    rx: Receiver<PatchMessage>,
    background_handle: Option<egui::TextureHandle>,
    link_bar_color: egui::Color32,
    username: String,
    password: String,
    progress_bar_state: ProgressBarState,
    play_button_state: PlayButtonState,
    program_version: String,
}

impl PatcherUI {
    pub fn new(sender: Sender<GUIMessage>, receiver: Receiver<PatchMessage>) -> PatcherUI {
        PatcherUI {
            tx: sender,
            rx: receiver,
            background_handle: None,
            link_bar_color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 240),
            username: String::new(),
            password: String::new(),
            progress_bar_state: ProgressBarState::Connecting(
                "Waiting for patch server...".to_string(),
            ),
            play_button_state: PlayButtonState::Disabled,
            program_version: version_summary(),
        }
    }

    pub fn run(sender: Sender<GUIMessage>, receiver: Receiver<PatchMessage>) {
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
            Box::new(|_cc| Box::new(PatcherUI::new(sender, receiver))),
        );
    }

    fn load_background(&mut self, ui: &mut egui::Ui) -> egui::TextureHandle {
        if let Some(handle) = &self.background_handle {
            return handle.clone();
        }
        let bg_img = load_image_from_memory(include_bytes!("../../assets/top_bg.png"))
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

    fn handle_messages(&mut self, frame: &mut eframe::Frame) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                PatchMessage::Error(message) => {
                    self.progress_bar_state = ProgressBarState::Error(message);
                }
                PatchMessage::Downloading(message, progress) => {
                    self.progress_bar_state = ProgressBarState::Downloading(message, progress);
                }
                PatchMessage::Info(message) => {
                    self.progress_bar_state = ProgressBarState::Connecting(message);
                }
                PatchMessage::PatchStatus(status) => {
                    match status {
                        PatchStatus::Finished => {
                            self.progress_bar_state =
                                ProgressBarState::Downloading("Ready!".to_string(), 1.);
                            self.play_button_state = PlayButtonState::Play;
                        }
                        PatchStatus::Working => {
                            self.play_button_state = PlayButtonState::Disabled;
                        }
                        PatchStatus::Error => {
                            self.play_button_state = PlayButtonState::Retry;
                        }
                        PatchStatus::Close => {
                            // We are done!
                            frame.close();
                        }
                    }
                }
            }
        }
    }

    fn window(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.handle_messages(frame);
        atomix::window_frame(ctx, frame, "Atomix ECO Launcher", |ui| {
            self.background(ui);
            self.central_panel(ui);
        });
    }

    fn background(&mut self, ui: &mut egui::Ui) {
        let bg_handle = self.load_background(ui);

        // The image should be as wide as the window can contain.
        // Calculate how tall it needs to be without stretching.
        let image_size = bg_handle.size_vec2();
        let fit_x = ui.available_width();
        let scale = fit_x / image_size.x;
        let fit_y = image_size.y * scale;

        let max_rect = ui.max_rect();
        let top = max_rect.top();
        let left = max_rect.left();
        let right = max_rect.right();
        let bottom = max_rect.bottom();
        let bg_rect = egui::Rect {
            min: egui::Pos2 { x: left, y: top },
            max: egui::Pos2 {
                x: right,
                y: fit_y.min(bottom - 25.),
            },
        };

        egui::Image::new(&bg_handle, bg_handle.size_vec2()).paint_at(ui, bg_rect);
    }

    fn central_panel(&mut self, ui: &mut egui::Ui) {
        self.bottom_panel(ui);
        self.login_panel(ui);
    }

    fn login_panel(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("login_panel_top")
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::WHITE)
                    .outer_margin(egui::style::Margin {
                        left: 350.,
                        right: 350.,
                        top: 100.,
                        bottom: 100.,
                    })
                    .inner_margin(20.)
                    .rounding(25.)
                    .shadow(egui::epaint::Shadow {
                        extrusion: 30.,
                        color: egui::Color32::DARK_GRAY,
                    }),
            )
            .show_inside(ui, |ui| {
                ui.style_mut().text_styles = [(
                    egui::TextStyle::Body,
                    egui::FontId::new(32.0, egui::FontFamily::Proportional),
                )]
                .into();

                ui.style_mut().visuals.extreme_bg_color = egui::Color32::LIGHT_GRAY;

                ui.add(egui::Label::new("Username"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.username).text_color(egui::Color32::BLACK),
                );

                ui.add(egui::Label::new("Password"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .text_color(egui::Color32::BLACK)
                        .password(true),
                );
            });
    }

    fn bottom_panel(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(
                egui::Frame::none()
                    .fill(self.link_bar_color)
                    .rounding(egui::Rounding {
                        nw: 0.,
                        ne: 0.,
                        sw: 25.,
                        se: 25.,
                    }),
            )
            .resizable(false)
            .min_height(144.)
            .show_inside(ui, |ui| {
                self.progress_panel(ui);
                self.links_panel(ui);
            });
    }

    fn links_panel(&self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("links_panel")
            .frame(egui::Frame::none().inner_margin(15.))
            .show_inside(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.style_mut().text_styles = [
                        // URL buttons
                        (
                            egui::TextStyle::Button,
                            egui::FontId::new(32.0, egui::FontFamily::Proportional),
                        ),
                        // Version label
                        (
                            egui::TextStyle::Body,
                            egui::FontId::new(16.0, egui::FontFamily::Proportional),
                        ),
                    ]
                    .into();

                    ui.style_mut().visuals.widgets.noninteractive.bg_stroke =
                        egui::Stroke::new(1., egui::Color32::GRAY);

                    // Control panel link
                    if ui
                        .add(egui::Button::new("Control Panel").fill(egui::Color32::TRANSPARENT))
                        .clicked()
                    {
                        open::that("https://ecocp.atomixro.com").ok();
                    }

                    ui.separator();

                    // Registration link
                    if ui
                        .add(egui::Button::new("Register").fill(egui::Color32::TRANSPARENT))
                        .clicked()
                    {
                        open::that("https://ecocp.atomixro.com/register").ok();
                    }

                    // Version string
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                        ui.label(&self.program_version);
                    });
                });
            });
    }

    fn progress_panel(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::bottom("progress_panel_inner")
            .resizable(false)
            .min_height(80.)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::LIGHT_GRAY)
                    .outer_margin(egui::style::Margin::same(0.))
                    .inner_margin(egui::style::Margin::same(15.))
                    .rounding(egui::Rounding::same(25.)),
            )
            .show_inside(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    self.play_button_panel(ui);
                    self.patch_progress_bar(ui);
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
                        left: 35.,
                        right: 10.,
                        top: 4.,
                        bottom: 4.,
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
                    self.play_button(ui);
                });
            });
    }

    fn send(&self, message: GUIMessage) {
        if let Err(why) = self.tx.send(message) {
            eprintln!("Could not send message from GUI to PatchWorker: {why}");
        }
    }

    fn play_button(&mut self, ui: &mut egui::Ui) {
        let rounding = 25.;
        match self.play_button_state {
            PlayButtonState::Disabled => {
                ui.add(
                    atomix::RoundButton::new("WAIT")
                        .rounding(rounding)
                        .sense(egui::Sense::hover()),
                );
            }
            PlayButtonState::Play => {
                if ui
                    .add(atomix::RoundButton::new("PLAY").rounding(rounding))
                    .clicked()
                {
                    self.send(GUIMessage::Play);
                }
            }
            PlayButtonState::Retry => {
                if ui
                    .add(atomix::RoundButton::new("RETRY").rounding(rounding))
                    .clicked()
                {
                    self.send(GUIMessage::Retry);
                }
            }
        };
    }

    fn patch_progress_bar(&mut self, ui: &mut egui::Ui) {
        // Progress bar primary color
        ui.style_mut().visuals.selection.bg_fill = self.progress_bar_state.foreground_color();
        // Progress bar secondary color
        ui.style_mut().visuals.extreme_bg_color = self.progress_bar_state.background_color();
        ui.style_mut().text_styles = [(
            egui::TextStyle::Button,
            egui::FontId::new(32.0, egui::FontFamily::Proportional),
        )]
        .into();
        ui.style_mut().visuals.override_text_color = Some(self.progress_bar_state.text_color());
        ui.add(
            atomix::ProgressBar::new(self.progress_bar_state.amount())
                .height(72.)
                .rounding(25.)
                .text(self.progress_bar_state.text()),
        );
    }
}

impl eframe::App for PatcherUI {
    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        egui::Rgba::TRANSPARENT // Make sure we don't paint anything behind the rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.window(ctx, frame);
    }
}

impl Drop for PatcherUI {
    fn drop(&mut self) {
        self.send(GUIMessage::Close);
    }
}
