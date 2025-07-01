use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct AppConfig {
    /// Path to the C++ project directory
    #[clap(short, long, value_parser)]
    pub project_path: PathBuf,

    /// Output directory for the WASM build
    #[clap(short, long, value_parser, default_value = "dist")]
    pub output_dir: PathBuf,

    /// Build configuration (e.g., Debug, Release)
    #[clap(short, long, value_parser, default_value = "Release")]
    pub build_config: String,

    /// Target WASM environment (e.g., web, wasi)
    #[clap(short, long, value_parser, default_value = "web")]
    pub target_env: String,

    /// Enable support for ImGui (adds necessary WebGL/GLFW flags)
    #[clap(long)]
    pub with_imgui: bool,

    /// Additional emcc flags (space-separated)
    #[clap(long)]
    pub emcc_flags: Option<String>,

    /// Optional: Path to a specific Emscripten config file (not yet implemented)
    #[clap(long)]
    pub emscripten_config: Option<PathBuf>,

    /// Optional: Name of the final .wasm / .js file
    #[clap(long, default_value = "output")]
    pub output_name: String,
}

impl AppConfig {
    pub fn new() -> Self {
        AppConfig::parse()
    }
}
