//! The `compiler` module contains the core logic for handling different C++ project
//! build systems (like CMake, Make) and orchestrating the compilation process
//! using Emscripten.

pub mod cmake_handler;
pub mod emscripten_runner;
pub mod make_handler;

use crate::app_config::AppConfig;
use std::path::Path;

/// A trait representing a handler for a specific build system.
///
/// Each build system (like CMake or Make) will have an implementation of this trait
/// to detect if a project uses that system and to perform the compilation steps.
pub trait BuildSystemHandler {
    /// Detects if the given project path is managed by this build system.
    ///
    /// # Arguments
    /// * `project_path` - The root path of the C++ project.
    ///
    /// # Returns
    /// `true` if the build system is detected, `false` otherwise.
    fn detect(project_path: &Path) -> bool where Self: Sized;

    /// Compiles the project using this build system and Emscripten.
    ///
    /// # Arguments
    /// * `project_path` - The root path of the C++ project.
    /// * `config` - The application configuration containing build settings.
    ///
    /// # Returns
    /// A `Result` indicating success or an error message string.
    fn compile(&self, project_path: &Path, config: &AppConfig) -> Result<(), String>;
}
