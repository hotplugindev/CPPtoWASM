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

    let mut cmd = Command::new(resolve_emscripten_tool(command_name));
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
    // For Emscripten tools, use a different approach since they don't all support --version
    if is_emscripten_tool(command_name) {
        return is_emscripten_tool_available(command_name);
    }

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

fn is_emscripten_tool(command_name: &str) -> bool {
    matches!(command_name, "emcc" | "em++" | "emmake" | "emcmake" | "emar" | "emranlib" | "emlink" | "emsize" | "emstrip")
}

fn is_emscripten_tool_available(command_name: &str) -> bool {
    // For emmake and emcmake, try running them without arguments - they should show usage
    if matches!(command_name, "emmake" | "emcmake") {
        let tool_name = resolve_emscripten_tool(command_name);
        match Command::new(&tool_name).output() {
            Ok(output) => {
                // These tools show usage when run without args and exit with non-zero status
                // But if they run and produce output, they exist
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined_output = format!("{} {}", stdout, stderr);
                
                if combined_output.contains("helper") || 
                   combined_output.contains("usage") ||
                   combined_output.contains("emscripten") ||
                   combined_output.contains("make") ||
                   combined_output.contains("FLAGS") {
                    log::debug!("Emscripten tool '{}' found and working", command_name);
                    true
                } else {
                    log::warn!("Command '{}' exists but doesn't appear to be working correctly", command_name);
                    false
                }
            }
            Err(e) => {
                if let std::io::ErrorKind::NotFound = e.kind() {
                    log::warn!("Command '{}' not found in PATH.", command_name);
                    false
                } else {
                    log::debug!("Command '{}' check resulted in error (assuming it exists): {}", command_name, e);
                    true
                }
            }
        }
    } else {
        // For other emscripten tools, try --version or help
        match Command::new(command_name).arg("--version").output() {
            Ok(_) => true,
            Err(e) => {
                if let std::io::ErrorKind::NotFound = e.kind() {
                    log::warn!("Command '{}' not found in PATH.", command_name);
                    false
                } else {
                    // Try with --help for tools that don't support --version
                    match Command::new(command_name).arg("--help").output() {
                        Ok(_) => {
                            log::debug!("Emscripten tool '{}' found (via --help)", command_name);
                            true
                        }
                        Err(_) => {
                            log::debug!("Command '{}' check resulted in error (assuming it exists): {}", command_name, e);
                            true
                        }
                    }
                }
            }
        }
    }
}

/// Resolves the correct Emscripten tool name for the current platform.
/// On Windows, appends `.bat` for emscripten wrapper tools (emmake, emcmake, etc).
pub fn resolve_emscripten_tool(tool: &str) -> String {
    if cfg!(windows) {
        match tool {
            "emmake" | "emcmake" | "emcc" | "em++" | "emar" | "emranlib" | "emlink" | "emsize" | "emstrip" => format!("{}.bat", tool),
            _ => tool.to_string(),
        }
    } else {
        tool.to_string()
    }
}
