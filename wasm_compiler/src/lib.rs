//! `wasm_compiler` is a Rust library and command-line tool designed to compile C++ projects
//! to WebAssembly (WASM). It aims to simplify the process of targeting WASM by wrapping
//! Emscripten toolchain commands and providing sensible defaults for various configurations,
//! including support for CMake, Makefiles, and direct C++ file compilation.
//!
//! ## Features
//! - Compiles C++ projects using CMake.
//! - Compiles C++ projects using Makefiles.
//! - Compiles single C++ files directly.
//! - Supports debug and release build configurations.
//! - Configurable Emscripten flags for fine-tuning.
//! - Basic support for ImGui projects via the `--with-imgui` flag.
//! - Outputs ES6 modules for modern JavaScript interoperability.
//!
//! ## Usage (CLI)
//! ```bash
//! # Assuming wasm_compiler is built and in PATH, or run via 'cargo run --'
//! wasm_compiler --project-path /path/to/cpp-project --output-dir /path/to/output [--with-imgui]
//! ```

pub mod app_config;
pub mod compiler;
pub mod utils;

use app_config::AppConfig;
use compiler::{BuildSystemHandler, cmake_handler::CMakeHandler, make_handler::MakeHandler, emscripten_runner::EmscriptenRunner};
// use std::path::Path; // Not directly used here anymore, but kept for context if needed

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Build system detection failed: {0}")]
    Detection(String),
    #[error("Compilation failed: {0}")]
    Compilation(String),
    #[error("Command execution failed: {0}")] // Retained if direct command usage happens elsewhere
    Command(String),
    #[error("File system operation failed: {0}")]
    FileSystem(String),
}

pub fn run() -> Result<(), Error> {
    // Ensure logger is initialized. If main.rs also does it, this is fine.
    // Consider using `try_init` if multiple initializations are an issue.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).try_init().ok();

    let config = AppConfig::new();

    log::info!("Starting WASM compilation for project at: {:?}", config.project_path);
    log::debug!("Using configuration: {:?}", config);

    if !config.project_path.exists() || !config.project_path.is_dir() {
        return Err(Error::Config(format!(
            "Project path {:?} does not exist or is not a directory.",
            config.project_path
        )));
    }

    // Canonicalize project_path early to resolve symlinks and relative paths.
    let project_path_abs = config.project_path.canonicalize().map_err(Error::Io)?;
    // Create a mutable config if we need to update project_path to its absolute form.
    // Or, pass project_path_abs to handlers and they can use it with original config.
    // For simplicity, let's assume handlers will use the absolute path when needed.

    utils::file_system::ensure_dir_exists(&config.output_dir)
        .map_err(Error::FileSystem)?;

    // 1. Detect build system
    if CMakeHandler::detect(&project_path_abs) {
        log::info!("CMake project detected.");
        let cmake_handler = CMakeHandler::new();
        cmake_handler.compile(&project_path_abs, &config).map_err(Error::Compilation)?;
    } else if MakeHandler::detect(&project_path_abs) {
        log::info!("Makefile project detected.");
        let make_handler = MakeHandler::new();
        make_handler.compile(&project_path_abs, &config).map_err(Error::Compilation)?;
    } else {
        log::warn!("No CMakeLists.txt or Makefile found. Attempting to find a C++ source file to compile directly.");

        let mut cpp_file_to_compile: Option<std::path::PathBuf> = None;
        for entry in walkdir::WalkDir::new(&project_path_abs).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "cpp" || ext == "cxx" || ext == "cc" {
                        if entry.file_name().to_string_lossy().contains("main") {
                            cpp_file_to_compile = Some(entry.path().to_path_buf());
                            break;
                        }
                        if cpp_file_to_compile.is_none() {
                             cpp_file_to_compile = Some(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }

        if let Some(source_file) = cpp_file_to_compile {
            log::info!("Found source file: {:?}. Attempting direct Emscripten compilation.", source_file);
            let em_runner = EmscriptenRunner::new();
            // Pass the whole config to compile_file
            em_runner.compile_file(&source_file, &config)
                .map_err(Error::Compilation)?;
            log::info!("Direct compilation successful.");
        } else {
             return Err(Error::Detection(
                "No CMakeLists.txt, Makefile, or obvious C++ source file found in the project root.".to_string()
            ));
        }
    }

    log::info!(
        "Compilation process finished. Output should be in {:?} (check for {}.js and {}.wasm)",
        config.output_dir, config.output_name, config.output_name
    );

    Ok(())
}
