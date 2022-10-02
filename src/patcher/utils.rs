use std::path::Path;

/// Format a quantity of bytes into a human readable string
pub fn byte_string<T>(bytes: T) -> String
where
    T: Into<u128>,
{
    byte_unit::Byte::from_bytes(bytes.into())
        .get_appropriate_unit(true) // binary units
        .to_string()
}

/// Gets a platform string to represent the current platform
/// Some examples: `windows-x86_64`, `linux-x86`, `macos-aarch64`
pub fn get_platform() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    format!("{os}-{arch}")
}

/// Sets a file to be executable on Unixlikes, or does nothing on Windows
pub fn set_executable<P>(path: P) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o764))?;
    }
    Ok(())
}
