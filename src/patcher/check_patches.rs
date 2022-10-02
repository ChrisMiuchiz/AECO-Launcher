use std::{error::Error, path::Path};

use crate::patcher::utils::set_executable;
use aeco_patch_config::fsobject::Archive;
use aeco_patch_config::fsobject::{Directory, FSObject, File};

use super::download;
use super::PatchWorker;

#[derive(Debug)]
struct ArchivePaths<'a, 'b> {
    pub hed: &'a Path,
    pub dat: &'b Path,
}

/// Checks files to be patched, patching them if necessary
pub fn check_platform_patches(
    worker: &mut PatchWorker,
    dir: &Directory,
) -> Result<(), Box<dyn Error>> {
    let check_platform = &dir.name;

    // The URL to start at needs to be patch/platform because that is where
    // platform specific files are stored
    let platform_net_path = worker.patch_url.join(&format!("{check_platform}/"))?;

    let total_files = get_total_files_in_patch(dir);
    let disk_dir = worker.self_dir.clone();
    let checked_files = check_dir(
        worker,
        dir,
        disk_dir,
        platform_net_path,
        check_platform,
        0,
        total_files,
    )?;

    // All files should have been checked, but it is not fatal if these
    // values do not match
    if checked_files != total_files {
        eprintln!(
            "Checked files: {checked_files}; total files: {total_files}. These should match."
        );
    }

    worker.send_download(format!("{total_files} files checked."), 1.);

    Ok(())
}

/// Iterates through a directory for files to be patched
fn check_dir<P>(
    worker: &mut PatchWorker,
    dir: &Directory,
    disk_dir: P,
    net_path: reqwest::Url,
    platform: &str,
    mut completed_files: usize,
    total_files: usize,
) -> Result<usize, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    for child in &dir.children {
        completed_files = match child {
            FSObject::File(file) => {
                let file_net_path = net_path.join(&file.name)?;
                let file_disk_path = disk_dir.as_ref().join(&file.name);

                check_file(
                    worker,
                    file,
                    file_disk_path,
                    file_net_path,
                    platform,
                    completed_files,
                    total_files,
                )?
            }
            FSObject::Directory(d) => {
                // Dir URLS need / to be parsed correctly
                let directory_net_path = net_path.join(&format!("{}/", &d.name))?;
                let directory_disk_path = disk_dir.as_ref().join(&d.name);

                check_dir(
                    worker,
                    d,
                    directory_disk_path,
                    directory_net_path,
                    platform,
                    completed_files,
                    total_files,
                )?
            }
            FSObject::Archive(a) => {
                let archive_paths = ArchivePaths {
                    hed: &disk_dir.as_ref().join(&a.name).with_extension("hed"),
                    dat: &disk_dir.as_ref().join(&a.name).with_extension("dat"),
                };

                // Dir URLs need / to be parsed correctly, and archives are stored online as .archive
                let archive_net_path = net_path.join(&format!("{}.archive/", &a.name))?;

                check_archive(
                    worker,
                    a,
                    &archive_paths,
                    archive_net_path,
                    platform,
                    completed_files,
                    total_files,
                )?
            }
        }
    }

    Ok(completed_files)
}

/// Checks whether a file should be patched, patching it if necessary
fn check_file<P>(
    worker: &mut PatchWorker,
    file: &File,
    disk_file: P,
    net_file: reqwest::Url,
    platform: &str,
    mut completed_files: usize,
    total_files: usize,
) -> Result<usize, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let file_to_check = disk_file.as_ref();
    let mut file_to_write = file_to_check.to_path_buf();

    send_checked_files_update(worker, completed_files + 1, total_files, platform);

    if !file_to_write.exists() {
        println!("Downloading new file {net_file} -> {:?}", &file_to_write);
        let file_bytes = download::patch(worker, net_file)?;
        std::fs::write(file_to_write, file_bytes)?;
    } else {
        let file_matches = {
            let disk_data = std::fs::read(&file_to_check)?;
            let disk_file_data = File::new(&file.name, &disk_data);
            file.digest == disk_file_data.digest
        };

        let is_self = file_to_check == worker.self_exe;

        // If the patched file is this program, don't try to overwrite it
        // while it is running. Instead, save it as a different file name
        // and move it later.
        if is_self {
            file_to_write = worker.get_self_aecoupdate_path()?;
        }

        if !file_matches {
            println!("Updating {net_file} -> {:?}", &file_to_write);
            let file_bytes = download::patch(worker, net_file)?;
            std::fs::write(&file_to_write, file_bytes)?;
            // If we got the file successfully, and it is a replacement for
            // this program, save the path to the new one for later so we
            // can switch to it.
            if is_self {
                // Make sure the file is exectuable on unixlike systems
                set_executable(&file_to_write)?;
                worker.updated_patcher = Some(file_to_write);
            }
        }
    }

    completed_files += 1;

    Ok(completed_files)
}

