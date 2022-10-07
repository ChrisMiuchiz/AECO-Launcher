pub enum TreeState {
    Clean,
    Dirty,
}

// These env vars are set at compile time by build.rs

/// Returns the version of the crate as specified in Cargo.toml
pub fn pkg_version() -> Option<&'static str> {
    std::option_env!("CARGO_PKG_VERSION")
}

/// Returns the full git hash
pub fn git_hash() -> Option<&'static str> {
    std::option_env!("GIT_HASH")
}

/// Returns `TreeState::Clean` if the source tree has no modifications,
/// returns `TreeState::Dirty` otherwise.
pub fn tree_state() -> Option<TreeState> {
    let git_changes: Option<&'static str> = std::option_env!("GIT_CHANGES");

    git_changes.map(|x| {
        if x != "0" {
            TreeState::Dirty
        } else {
            TreeState::Clean
        }
    })
}

/// Returns a snippet from the beginning of a git hash
pub fn short_hash() -> Option<&'static str> {
    let long_hash = match git_hash() {
        Some(x) => x,
        None => return None,
    };

    let start_index = 0;
    let end_index = 7.min(long_hash.len());

    Some(&long_hash[start_index..end_index])
}

/// Produces a summary of the crate version as a string
/// Example: `0.1.0-2cfe000-dirty`
pub fn version_summary() -> String {
    let dirty = "dirty";
    let mut pieces = Vec::<&str>::with_capacity(3);

    if let Some(version) = pkg_version() {
        pieces.push(version);
    }

    if let Some(hash) = short_hash() {
        pieces.push(hash);
    }

    if let Some(TreeState::Dirty) = tree_state() {
        pieces.push(dirty);
    }

    pieces.join("-")
}
