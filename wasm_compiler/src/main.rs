use std::process::Command;

use wasm_compiler::Error;

fn main() {
    // Initialize logger globally, if not already done by the library
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if let Err(e) = wasm_compiler::run() {
        log::error!("Application error: {}", e);
        match e {
            Error::Io(io_err) => eprintln!("Error: A file system I/O error occurred: {}", io_err),
            Error::Config(msg) => eprintln!("Error: Configuration issue: {}", msg),
            Error::Detection(msg) => eprintln!("Error: Build system detection failed: {}", msg),
            Error::Compilation(msg) => eprintln!("Error: Compilation process failed: {}", msg),
            Error::Command(msg) => eprintln!("Error: External command execution failed: {}", msg),
            Error::FileSystem(msg) => eprintln!("Error: File system operation failed: {}", msg),
        }
        std::process::exit(1);
    }
}
