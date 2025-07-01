use std::path::Path;
use crate::app_config::AppConfig;
use super::BuildSystemHandler;
use super::emscripten_runner::EmscriptenRunner; // Import EmscriptenRunner
use crate::utils::file_system;

pub struct CMakeHandler;

impl BuildSystemHandler for CMakeHandler {
    fn detect(project_path: &Path) -> bool {
        project_path.join("CMakeLists.txt").exists()
    }

    fn compile(&self, project_path: &Path, config: &AppConfig) -> Result<(), String> {
        log::info!("Compiling project with CMake at: {:?}", project_path);
        if !Self::detect(project_path) {
            return Err("CMakeLists.txt not found.".to_string());
        }

        let build_dir_name = "build_wasm_cmake"; // More specific name
        let build_dir = project_path.join(build_dir_name);
        file_system::ensure_dir_exists(&build_dir)?;
        file_system::ensure_dir_exists(&config.output_dir)?; // Ensure final output dir exists

        // 1. Configure with emcmake
        // `emcmake cmake <path_to_source> -B<path_to_build_dir> [options]`
        let mut cmake_args: Vec<String> = Vec::new();
        cmake_args.push(project_path.to_string_lossy().into_owned());
        cmake_args.push(format!("-DCMAKE_BUILD_TYPE={}", config.build_config));

        // Add Emscripten specific CMake flags. These flags are passed to CMake,
        // which then uses them to configure the Emscripten toolchain.
        // The actual emcc flags for compiling sources will be mostly handled by Emscripten's toolchain file.
        // However, we might want to pass some high-level options.
        // For example, if linking to specific libraries or setting definitions.
        // cmake_args.push(format!("-DEMSCRIPTEN_OUTPUT_NAME={}", config.output_name));

        // If using `-s` flags directly with emcmake, they might not always propagate as expected.
        // It's usually better to set these via CMAKE_CXX_FLAGS or target_link_options in CMakeLists.txt
        // or rely on Emscripten's toolchain defaults.
        // However, some global `-s` flags can be passed via EMMAKEN_CFLAGS or EMMAKEN_LDFLAGS environment variables
        // or by setting CMAKE_EXE_LINKER_FLAGS.

        // Example of setting linker flags that contain Emscripten -s options:
        // Note: This is one way; using a custom toolchain file or modifying CMakeLists.txt is often cleaner.
        let mut emcc_link_flags = Vec::new();
        // emcc_link_flags.push("-sALLOW_MEMORY_GROWTH=1".to_string());
        emcc_link_flags.push(format!("-sMODULARIZE=1"));
        emcc_link_flags.push(format!("-sEXPORT_ES6=1"));
        emcc_link_flags.push(format!("-sENVIRONMENT={}", match config.target_env.to_lowercase().as_str() {
            "web" => "web",
            "node" => "node",
            _ => "web,node" // Default
        }));
        emcc_link_flags.push(format!("-sEXPORTED_RUNTIME_METHODS=FS,callMain,setValue,getValue,UTF8ToString,stringToUTF8"));
        emcc_link_flags.push(format!("-o"));
        let output_js_in_build_dir = build_dir.join(format!("{}.js", config.output_name));
        emcc_link_flags.push(output_js_in_build_dir.to_string_lossy().into_owned());
        emcc_link_flags.push(format!("-sWASM_BINARY_NAME={}.wasm", config.output_name));


        match config.build_config.to_lowercase().as_str() {
            "debug" => {
                emcc_link_flags.push("-g4".to_string());
                emcc_link_flags.push("-O0".to_string());
                emcc_link_flags.push("-sASSERTIONS=2".to_string());
            }
            "release" => {
                emcc_link_flags.push("-O3".to_string());
                emcc_link_flags.push("--llvm-lto=1".to_string()); // Enable LTO for CMake
                emcc_link_flags.push("-sASSERTIONS=0".to_string());
            }
            _ => {
                emcc_link_flags.push("-O2".to_string());
                emcc_link_flags.push("-sASSERTIONS=1".to_string());
            }
        }

        if let Some(user_flags) = &config.emcc_flags {
            for flag in user_flags.split_whitespace() {
                emcc_link_flags.push(flag.to_string());
            }
        }

        // Setting CMAKE_EXE_LINKER_FLAGS to pass these flags to the linker invocation
        // This is generally more reliable for -s flags than trying to pass them as compiler flags.

        // Add ImGui specific flags if enabled
        if config.with_imgui {
            log::info!("ImGui support enabled for CMake, adding specific linker flags.");
            emcc_link_flags.push("-sUSE_GLFW=3".to_string());
            emcc_link_flags.push("-sUSE_WEBGL2=1".to_string());
            emcc_link_flags.push("-sFULL_ES3=1".to_string());
            emcc_link_flags.push("-sGL_ENABLE_GET_PROC_ADDRESS=1".to_string());
            emcc_link_flags.push("-sALLOW_MEMORY_GROWTH=1".to_string());
            if !config.emcc_flags.as_deref().unwrap_or("").contains("EXPORT_NAME") &&
               !emcc_link_flags.iter().any(|arg| arg.contains("EXPORT_NAME")) {
                 emcc_link_flags.push("-sEXPORT_NAME='Module'".to_string());
            }
            emcc_link_flags.push("-sUSE_SDL=2".to_string());
            emcc_link_flags.push("-sINITIAL_MEMORY=67108864".to_string());
            if config.build_config.to_lowercase().as_str() == "debug" {
                 emcc_link_flags.push("-sGL_ASSERTIONS=1".to_string());
            }
        }

        // Ensure user-provided emcc_flags are added (and de-duplicated if already added by ImGui)
        if let Some(user_flags) = &config.emcc_flags {
            for flag_str in user_flags.split_whitespace() {
                if !emcc_link_flags.contains(&flag_str.to_string()) {
                    emcc_link_flags.push(flag_str.to_string());
                }
            }
        }

        cmake_args.push(format!("-DCMAKE_EXE_LINKER_FLAGS={}", emcc_link_flags.join(" ")));
        // Alternative: Set CMAKE_CXX_FLAGS for compiler-specific flags, CMAKE_C_FLAGS for C
        // cmake_args.push(format!("-DCMAKE_CXX_FLAGS_INIT=\"{}\"", compiler_flags_str));

        log::debug!("Running emcmake cmake with args: {:?}", cmake_args.join(" "));
        EmscriptenRunner::run_emscripten_tool("emcmake", &["cmake".to_string()].iter().chain(cmake_args.iter()).cloned().collect::<Vec<String>>(), &build_dir, config)?;

        // 2. Build with emmake or directly with chosen generator (e.g., ninja)
        // `emmake make` or `cmake --build .` if Ninja or another generator is used
        // For simplicity, using `cmake --build .` which works with Makefiles, Ninja, etc.
        // Emscripten's emmake is essentially a wrapper for make.
        // Using `cmake --build` is often more portable across generators.
        // The environment variables set by `emcmake` should persist for this call if it's a child process.
        // However, to be certain, it's better to wrap the build command with `emmake` if using Makefiles,
        // or ensure the toolchain is correctly picked up if using Ninja.
        // Let's use `cmake --build . --config <BUILD_TYPE>`
        // The `-DCMAKE_BUILD_TYPE` in the configure step is for single-config generators like Makefiles.
        // For multi-config generators (like Visual Studio), `--config` in build step is used.
        // For emscripten with Makefiles/Ninja, CMAKE_BUILD_TYPE is usually sufficient.

        let build_tool_args = vec!["--build".to_string(), ".".to_string(), "--config".to_string(), config.build_config.clone()];
        log::debug!("Running cmake --build with args: {:?}", build_tool_args.join(" "));
        // We need to run this build command also within an emscripten environment,
        // so `emcc`/`em++` are used as compilers by `make` or `ninja`.
        // `emcmake` sets up the environment for `cmake` to generate the build files correctly.
        // The build tool (`make` or `ninja`) then needs to run. `emmake make` is one way.
        // If using `cmake --build .`, it calls the underlying build system.
        // We might need `emmake` if the generator is Makefiles.
        // A common pattern is `emcmake cmake ..` then `emmake make`.
        // If Ninja is the generator: `emcmake cmake .. -G Ninja` then `ninja`. (emmake ninja might not be standard)
        // For now, let's assume `emmake make` is the most common for simple projects.
        // If CMakeLists.txt specifies Ninja, this might need adjustment.
        // A more robust approach would be to detect the generator or allow user to specify.
        // For now, stick to `emmake make` if makefiles are default, or `cmake --build .` and hope emcc is picked up.
        // Let's try `emmake make` first.

        let make_args = vec!["make".to_string()]; // Add verbosity or specific targets if needed e.g. "VERBOSE=1"
        log::debug!("Running emmake make with args: {:?}", make_args.join(" "));
        EmscriptenRunner::run_emscripten_tool("emmake", &make_args, &build_dir, config)?;

        log::info!("CMake project built successfully in {:?}", build_dir);

        // 3. Copy artifacts to the final output directory
        // The output name from emcc flags was set to `build_dir/output_name.js` and `.wasm`
        let src_js = output_js_in_build_dir.clone();
        let src_wasm = build_dir.join(format!("{}.wasm", config.output_name));
        // let src_html = build_dir.join(format!("{}.html", config.output_name)); // If emcc generated one

        let dest_js = config.output_dir.join(format!("{}.js", config.output_name));
        let dest_wasm = config.output_dir.join(format!("{}.wasm", config.output_name));
        // let dest_html = config.output_dir.join(format!("{}.html", config.output_name));

        if src_js.exists() {
            std::fs::copy(&src_js, &dest_js)
                .map_err(|e| format!("Failed to copy JS file {:?} to {:?}: {}", src_js, dest_js, e))?;
            log::info!("Copied {:?} to {:?}", src_js, dest_js);
        } else {
            return Err(format!("Expected JS output file not found: {:?}", src_js));
        }

        if src_wasm.exists() {
            std::fs::copy(&src_wasm, &dest_wasm)
                .map_err(|e| format!("Failed to copy WASM file {:?} to {:?}: {}", src_wasm, dest_wasm, e))?;
            log::info!("Copied {:?} to {:?}", src_wasm, dest_wasm);
        } else {
            // Some emcc configurations might embed WASM in JS, or not produce a separate .wasm if only a .js target is specified.
            // Our flags (-sWASM_BINARY_NAME) should ensure a separate .wasm file.
            return Err(format!("Expected WASM output file not found: {:?}", src_wasm));
        }

        // if src_html.exists() {
        //     std::fs::copy(&src_html, &dest_html)
        //         .map_err(|e| format!("Failed to copy HTML file: {}", e))?;
        // }

        log::info!("Successfully compiled CMake project. Output in {:?}", config.output_dir);
        Ok(())
    }
}

impl CMakeHandler {
    pub fn new() -> Self {
        CMakeHandler
    }
}
