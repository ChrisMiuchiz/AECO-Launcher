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

    let patchworker = match patcher::PatchWorker::new(patch_tx, gui_rx) {
        Ok(patchworker) => patchworker,
        Err(why) => {
            eprintln!("Could not initialize patch worker: {why}");
            return;
        }
    };

    // Check for whether the patcher is a temporary updated one before creating
    // a GUI.
    // If an error occurs here, run the GUI anyway. The patchworker will do this
    // operation again, and if it fails again, it will be able to display an
    // error message to the user.
    match patchworker.check_patcher_aecoupdate() {
        Ok(patcher::RunState::Close) => return,
        Ok(patcher::RunState::Continue) => {}
        Err(why) => eprintln!("{:?}", why.internal_error),
    }

    std::thread::spawn(move || patchworker.run());
    ui::PatcherUI::run(gui_tx, patch_rx);
}
