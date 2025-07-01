use std::path::Path;
use crate::app_config::AppConfig;
use super::BuildSystemHandler;
use super::emscripten_runner::EmscriptenRunner; // Import EmscriptenRunner
use crate::utils::file_system;
use std::fs;

pub struct MakeHandler;

impl BuildSystemHandler for MakeHandler {
    fn detect(project_path: &Path) -> bool {
        project_path.join("Makefile").exists() || project_path.join("makefile").exists()
    }

    fn compile(&self, project_path: &Path, config: &AppConfig) -> Result<(), String> {
        log::info!("Compiling project with Make at: {:?}", project_path);
        if !Self::detect(project_path) {
            return Err("Makefile not found.".to_string());
        }

        file_system::ensure_dir_exists(&config.output_dir)?;

        // For Makefile projects, emmake handles wrapping most things.
        // We need to pass relevant emcc flags. This can be done by:
        // 1. Setting environment variables like CFLAGS, CXXFLAGS, LDFLAGS before calling `emmake make`.
        // 2. Passing them as arguments to `make`, e.g., `emmake make CXX=em++ CXXFLAGS="..." LDFLAGS="..."`
        // Emscripten's `emmake` sets `CC`, `CXX`, `LD`, etc., to `emcc`, `em++`, `emlink` internally.
        // So, we mainly need to provide additional flags.

        let mut make_args: Vec<String> = Vec::new();
        make_args.push("make".to_string()); // The command emmake will run

        // Construct CXXFLAGS and LDFLAGS strings
        // Most of the get_base_emcc_args are linker flags or general compiler options.
        // We might need to separate them if Makefile distinguishes CFLAGS/CXXFLAGS from LDFLAGS.
        // For simplicity, let's try passing most as CXXFLAGS and some specific linker flags as LDFLAGS.

        let mut cxx_flags = Vec::new();
        let mut ld_flags = Vec::new();

        // Common flags (optimization, debug, exceptions)
        match config.build_config.to_lowercase().as_str() {
            "debug" => {
                cxx_flags.push("-g4".to_string());
                cxx_flags.push("-O0".to_string());
                cxx_flags.push("-sASSERTIONS=2".to_string());
                cxx_flags.push("-sSAFE_HEAP=1".to_string()); // Good for debugging
            }
            "release" => {
                cxx_flags.push("-O3".to_string());
                cxx_flags.push("-sASSERTIONS=0".to_string());
                ld_flags.push("--llvm-lto=1".to_string());
            }
            _ => {
                cxx_flags.push("-O2".to_string());
                cxx_flags.push("-sASSERTIONS=1".to_string());
            }
        }
        cxx_flags.push("-fwasm-exceptions".to_string());

        // Linker specific flags for JS interop and output naming
        ld_flags.push(format!("-sMODULARIZE=1"));
        ld_flags.push(format!("-sEXPORT_ES6=1"));
        ld_flags.push(format!("-sENVIRONMENT={}", match config.target_env.to_lowercase().as_str() {
            "web" => "web",
            "node" => "node",
            _ => "web,node"
        }));
        ld_flags.push("-sEXPORTED_RUNTIME_METHODS=FS,callMain,setValue,getValue,UTF8ToString,stringToUTF8".to_string());

        // Output for Makefiles is trickier if the Makefile itself defines the output location.
        // We aim for the final linked product to be named according to config.output_name and be in config.output_dir.
        // One common way is to pass `TARGET=<name>` or `OUT=<dir>` to make if the Makefile supports it.
        // If not, we might have to find the output file and copy it.
        // For now, let's assume the Makefile builds something like `a.out` or a specific target,
        // and we'll try to control the final linking step's output name if possible.
        // This often requires modifying the Makefile or hoping it uses LDFLAGS for the output command.

        // Add user-defined emcc flags
        if let Some(user_flags_str) = &config.emcc_flags {
            for flag in user_flags_str.split_whitespace() {
                // Heuristic: if it starts with -o or is known linker flag, add to LDFLAGS
                if flag.starts_with("-o") || flag.starts_with("-s") || flag.contains("LINK") || flag.contains("LTO") {
                    ld_flags.push(flag.to_string());
                } else {
                    cxx_flags.push(flag.to_string());
                }
            }
        }

        // Important: The final output naming with `-o <file>.js` and `-sWASM_BINARY_NAME`
        // must be part of the LDFLAGS for the final link command.

        // Add ImGui specific flags if enabled
        if config.with_imgui {
            log::info!("ImGui support enabled for Make, adding specific linker and compiler flags.");
            ld_flags.push("-sUSE_GLFW=3".to_string());
            ld_flags.push("-sUSE_WEBGL2=1".to_string());
            ld_flags.push("-sFULL_ES3=1".to_string());
            ld_flags.push("-sGL_ENABLE_GET_PROC_ADDRESS=1".to_string());
            ld_flags.push("-sALLOW_MEMORY_GROWTH=1".to_string());
            if !config.emcc_flags.as_deref().unwrap_or("").contains("EXPORT_NAME") &&
               !ld_flags.iter().any(|arg| arg.contains("EXPORT_NAME")) {
                ld_flags.push("-sEXPORT_NAME='Module'".to_string());
            }
            ld_flags.push("-sUSE_SDL=2".to_string());
            ld_flags.push("-sINITIAL_MEMORY=67108864".to_string());

            // Add GL_ASSERTIONS to CXXFLAGS for debug builds with ImGui
            if config.build_config.to_lowercase().as_str() == "debug" {
                if !cxx_flags.contains(&"-sGL_ASSERTIONS=1".to_string()) {
                    cxx_flags.push("-sGL_ASSERTIONS=1".to_string());
                }
            }
        }

        // Ensure user-provided emcc_flags are de-duplicated if already added by ImGui
        if let Some(user_flags_str) = &config.emcc_flags {
            for flag_str in user_flags_str.split_whitespace() {
                let flag = flag_str.to_string();
                // Heuristic: if it starts with -o or is known linker flag, add to LDFLAGS
                if flag.starts_with("-o") || flag.starts_with("-s") || flag.contains("LINK") || flag.contains("LTO") {
                    if !ld_flags.contains(&flag) {
                        ld_flags.push(flag);
                    }
                } else {
                    if !cxx_flags.contains(&flag) {
                        cxx_flags.push(flag);
                    }
                }
            }
        }

        let output_js_name_for_ld = format!("{}.js", config.output_name); // This will be relative to where make runs link step
        ld_flags.push("-o".to_string());
        ld_flags.push(output_js_name_for_ld.clone()); // Make will create this in its build dir
        ld_flags.push(format!("-sWASM_BINARY_NAME={}.wasm", config.output_name));


        if !cxx_flags.is_empty() {
            make_args.push(format!("CXXFLAGS={}", cxx_flags.join(" ")));
            make_args.push(format!("CFLAGS={}", cxx_flags.join(" "))); // Apply to C files too
        }
        if !ld_flags.is_empty() {
            make_args.push(format!("LDFLAGS={}", ld_flags.join(" ")));
        }

        // Optionally, allow specifying a make target
        // make_args.push("all"); // or some default target

        log::debug!("Running emmake with args: {:?}", make_args.join(" "));
        // `emmake` needs to be run from the project path where Makefile exists.
        EmscriptenRunner::run_emscripten_tool("emmake", &make_args, project_path, config)?;

        log::info!("Make project build command executed via emmake.");

        // After `emmake make` finishes, the output files (`output_name.js`, `output_name.wasm`)
        // should be in the `project_path` (or wherever Makefile places its output, typically CWD).
        // We then copy them to the configured `output_dir`.

        let built_js_path = project_path.join(format!("{}.js", config.output_name));
        let built_wasm_path = project_path.join(format!("{}.wasm", config.output_name));

        let dest_js_path = config.output_dir.join(format!("{}.js", config.output_name));
        let dest_wasm_path = config.output_dir.join(format!("{}.wasm", config.output_name));

        if built_js_path.exists() {
            fs::copy(&built_js_path, &dest_js_path)
                .map_err(|e| format!("Failed to copy JS from {:?} to {:?}: {}", built_js_path, dest_js_path, e))?;
            log::info!("Copied JS to {:?}", dest_js_path);
        } else {
            return Err(format!("Expected JS output file not found after make: {:?}", built_js_path));
        }

        if built_wasm_path.exists() {
            fs::copy(&built_wasm_path, &dest_wasm_path)
                .map_err(|e| format!("Failed to copy WASM from {:?} to {:?}: {}", built_wasm_path, dest_wasm_path, e))?;
            log::info!("Copied WASM to {:?}", dest_wasm_path);
        } else {
            return Err(format!("Expected WASM output file not found after make: {:?}", built_wasm_path));
        }

        // Clean up build artifacts from source directory? Optional.
        // fs::remove_file(&built_js_path).ok();
        // fs::remove_file(&built_wasm_path).ok();
        // fs::remove_file(project_path.join(format!("{}.html", config.output_name))).ok(); // If HTML is generated
        // fs::remove_file(project_path.join(format!("{}.worker.js", config.output_name))).ok(); // If pthreads worker is generated

        log::info!("Successfully compiled Makefile project. Output in {:?}", config.output_dir);
        Ok(())
    }
}

impl MakeHandler {
    pub fn new() -> Self {
        MakeHandler
    }
}
