use std::path::PathBuf;
use std::process::Command;

pub fn is_dir(path: &str) -> Result<PathBuf, String> {
    let path: PathBuf = path.into();

    if path.is_dir() {
        Ok(path)
    } else {
        Err(String::from(
            "The path provided is not a directory. Please provide a directory path.",
        ))
    }
}

pub enum CheckResult {
    Ok(String),
    NotFound,
}

#[cfg(unix)]
/// Checks whether jpegoptim exists on the path.
/// Returns the path to `jpegoptim` if it exists.
/// Returns [`CheckResult::NotFound`] if it does not exist.
pub fn check_jpegoptim() -> CheckResult {
    let cmd = Command::new("bash")
        .arg("-c")
        .arg("which jpegoptim")
        .output();

    match cmd {
        Ok(output) => {
            if output.status.success() {
                let path = String::from_utf8(output.stdout).unwrap();
                CheckResult::Ok(path.trim().to_string())
            } else {
                CheckResult::NotFound
            }
        }
        Err(_) => CheckResult::NotFound,
    }
}

#[cfg(target_os = "windows")]
/// Checks whether jpegoptim.exe exists on the path.
/// Returns the path to `jpegoptim.exe` if it exists.
/// Returns [`CheckResult::NotFound`] if it does not exist.
pub fn check_jpegoptim() -> CheckResult {
    todo!("Not yet implemented")
    // FIXME: This should attempt to first use `pwsh.exe`, then `cmd.exe`, to find a `jpegoptim.exe` in the PATH.
    // Maybe there is a nicer way, instead of having to do that? I hope so!
}
