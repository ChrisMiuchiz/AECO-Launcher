// Don't open a command prompt on Windows
#![windows_subsystem = "windows"]

mod message;
mod patcher;
mod ui;
use message::{GUIMessage, PatchMessage};
use std::sync::mpsc::channel;

fn main() {
    let (gui_tx, gui_rx) = channel::<GUIMessage>();
    let (patch_tx, patch_rx) = channel::<PatchMessage>();

    let mut patchworker = match patcher::PatchWorker::new(patch_tx, gui_rx) {
        Ok(patchworker) => patchworker,
        Err(why) => {
            eprintln!("Could not initialize patch worker: {why}");
            return;
        }
    };

    std::thread::spawn(move || patchworker.run());
    ui::PatcherUI::run(gui_tx, patch_rx);
}
