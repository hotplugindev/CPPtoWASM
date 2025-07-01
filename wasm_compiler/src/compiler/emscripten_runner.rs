use std::path::{Path, PathBuf};
use crate::app_config::AppConfig;
use crate::utils::command_runner::{self, run_command};
use crate::utils::file_system;

pub struct EmscriptenRunner;

impl EmscriptenRunner {
    pub fn new() -> Self {
        EmscriptenRunner
    }

    fn get_base_emcc_args(config: &AppConfig, output_name: &str) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        // Output WASM and JS file
        // ... (comments as before)

        // Standard Library & Runtime
        // ... (comments as before)
        // args.push("-sFILESYSTEM=1".to_string());

        // Threading:
        // ... (comments as before)

        // Exception Handling:
        args.push("-fwasm-exceptions".to_string());

        // Memory Management:
        // args.push("-sALLOW_MEMORY_GROWTH=1".to_string()); // Default in newer Emscripten often, but good to be explicit if needed.
                                                          // For ImGui, memory growth can be important.

        // Optimization & Size
        match config.build_config.to_lowercase().as_str() {
            "debug" => {
                args.push("-g4".to_string());
                args.push("-O0".to_string());
                args.push("-sASSERTIONS=2".to_string());
                args.push("-sSAFE_HEAP=1".to_string());
                args.push("-sGL_ASSERTIONS=1".to_string()); // Good for ImGui debugging
            }
            "release" => {
                args.push("-O3".to_string());
                args.push("-sASSERTIONS=0".to_string());
                args.push("--llvm-lto".to_string());
            }
            _ => {
                args.push("-O2".to_string());
                args.push("-sASSERTIONS=1".to_string());
            }
        }

        // JS Interop & Environment
        args.push("-sMODULARIZE=1".to_string());
        args.push("-sEXPORT_ES6=1".to_string());
        match config.target_env.to_lowercase().as_str() {
            "web" => args.push("-sENVIRONMENT=web".to_string()),
            "node" => args.push("-sENVIRONMENT=node".to_string()),
            // ... (wasi comments as before)
            _ => args.push("-sENVIRONMENT=web,node".to_string()),
        }
        args.push("-sEXPORTED_RUNTIME_METHODS=FS,callMain,setValue,getValue,UTF8ToString,stringToUTF8".to_string());
        args.push(format!("-sWASM_BINARY_NAME={}.wasm", output_name));


        // Third-party libs / UI specific flags
        if config.with_imgui {
            log::info!("ImGui support enabled, adding specific Emscripten flags.");
            args.push("-sUSE_GLFW=3".to_string());      // Use Emscripten's GLFW emulation for window/input
            args.push("-sUSE_WEBGL2=1".to_string());    // Prefer WebGL2
            args.push("-sFULL_ES3=1".to_string());      // Request full GLES3 features for WebGL2
            // args.push("-sMIN_WEBGL_VERSION=2".to_string()); // Explicitly require WebGL2
            args.push("-sGL_ENABLE_GET_PROC_ADDRESS=1".to_string()); // Needed by some GL loaders
            args.push("-sALLOW_MEMORY_GROWTH=1".to_string()); // ImGui can use a fair bit of memory

            // For older ImGui examples or simpler WebGL1 contexts, one might use:
            // args.push("-sLEGACY_GL_EMULATION=1".to_string());
            // args.push("-sGL_VERSION=2".to_string()); // For GLES2/WebGL1

            // Export the main loop function if the C++ code uses emscripten_set_main_loop
            // This is often the case with ImGui examples.
            if !config.emcc_flags.as_deref().unwrap_or("").contains("EXPORT_NAME") &&
               !args.iter().any(|arg| arg.contains("EXPORT_NAME")) { // Avoid duplicate if user adds it
                args.push("-sEXPORT_NAME='Module'".to_string()); // Default export name for emscripten_set_main_loop related things
            }
             // Common ImGui examples might require these for the main loop and canvas setup
            args.push("-sUSE_SDL=2".to_string()); // ImGui examples often use SDL for windowing/input even with GLFW for GL
                                               // If using SDL for event handling with ImGui.
                                               // This pulls in SDL2 which provides a main loop and event handling.
                                               // If the C++ code directly uses emscripten_set_main_loop with manual canvas,
                                               // then USE_SDL might not be strictly necessary, but often helps.
            args.push("-sINITIAL_MEMORY=67108864".to_string()); // 64MB initial memory, ImGui can be memory hungry
        }

