use super::check_patches::check_platform_patches;
use super::constants::*;
use super::download;
use super::error::{PatchError, PatchErrorLevel, ToPatchError};
use super::utils::set_executable;
use super::utils::{byte_string, get_platform};
use crate::message::{GUIMessage, PatchMessage, PatchStatus};
use aeco_patch_config::fsobject::*;
use aeco_patch_config::status::ServerStatus;
use std::error::Error;
use std::ffi::OsStr;
use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};
use subprocess::PopenError;

const UPDATE_FILE_EXTENSION: &str = "aecoupdate";

pub struct PatchWorker {
    tx: Sender<PatchMessage>,
    rx: Receiver<GUIMessage>,
    pub self_dir: PathBuf,
    pub self_exe: PathBuf,
    pub client: reqwest::Client,
    pub server_url: reqwest::Url,
    pub game_base_url: reqwest::Url,
    pub game_zip_url: reqwest::Url,
    pub patchlist_url: reqwest::Url,
    pub status_url: reqwest::Url,
    pub patch_url: reqwest::Url,
    pub runtime: tokio::runtime::Runtime,
    pub updated_patcher: Option<PathBuf>,
}

impl PatchWorker {
    pub fn new(
        sender: Sender<PatchMessage>,
        receiver: Receiver<GUIMessage>,
    ) -> Result<Self, Box<dyn Error>> {
        let self_exe = std::env::current_exe()?;
        let self_dir = self_exe
            .parent()
            .ok_or_else(|| "No parent directory for the launcher was found.".to_string())?
            .to_path_buf();

        let server_url = reqwest::Url::parse(PATCH_SERVER)?;
        let game_base_url = server_url.join(BASE_DIR)?;
        let game_zip_url = game_base_url.join(BASE_ZIP)?;
        let meta_url = server_url.join(META_DIR)?;
        let patchlist_url = meta_url.join(PATCHLIST)?;
        let status_url = meta_url.join(STATUS)?;
        let patch_url = server_url.join(PATCH_DIR)?;

        let client = reqwest::Client::builder().build()?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        Ok(Self {
            tx: sender,
            rx: receiver,
            self_dir,
            self_exe,
            client,
            server_url,
            game_base_url,
            game_zip_url,
            patchlist_url,
            status_url,
            patch_url,
            runtime,
            updated_patcher: None,
        })
    }

    /// Send a message to the GUI
    fn send(&self, message: PatchMessage) {
        if let Err(why) = self.tx.send(message) {
            eprintln!("Could not send message from PatchWorker to GUI: {why}");
        }
    }

    /// Send an error to the GUI
    pub fn send_error(&self, text: String) {
        self.send_status(PatchStatus::Error);
        self.send(PatchMessage::Error(text));
    }

    /// Send download information to the GUI
    pub fn send_download(&self, text: String, percentage: f32) {
        self.send(PatchMessage::Downloading(text, percentage));
    }

    /// Send misc information to the GUI
    pub fn send_info(&self, text: String) {
        self.send(PatchMessage::Info(text));
    }

    /// Send information about the result of the patch routine to the GUI
    pub fn send_status(&self, status: PatchStatus) {
        self.send(PatchMessage::PatchStatus(status));
        self.clear_recv();
    }

    fn recv(&self) -> Result<GUIMessage, std::sync::mpsc::RecvError> {
        self.rx.recv()
    }

    fn clear_recv(&self) {
        while self.rx.try_recv().is_ok() {}
    }

    pub fn run(&mut self) {
        self.main_loop();
    }

    fn main_loop(&mut self) {
        // Start by performing the patch check
        let mut message = GUIMessage::Retry;
        loop {
            match message {
                GUIMessage::Retry => {
                    self.send_status(PatchStatus::Working);
                    if let Err(why) = self.patch_routine() {
                        // Communicate error status to the GUI
                        self.send_status(PatchStatus::Error);

                        // Display error message
                        match why.level {
                            PatchErrorLevel::Low => self.send_info(why.friendly_message),
                            PatchErrorLevel::High => self.send_error(why.friendly_message),
                        }

                        // Log more detailed error info to the terminal
                        eprintln!("{:?}", why.internal_error);
                    }
                }
                GUIMessage::Play => {
                    match self.start_game() {
                        Ok(_) => {
                            // The game is running and we can exit
                            std::thread::sleep(std::time::Duration::from_secs(3));
                            self.send_status(PatchStatus::GameLaunched);
                            return;
                        }
                        Err(why) => {
                            // Could not launch the game, need to stay open to inform user
                            self.send_status(PatchStatus::Error);
                            self.send_error("Failed to launch the game".to_string());
                            eprintln!("Failed to launch game: {why}");
                        }
                    }
                }
            }

            message = match self.recv() {
                Ok(m) => m,
                Err(why) => {
                    eprintln!("{why}");
                    return;
                }
            };
        }
    }