/// Iterates through an archive checking for files to be patched, patching them if necessary
fn check_archive(
    worker: &mut PatchWorker,
    archive: &Archive,
    archive_paths: &ArchivePaths,
    net_path: reqwest::Url,
    platform: &str,
    mut completed_files: usize,
    total_files: usize,
) -> Result<usize, Box<dyn Error>> {
    // Open the ECO archive
    let mut disk_archive = aeco_archive::Archive::open_pair(archive_paths.dat, archive_paths.hed)
        .map_err(|_| format!("Couldn't open archive {}", &archive.name))?;

    // Keep track of if changes were made to this archive.
    // If changes were made, we will need to finalize (and defrag!) the archive
    // afterwards.
    let mut changes_made = false;

    // Go through each of the files in the patch's archive info
    for file in &archive.files {
        // Update the GUI to display how many files have been checked so far
        send_checked_files_update(worker, completed_files + 1, total_files, platform);

        // Figure out if the file in the archive matches the one stored on the
        // server. If a file is not present in the archive at all, that is
        // considered to not match.
        let file_matches = file_matches_in_archive(&disk_archive, file).map_err(|_| {
            // TODO: ArchiveError should impl Error
            format!("Failed while reading {}", archive.name)
        })?;

        // If the file in the archive is outdated, download it and insert it
        // into the archive on disk.
        if !file_matches {
            let new_file_url = net_path.join(&file.name)?;
            println!("Downloading {new_file_url} -> {archive_paths:?}");
            let new_file_bytes = download::patch(worker, new_file_url)?;
            disk_archive
                .add_file(&file.name, &new_file_bytes)
                .map_err(|_| format!("Couldn't write to archive {}", &archive.name))?;
            changes_made = true;
        }

        completed_files += 1;
    }

    // If the archive on disk has been altered, make sure changes get saved,
    // and make sure that any wasted space gets elimintated.
    if changes_made {
        disk_archive
            .finalize()
            .map_err(|_| format!("Couldn't finalize archive {}", &archive.name))?;
        disk_archive
            .defrag()
            .map_err(|_| format!("Couldn't defrag archive {}", &archive.name))?;
    }

    Ok(completed_files)
}

/// Reads an ECO archive and checks if a file inside it matches the given File
/// info.
fn file_matches_in_archive(
    disk_archive: &aeco_archive::Archive,
    file: &File,
) -> Result<bool, aeco_archive::ArchiveError> {
    match disk_archive.get_file(&file.name) {
        Ok(archive_data) => {
            // File is present in the archive
            let archive_file = File::new(&file.name, &archive_data);
            // Is it the same as the one on the server?
            Ok(file.digest == archive_file.digest)
        }
        Err(aeco_archive::ArchiveError::FileNotPresentError) => {
            // The file is not present, so it doesn't match
            Ok(false)
        }
        Err(why) => {
            // Some other error happened
            Err(why)
        }
    }
}

fn get_total_files_in_patch(dir: &Directory) -> usize {
    let mut total_files = 0;

    for child in &dir.children {
        total_files += match child {
            FSObject::File(_) => 1,
            FSObject::Directory(d) => get_total_files_in_patch(d),
            FSObject::Archive(a) => a.files.len(),
        };
    }

    total_files
}

fn send_checked_files_update(
    worker: &PatchWorker,
    files_checked: usize,
    total_files: usize,
    platform: &str,
) {
    let progress = files_checked as f32 / total_files as f32;
    worker.send_download(
        format!("Checking file {files_checked} / {total_files} for platform '{platform}'",),
        progress,
    );
}
