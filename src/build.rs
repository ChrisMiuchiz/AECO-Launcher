use std::process::Command;

fn main() {
    git_hash();
    git_changes();
    println!("cargo:rustc-rerun-if-changed=.git/HEAD");
}

/// Optionally emits GIT_HASH containing the full hash of the latest commit.
fn git_hash() {
    let output = match Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        Ok(x) => x,
        Err(_) => return,
    };

    let git_hash = match String::from_utf8(output.stdout) {
        Ok(x) => x,
        Err(_) => return,
    };

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}

/// Optionally emits GIT_CHANGES as 0 or 1 depending on whether
/// there are uncommitted changes or diffs
fn git_changes() {
    let output = match Command::new("git")
        .args(&["diff", "--cached", "--name-status"])
        .output()
    {
        Ok(x) => x,
        Err(_) => return,
    };

    let git_uncommitted = match String::from_utf8(output.stdout) {
        Ok(x) => x,
        Err(_) => return,
    };

    let output = match Command::new("git").args(&["diff"]).output() {
        Ok(x) => x,
        Err(_) => return,
    };

    let git_diff = match String::from_utf8(output.stdout) {
        Ok(x) => x,
        Err(_) => return,
    };

    let git_changes: u8 = if git_uncommitted.trim().is_empty() && git_diff.trim().is_empty() {
        0
    } else {
        1
    };
    println!("cargo:rustc-env=GIT_CHANGES={}", git_changes);
}