    fn patch_routine(&mut self) -> Result<(), PatchError> {
        self.check_patcher_aecoupdate()?;

        self.send_info("Checking server status".to_string());
        let server_status = download::server_status(self)?;

        match server_status {
            ServerStatus::Online => self.send_info("Server is online".to_string()),
            ServerStatus::Maintenance => {
                return Err(Box::<dyn Error>::from(format!(
                    "Received server status {server_status:?}"
                ))
                .to_patch_error_level("Server is down for maintenance", PatchErrorLevel::Low));
            }
        }

        // Make sure the game is installed, and install it if not
        self.ensure_game_installed()?;

        // Get patch information from the patch server
        let patch = download::patch_metadata(self)?;

        // Apply patches for all platforms and for this specific platform
        for platform in ["all", &get_platform()] {
            // Compare local files against the patch data, and update files if needed
            if let Some(platform_dir) = subdir_by_name(&patch, platform) {
                check_platform_patches(self, platform_dir).map_err(|why| {
                    why.to_patch_error(&format!("Failed to check files for platform '{platform}'"))
                })?;
            } else {
                println!("No patch directory found for platform \'{platform}\'");
            }
        }

        self.send_status(PatchStatus::Finished);

        // Open the new patcher if there is one
        if let Some(p) = &self.updated_patcher {
            let error = start_process_and_close(&[p]);
            return Err(error.to_patch_error("Could not start updated launcher"));
        }

        Ok(())
    }

    /// Checks whether the game is in the same directory as this program
    fn is_game_present(&self) -> bool {
        let game_path = self.self_dir.join(GAME_EXE);
        game_path.is_file()
    }

