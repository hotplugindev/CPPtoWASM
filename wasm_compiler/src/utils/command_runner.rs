use std::process::{Command, Output, Stdio};
use std::path::Path;
use std::ffi::OsStr;

pub fn run_command(
    command_name: &str,
    args: &[impl AsRef<OsStr>],
    current_dir: Option<&Path>,
) -> Result<Output, String> {
    log::debug!(
        "Running command: {} {} (in {:?})",
        command_name,
        args.iter().map(|a| a.as_ref().to_string_lossy()).collect::<Vec<_>>().join(" "),
        current_dir.unwrap_or_else(|| Path::new("."))
    );

    let mut cmd = Command::new(command_name);
    cmd.args(args);

    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }

    // Capture stdio for better error reporting
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| {
        format!(
            "Failed to execute command '{}': {}. Is it installed and in your PATH?",
            command_name, e
        )
    })?;

    if output.status.success() {
        log::debug!(
            "Command '{}' executed successfully. Stout: {}",
            command_name,
            String::from_utf8_lossy(&output.stdout)
        );
        Ok(output)
    } else {
        let err_msg = format!(
            "Command '{}' failed with status: {}.\nStdout: {}\nStderr: {}",
            command_name,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        log::error!("{}", err_msg);
        Err(err_msg)
    }
}

pub fn is_command_in_path(command_name: &str) -> bool {
    match Command::new(command_name).arg("--version").output() {
        Ok(_) => true,
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                log::warn!("Command '{}' not found in PATH.", command_name);
                false
            } else {
                // Command might exist but failed for other reasons (e.g. --version not supported)
                // For simplicity, we'll assume it exists if it's not a NotFound error.
                // A more robust check might involve `which` command or PATH environment variable parsing.
                log::debug!("Command '{}' check resulted in error (assuming it exists): {}", command_name, e);
                true
            }
        }
    }
}
