use std::error::Error;
use std::fs::File;
use std::io::Write;

use super::error::{PatchError, ToPatchError};
use super::utils::byte_string;
use super::PatchWorker;
use aeco_patch_config::fsobject::Directory;
use aeco_patch_config::status::ServerStatus;
use futures_util::StreamExt;

pub fn server_status(worker: &PatchWorker) -> Result<ServerStatus, PatchError> {
    let json_bytes = memory_file(worker, worker.status_url.clone(), |_, _| {})
        .map_err(|why| why.to_patch_error("Failed to get server status"))?;

    let server_status = serde_json::from_slice::<ServerStatus>(&json_bytes)
        .map_err(|why| why.to_patch_error("Failed to parse server status"))?;

    Ok(server_status)
}

/// Downloads a file and returns it in a temporary file
pub fn temp_file<F>(
    worker: &PatchWorker,
    url: reqwest::Url,
    callback: F,
) -> Result<std::fs::File, Box<dyn Error>>
where
    F: Fn(u64, Option<u64>), /* downloaded bytes, total bytes */
{
    // Request URL
    let response = worker
        .runtime
        .block_on(worker.client.get(url).send())
        .map_err(|why| why.to_string())?;

    // Check response status
    let status = response.status();
    if !status.is_success() {
        Err(format!("URL request failed: {status}"))?;
    }

    // Create a new temporary file for the data to go into
    let mut file = tempfile::tempfile_in(&worker.self_dir).map_err(|why| why.to_string())?;

    // Keep track of the total size and the number of bytes downloaded so far.
    // The server doesn't need to tell us how long the content is.
    let total_size = response.content_length();
    let mut downloaded_size = 0u64;

    let mut stream = response.bytes_stream();
    while let Some(stream_result) = worker.runtime.block_on(stream.next()) {
        // Get next chunk of bytes from stream
        let bytes = stream_result?;

        // Write the bytes to the file
        file.write_all(&bytes).map_err(|why| why.to_string())?;

        downloaded_size += bytes.len() as u64;

        callback(downloaded_size, total_size);
    }

    Ok(file)
}

/// Downloads a file and returns it in a Vec
pub fn memory_file<F>(
    worker: &PatchWorker,
    url: reqwest::Url,
    callback: F,
) -> Result<Vec<u8>, Box<dyn Error>>
where
    F: Fn(u64, Option<u64>), /* downloaded bytes, total bytes */
{
    // Request URL
    let response = worker.runtime.block_on(worker.client.get(url).send())?;

    // Check response status
    let status = response.status();
    if !status.is_success() {
        return Err(format!("URL request failed: {status}").into());
    }

    // Keep track of the total size and the number of bytes downloaded so far.
    // The server doesn't need to tell us how long the content is.
    let total_size = response.content_length();
    let mut downloaded_size = 0u64;

    // If we know the total size of the download, we can pre-allocate the Vec
    // so there will be no more allocations while downloading
    let mut result = match total_size {
        Some(size) => {
            let size = usize::try_from(size)
                .map_err(|_| "File to download is too large to load into memory".to_string())?;
            Vec::<u8>::with_capacity(size)
        }
        None => Vec::<u8>::new(),
    };

    let mut stream = response.bytes_stream();
    while let Some(stream_result) = worker.runtime.block_on(stream.next()) {
        // Get next chunk of bytes from stream
        let bytes = stream_result.map_err(|why| why.to_string())?;

        // Write the bytes to the Vec
        result.extend(&bytes);

        downloaded_size += bytes.len() as u64;

        callback(downloaded_size, total_size);
    }

    Ok(result)
}

/// Downloads a file and returns the resulting bytes
pub fn patch(worker: &PatchWorker, net_file: reqwest::Url) -> Result<Vec<u8>, Box<dyn Error>> {
    let data = memory_file(worker, net_file, |_, _| {})?;
    Ok(data)
}

pub fn game_base(worker: &PatchWorker) -> Result<File, Box<dyn Error>> {
    temp_file(worker, worker.game_zip_url.clone(), |downloaded, total| {
        let pretty_downloaded = byte_string(downloaded);
        if let Some(total) = total {
            let downloaded = downloaded.min(total);
            let progress = downloaded as f32 / total as f32;
            let pretty_total = byte_string(total);
            worker.send_download(
                format!("Downloading base game ({pretty_downloaded} / {pretty_total})"),
                progress,
            );
        } else {
            worker.send_download(format!("Downloading base game ({pretty_downloaded})"), 1.);
        }
    })
}

/// Downloads the patchlist and returns the parsed result
pub fn patch_metadata(worker: &PatchWorker) -> Result<Directory, PatchError> {
    let result = memory_file(worker, worker.patchlist_url.clone(), |downloaded, total| {
        let pretty_downloaded = byte_string(downloaded);
        if let Some(total) = total {
            let downloaded = downloaded.min(total);
            let progress = downloaded as f32 / total as f32;
            let pretty_total = byte_string(total);
            worker.send_download(
                format!("Downloading patch info ({pretty_downloaded} / {pretty_total})"),
                progress,
            );
        } else {
            worker.send_download(format!("Downloading patch info ({pretty_downloaded})"), 1.);
        }
    });

    let json_bytes = result.map_err(|why| why.to_patch_error("Failed to get patch info"))?;

    let patch_dir = serde_json::from_slice::<Directory>(&json_bytes)
        .map_err(|why| why.to_patch_error("Failed to parse patch info"))?;

    Ok(patch_dir)
}