    /// Unpacks the base game ZIP to the same directory as this program
    fn unpack_base(&self, base_file: std::fs::File) -> Result<(), Box<dyn Error>> {
        // Open base game archive
        let mut archive = zip::read::ZipArchive::new(base_file)?;

        self.send_download("Extracting base game".to_string(), 0.);

        // Modified from zip/src/read.rs:extract
        // to provide real-time feedback to the GUI
        let total_archive_count = archive.len();

        // Calculate the total number of bytes to be extracted
        let mut total_archive_bytes = 0;
        for file_number in 0..total_archive_count {
            let file = archive.by_index(file_number)?;
            total_archive_bytes += file.size();
        }

        // Get total number of bytes as a human readable string
        let pretty_total = byte_string(total_archive_bytes);

        // Keep track of how many bytes have been decompressed so far
        let mut decompressed_bytes = 0;

        for file_number in 0..total_archive_count {
            // Report progress in terms of bytes extracted
            let progress = decompressed_bytes as f32 / total_archive_bytes as f32;
            let pretty_decompressed = byte_string(decompressed_bytes);
            self.send_download(
                format!(
                    "Extracting file {} of {} ({pretty_decompressed} / {pretty_total})",
                    file_number + 1,
                    total_archive_count
                ),
                progress,
            );

            // Get the next file from the archive
            let mut file = archive.by_index(file_number)?;

            // Get its path and figure out where it should go on the system
            let filepath = file.enclosed_name().ok_or("Invalid file path")?;
            let outpath = self.self_dir.join(filepath);

            if file.name().ends_with('/') {
                // Create directories if needed
                std::fs::create_dir_all(&outpath)?;
            } else {
                // Create parent directories if needed
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(&p)?;
                    }
                }

                // Copy extracted file to disk
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
                }
            }

            // Keep track of how many bytes have been extracted so far
            decompressed_bytes += file.size();
        }

        self.send_download("Finished installing base game".to_string(), 1.);

        Ok(())
    }

    /// Checks whether the game is installed and installs it if not
    fn ensure_game_installed(&self) -> Result<(), PatchError> {
        self.send_download("Checking game installation".to_string(), 1.);
        if !self.is_game_present() {
            self.send_download("Downloading game since it is not installed".to_string(), 0.);

            // Download the base game
            let game_base_file = download::game_base(self)
                .map_err(|why| why.to_patch_error("Failed while downloading base game"))?;

            // Extract the base game to disk
            self.unpack_base(game_base_file)
                .map_err(|why| why.to_patch_error("Failed while unpacking base game"))?;
        }

        Ok(())
    }

    fn start_game(&self) -> Result<(), Box<dyn Error>> {
        let game_full_path = self.self_dir.join(GAME_EXE);
        let eco = OsStr::new(&game_full_path);
        let launch = OsStr::new("/launch");
        let wine = OsStr::new("wine");
        let args = {
            #[cfg(unix)]
            {
                // TODO: On Unixlike systems, perhaps a new wineprefix should be created
                // TODO: On Unixlike systems, help the user install Wine
                [wine, eco, launch].to_vec()
            }
            #[cfg(windows)]
            {
                [eco, launch].to_vec()
            }
        };

        std::env::set_current_dir(&self.self_dir)?;
        let error = start_process_and_close(&args);
        return Err(error.into());
    }

    fn check_patcher_aecoupdate(&self) -> Result<(), PatchError> {
        // Get current extension, or finish if there is none
        let extension = match self.self_exe.extension() {
            Some(ext) => ext,
            None => {
                // Remove aecoupdate launcher if there is one
                self.remove_aecoupdate_file()
                    .map_err(|why| why.to_patch_error("Failed to remove temporary launcher"))?;
                return Ok(());
            }
        };

        // Finish if the current extension isn't the update extension
        if extension != UPDATE_FILE_EXTENSION {
            return Ok(());
        }

        // Remove the extension
        // This only removes the last extension, so if the file is supposed to
        // have an extension (e.g. ".exe") that part will remain, and it will
        // become the new extension
        let new_file_path = {
            let mut path = self.self_exe.clone();
            path.set_extension("");
            path
        };

        // Copy this program to the normal program filename
        // Try a few times, it is possible that the old process hasn't shut
        // down yet
        let retries = 5;
        for retry in 1..=retries {
            if let Err(why) = std::fs::copy(&self.self_exe, &new_file_path) {
                if retry == retries {
                    return Err(why.to_patch_error("Failed to overwrite patcher"));
                }
                std::thread::sleep(std::time::Duration::from_millis(250));
            } else {
                break;
            }
        }

        // Make sure the file is executable on unixlike systems
        set_executable(&new_file_path)
            .map_err(|why| why.to_patch_error("Failed to make patcher executable"))?;

        // Open the restored launcher and close this one
        let error = start_process_and_close(&[new_file_path]);
        Err(error.to_patch_error("Failed to start new launcher"))
    }

    fn remove_aecoupdate_file(&self) -> Result<(), Box<dyn Error>> {
        let path = self.get_self_aecoupdate_path()?;
        if path.exists() {
            // Try a few times, it is possible that the old process hasn't shut
            // down yet
            let retries = 5;
            for retry in 1..=retries {
                if let Err(why) = std::fs::remove_file(&path) {
                    if retry == retries {
                        return Err(why.into());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(250));
                } else {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn get_self_aecoupdate_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        let current_name = self
            .self_exe
            .file_name()
            .ok_or_else(|| "Failed to get launcher file name".to_string())?
            .to_str()
            .ok_or_else(|| "Failed to read launcher file name as a string".to_string())?;
        let file_name = format!("{current_name}.{UPDATE_FILE_EXTENSION}");
        Ok(self.self_exe.with_file_name(file_name))
    }
}

/// Gets a Directory child from a Directory by name, if it is present
fn subdir_by_name<'a>(dir: &'a Directory, name: &str) -> Option<&'a Directory> {
    for child in &dir.children {
        if let FSObject::Directory(d) = child {
            if d.name == name {
                return Some(d);
            }
        }
    }
    None
}

/// Starts a new process and closes the current one. Any case in which this
/// returns indicates an error.
fn start_process_and_close(args: &[impl AsRef<OsStr>]) -> PopenError {
    match subprocess::Popen::create(args, subprocess::PopenConfig::default()) {
        Ok(mut popen) => {
            // Close this program
            popen.detach();
            std::process::exit(0)
        }
        Err(why) => why,
    }
}