        // Add any user-specified flags last, so they can override defaults
        if let Some(flags_str) = &config.emcc_flags {
            for flag in flags_str.split_whitespace() {
                // Avoid duplicating flags if they were already added by with_imgui logic
                if !args.contains(&flag.to_string()) {
                    args.push(flag.to_string());
                }
            }
        }

        args
    }

    pub fn compile_file(
        &self,
        source_file: &Path,
        config: &AppConfig,
    ) -> Result<PathBuf, String> {
        log::info!("Compiling single file with emcc: {:?}", source_file);

        if !command_runner::is_command_in_path("emcc") {
            return Err("emcc not found in PATH. Please ensure Emscripten SDK is installed and configured.".to_string());
        }

        file_system::ensure_dir_exists(&config.output_dir)?;

        let output_js_target_path = config.output_dir.join(format!("{}.js", config.output_name));
        let output_wasm_target_path = config.output_dir.join(format!("{}.wasm", config.output_name));

        let mut emcc_args = Self::get_base_emcc_args(config, &config.output_name);
        emcc_args.insert(0, source_file.to_string_lossy().to_string());
        emcc_args.push("-o".to_string());
        emcc_args.push(output_js_target_path.to_string_lossy().to_string());

        log::debug!("Running emcc with args: {:?}", emcc_args.join(" "));

        match run_command("emcc", &emcc_args, Some(config.project_path.as_path())) {
            Ok(_output) => {
                log::info!("File compiled successfully. JS output: {:?}, WASM output: {:?}",
                    output_js_target_path, output_wasm_target_path);
                if output_wasm_target_path.exists() {
                    Ok(output_wasm_target_path)
                } else {
                    Err(format!("WASM file {:?} not found after compilation, though emcc succeeded. Check emcc flags.", output_wasm_target_path))
                }
            }
            Err(e) => {
                log::error!("emcc compilation failed: {}", e);
                Err(format!("emcc compilation failed: {}", e))
            }
        }
    }

    pub fn run_emscripten_tool(
        tool: &str, // "emcc", "em++", "emcmake", "emmake", "emar", etc.
        args: &[String],
        current_dir: &Path,
        config: &AppConfig, // Pass config for context if needed for env vars or toolchain paths
    ) -> Result<String, String> {
        if !command_runner::is_command_in_path(tool) {
            return Err(format!(
                "{} not found in PATH. Please ensure Emscripten SDK is installed and configured.",
                tool
            ));
        }

        // Potentially set Emscripten-specific environment variables if not using emcmake/emmake
        // e.g., EMCC_CFLAGS, if the tool doesn't automatically pick up the toolchain.
        // For emcmake and emmake, they handle setting up the environment for cmake/make.

        log::info!("Executing Emscripten tool: {} {} in {:?}", tool, args.join(" "), current_dir);

        // Create a string representation of the args for logging/error messages
        // let args_str_vec: Vec<&str> = args.iter().map(AsRef::as_ref).collect();


        match run_command(tool, args, Some(current_dir)) {
            Ok(output) => {
                let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
                if !stderr_str.is_empty() && !output.status.success() { // emcmake might print to stderr on success
                    log::warn!("{} execution produced stderr:\n{}", tool, stderr_str);
                }
                log::info!("{} executed successfully. Output:\n{}", tool, stdout_str);
                Ok(stdout_str)
            }
            Err(e) => {
                log::error!("{} execution failed: {}", tool, e);
                Err(format!("{} execution failed: {}", tool, e))
            }
        }
    }
}
