use std::path::Path;
use crate::app_config::AppConfig;
use crate::utils::command_runner::resolve_emscripten_tool;
use crate::compiler::emscripten_runner::EmscriptenRunner;
use super::LibraryHandler;

pub struct ImGuiHandler;

impl ImGuiHandler {
    pub fn new() -> Self {
        ImGuiHandler
    }
    
    fn find_source_files(&self, project_path: &Path, sources: &mut Vec<std::path::PathBuf>, config: &AppConfig) -> Result<(), String> {
        // Read the project directory and find all C++ source files
        let entries = std::fs::read_dir(project_path)
            .map_err(|e| format!("Failed to read project directory: {}", e))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "cpp" || extension == "cxx" || extension == "cc" || extension == "c" {
                        sources.push(path);
                    }
                }
            }
        }
        
        // Also look for ImGui source files in typical locations
        let imgui_dir = project_path.join("..").join("..");
        if imgui_dir.exists() {
            let imgui_sources = vec![
                ("imgui.cpp", imgui_dir.join("imgui.cpp")),
                ("imgui_demo.cpp", imgui_dir.join("imgui_demo.cpp")),
                ("imgui_draw.cpp", imgui_dir.join("imgui_draw.cpp")),
                ("imgui_tables.cpp", imgui_dir.join("imgui_tables.cpp")),
                ("imgui_widgets.cpp", imgui_dir.join("imgui_widgets.cpp")),
            ];
            
            for (name, path) in imgui_sources {
                if path.exists() {
                    log::debug!("Found ImGui source: {}", name);
                    sources.push(path);
                }
            }
            
            // Add compatible backend implementations based on project analysis
            self.add_compatible_backends(project_path, &imgui_dir, sources, config)?;
        }
        
        Ok(())
    }
    
    fn add_compatible_backends(&self, project_path: &Path, imgui_dir: &Path, sources: &mut Vec<std::path::PathBuf>, config: &AppConfig) -> Result<(), String> {
        let backends_dir = imgui_dir.join("backends");
        if !backends_dir.exists() {
            return Ok(());
        }
        
        // Determine which backends are needed based on project analysis
        let needed_backends = self.determine_needed_backends(project_path, config)?;
        
        for backend_name in needed_backends {
            let backend_path = backends_dir.join(&backend_name);
            if backend_path.exists() {
                log::info!("Including compatible backend: {}", backend_name);
                sources.push(backend_path);
            } else {
                log::warn!("Required backend not found: {}", backend_name);
            }
        }
        
        Ok(())
    }
    
    fn determine_needed_backends(&self, project_path: &Path, config: &AppConfig) -> Result<Vec<String>, String> {
        let mut backends = Vec::new();
        
        // Analyze main.cpp and other source files to understand what's being used
        let main_cpp = project_path.join("main.cpp");
        let mut uses_sdl = false;
        let mut uses_glfw = false;
        let mut uses_opengl2 = false;
        let mut uses_opengl3 = false;
        let mut sdl_version = 3; // Default to SDL3 since that's what we configure
        
        if main_cpp.exists() {
            let content = std::fs::read_to_string(&main_cpp)
                .map_err(|e| format!("Failed to read main.cpp: {}", e))?;
            
            // Check for SDL usage
            if content.contains("SDL_") || content.contains("#include <SDL") || content.contains("#include \"SDL") {
                uses_sdl = true;
                
                // Determine SDL version based on includes and API usage
                if content.contains("SDL2/") || content.contains("SDL_WINDOW_") {
                    sdl_version = 2;
                } else if content.contains("SDL3/") || content.contains("SDL_CreateWindow") {
                    sdl_version = 3;
                }
            }
            
            // Check for GLFW usage
            if content.contains("glfw") || content.contains("GLFW") {
                uses_glfw = true;
            }
            
            // Check for OpenGL version usage
            if content.contains("GL_VERSION_2") || content.contains("glBegin") || content.contains("glVertex") {
                uses_opengl2 = true;
            }
            if content.contains("GL_VERSION_3") || content.contains("glGenVertexArrays") || content.contains("glUseProgram") {
                uses_opengl3 = true;
            }
            
            // Check ImGui backend includes to determine what's actually being used
            if content.contains("imgui_impl_sdl2") {
                uses_sdl = true;
                sdl_version = 2;
            }
            if content.contains("imgui_impl_sdl3") {
                uses_sdl = true;
                sdl_version = 3;
            }
            if content.contains("imgui_impl_glfw") {
                uses_glfw = true;
            }
            if content.contains("imgui_impl_opengl2") {
                uses_opengl2 = true;
            }
            if content.contains("imgui_impl_opengl3") {
                uses_opengl3 = true;
            }
        }
        
        // For web/Emscripten builds, prefer SDL3 and OpenGL3/WebGL
        if config.target_env.to_lowercase().as_str() == "web" {
            // For web builds, we typically use SDL3 and OpenGL3
            if uses_sdl || (!uses_glfw && !uses_sdl) { // Default to SDL if nothing is explicitly detected
                backends.push(format!("imgui_impl_sdl{}.cpp", sdl_version));
                log::info!("Using SDL{} for web build", sdl_version);
            }
            
            if uses_glfw {
                backends.push("imgui_impl_glfw.cpp".to_string());
                log::info!("Using GLFW for web build");
            }
            
            // For web, prefer OpenGL3/WebGL2
            if uses_opengl3 || (!uses_opengl2 && !uses_opengl3) { // Default to OpenGL3 if nothing detected
                backends.push("imgui_impl_opengl3.cpp".to_string());
                log::info!("Using OpenGL3 for web build");
            } else if uses_opengl2 {
                backends.push("imgui_impl_opengl2.cpp".to_string());
                log::info!("Using OpenGL2 for web build");
            }
        } else {
            // For non-web builds, include what's detected
            if uses_sdl {
                backends.push(format!("imgui_impl_sdl{}.cpp", sdl_version));
            }
            if uses_glfw {
                backends.push("imgui_impl_glfw.cpp".to_string());
            }
            if uses_opengl2 {
                backends.push("imgui_impl_opengl2.cpp".to_string());
            }
            if uses_opengl3 {
                backends.push("imgui_impl_opengl3.cpp".to_string());
            }
        }
        
        // If no backends were determined, make reasonable defaults based on project path
        if backends.is_empty() {
            let project_name = project_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            if project_name.contains("sdl") {
                backends.push("imgui_impl_sdl3.cpp".to_string());
                backends.push("imgui_impl_opengl3.cpp".to_string());
            } else if project_name.contains("glfw") {
                backends.push("imgui_impl_glfw.cpp".to_string());
                backends.push("imgui_impl_opengl3.cpp".to_string());
            } else {
                // Default fallback
                backends.push("imgui_impl_sdl3.cpp".to_string());
                backends.push("imgui_impl_opengl3.cpp".to_string());
            }
        }
        
        log::info!("Determined needed backends: {:?}", backends);
        Ok(backends)
    }
    
    fn extract_include_paths(&self, source_file: &Path, include_paths: &mut std::collections::HashSet<std::path::PathBuf>) -> Result<(), String> {
        let content = std::fs::read_to_string(source_file)
            .map_err(|e| format!("Failed to read source file {:?}: {}", source_file, e))?;
            
        // Parse include statements
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#include") {
                self.parse_include_line(trimmed, source_file, include_paths);
            }
        }
        
        Ok(())
    }
    
    fn parse_include_line(&self, line: &str, source_file: &Path, include_paths: &mut std::collections::HashSet<std::path::PathBuf>) {
        // Extract the include path from lines like:
        // #include "imgui.h"
        // #include <SDL3/SDL.h>
        // #include "../libs/emscripten/emscripten_mainloop_stub.h"
        
        if let Some(start) = line.find('"') {
            if let Some(end) = line.rfind('"') {
                if start < end {
                    let include_file = &line[start + 1..end];
                    self.resolve_include_path(include_file, source_file, include_paths);
                }
            }
        } else if let Some(start) = line.find('<') {
            if let Some(end) = line.rfind('>') {
                if start < end {
                    let include_file = &line[start + 1..end];
                    self.resolve_include_path(include_file, source_file, include_paths);
                }
            }
        }
    }
        
    fn resolve_include_path(&self, include_file: &str, source_file: &Path, include_paths: &mut std::collections::HashSet<std::path::PathBuf>) {
        let source_dir = source_file.parent().unwrap_or(Path::new("."));
        
        // Handle relative includes (with quotes)
        if include_file.starts_with("../") || include_file.starts_with("./") || !include_file.contains('/') {
            let resolved_path = source_dir.join(include_file);
            if let Some(include_dir) = resolved_path.parent() {
                if include_dir.exists() {
                    include_paths.insert(include_dir.to_path_buf());
                    log::debug!("Added include path from relative include '{}': {:?}", include_file, include_dir);
                }
            }
            
            // For relative paths like "../libs/emscripten/file.h", we need to add the source directory
            // AND all parent directories that might be needed for the relative path resolution
            if include_file.starts_with("../") {
                include_paths.insert(source_dir.to_path_buf());
                log::debug!("Added source directory for relative include resolution: {:?}", source_dir);
                
                // Count how many "../" are in the path and add those parent directories
                let mut count = 0;
                let mut remaining = include_file;
                while remaining.starts_with("../") {
                    remaining = &remaining[3..];
                    count += 1;
                }
                
                // Add parent directories up to the count needed
                let mut current_dir = source_dir;
                for i in 0..count {
                    if let Some(parent) = current_dir.parent() {
                        include_paths.insert(parent.to_path_buf());
                        log::debug!("Added parent directory level {}: {:?}", i + 1, parent);
                        current_dir = parent;
                    }
                }
            }
        } else if include_file.contains('/') {
            let parts: Vec<&str> = include_file.split('/').collect();
            if parts.len() > 1 {
                // For includes like "imgui_impl_sdl3.h", look for backends directory
                if include_file.starts_with("imgui_impl_") {
                    let imgui_root = source_dir.join("..").join("..");
                    let backends_dir = imgui_root.join("backends");
                    if backends_dir.exists() {
                        log::debug!("Added ImGui backends directory: {:?}", backends_dir);
                        include_paths.insert(backends_dir);
                    }
                }
                
                // For other includes with paths, try to find the base directory
                let mut current_dir = source_dir.to_path_buf();
                for _ in 0..5 { // Search up to 5 levels up
                    let potential_path = current_dir.join(&parts[0]);
                    if potential_path.exists() {
                        include_paths.insert(current_dir.clone());
                        log::debug!("Added include path for '{}': {:?}", include_file, current_dir);
                        break;
                    }
                    if let Some(parent) = current_dir.parent() {
                        current_dir = parent.to_path_buf();
                    } else {
                        break;
                    }
                }
            }
        }
        
        // Always add the ImGui root directory for basic includes
        let imgui_root = source_dir.join("..").join("..");
        if imgui_root.join("imgui.h").exists() {
            include_paths.insert(imgui_root.clone());
            log::debug!("Added ImGui root directory: {:?}", imgui_root);
        }
        
        // Add the source file's directory itself
        include_paths.insert(source_dir.to_path_buf());
    }
}

impl LibraryHandler for ImGuiHandler {
    fn library_name(&self) -> &'static str {
        "ImGui"
    }
    
    fn detect(&self, project_path: &Path) -> bool {
        // Check if this is an ImGui project by looking for:
        // 1. ImGui example directory structure
        // 2. ImGui source files
        // 3. ImGui includes in source files
        
        let is_imgui_example = project_path.to_string_lossy().contains("imgui") && 
                               project_path.to_string_lossy().contains("example");
        
        if is_imgui_example {
            return true;
        }
        
        // Check for ImGui source files in the project or nearby directories
        let imgui_dir = project_path.join("..").join("..");
        let has_imgui_sources = imgui_dir.join("imgui.cpp").exists() &&
                               imgui_dir.join("imgui.h").exists();
        
        if has_imgui_sources {
            return true;
        }
        
        // Check for ImGui includes in source files
        let main_cpp = project_path.join("main.cpp");
        if main_cpp.exists() {
            if let Ok(content) = std::fs::read_to_string(&main_cpp) {
                if content.contains("#include \"imgui.h\"") || 
                   content.contains("#include <imgui.h>") ||
                   content.contains("imgui_impl_") {
                    return true;
                }
            }
        }
        
        // Check other common C++ file extensions
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "cpp" || extension == "cxx" || extension == "cc" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("#include \"imgui.h\"") || 
                               content.contains("#include <imgui.h>") ||
                               content.contains("imgui_impl_") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        false
    }
    
    fn compile(&self, project_path: &Path, config: &AppConfig) -> Result<(), String> {
        log::info!("Compiling ImGui project using ImGuiHandler");
        
        // Find all source files in the project
        let mut sources = Vec::new();
        self.find_source_files(project_path, &mut sources, config)?;
        
        // If no sources found in project directory, look for ImGui sources in typical locations
        if sources.is_empty() {
            let imgui_dir = project_path.join("..").join("..");
            let default_sources = vec![
                project_path.join("main.cpp"),
                imgui_dir.join("imgui.cpp"),
                imgui_dir.join("imgui_demo.cpp"),
                imgui_dir.join("imgui_draw.cpp"),
                imgui_dir.join("imgui_tables.cpp"),
                imgui_dir.join("imgui_widgets.cpp"),
            ];
            
            for source in default_sources {
                if source.exists() {
                    sources.push(source);
                }
            }
            
            // Add compatible backends for default case as well
            if imgui_dir.exists() {
                self.add_compatible_backends(project_path, &imgui_dir, &mut sources, config)?;
            }
        }

        if sources.is_empty() {
            return Err("No source files found for ImGui project".to_string());
        }

        // Extract include paths from all source files
        let mut include_paths: std::collections::HashSet<std::path::PathBuf> = std::collections::HashSet::new();
        for source in &sources {
            self.extract_include_paths(source, &mut include_paths)?;
        }

        // Build emcc command arguments
        let mut emcc_args = Vec::new();
        
        // Add source files
        for source in &sources {
            emcc_args.push(source.to_string_lossy().to_string());
        }

        // Add extracted include directories
        for include_path in &include_paths {
            emcc_args.push(format!("-I{}", include_path.to_string_lossy()));
        }

        // Add C++ standard
        emcc_args.push("-std=c++11".to_string());

        // Add build-specific flags
        match config.build_config.to_lowercase().as_str() {
            "debug" => {
                emcc_args.push("-g4".to_string());
                emcc_args.push("-O0".to_string());
                emcc_args.push("-sASSERTIONS=2".to_string());
                emcc_args.push("-sSAFE_HEAP=1".to_string());
            }
            "release" => {
                emcc_args.push("-O3".to_string());
                emcc_args.push("-sASSERTIONS=0".to_string());
                // Note: --llvm-lto is deprecated and ignored in newer Emscripten versions
                // LTO is enabled by default in -O3 builds
            }
            _ => {
                emcc_args.push("-O2".to_string());
                emcc_args.push("-sASSERTIONS=1".to_string());
            }
        }

        // Determine which backends are actually being used for dynamic flag configuration
        let using_sdl = sources.iter().any(|s| s.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.contains("imgui_impl_sdl"))
            .unwrap_or(false));
        let using_glfw = sources.iter().any(|s| s.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.contains("imgui_impl_glfw"))
            .unwrap_or(false));

        // Add Emscripten-specific flags based on detected backends
        if using_sdl {
            emcc_args.push("-sUSE_SDL=3".to_string()); // Use SDL3
            log::info!("Adding SDL3 Emscripten flags");
        }
        
        if using_glfw {
            emcc_args.push("-sUSE_GLFW=3".to_string()); // Use GLFW for web
            log::info!("Adding GLFW Emscripten flags");
        }
        
        // Common OpenGL/WebGL flags
        emcc_args.push("-sUSE_WEBGL2=1".to_string());
        emcc_args.push("-sFULL_ES3=1".to_string());
        emcc_args.push("-sALLOW_MEMORY_GROWTH=1".to_string());
        emcc_args.push("-sMODULARIZE=1".to_string());
        emcc_args.push("-sEXPORT_ES6=1".to_string());
        emcc_args.push(format!("-sENVIRONMENT={}", match config.target_env.to_lowercase().as_str() {
            "web" => "web",
            "node" => "node",
            _ => "web"
        }));
        emcc_args.push("-sEXPORTED_RUNTIME_METHODS=FS,callMain,setValue,getValue,UTF8ToString,stringToUTF8".to_string());
        emcc_args.push("-sEXPORT_NAME='Module'".to_string());
        emcc_args.push("-sINITIAL_MEMORY=67108864".to_string()); // 64MB
        emcc_args.push("-sGL_ENABLE_GET_PROC_ADDRESS=1".to_string());

        // Exception handling
        emcc_args.push("-fwasm-exceptions".to_string());

        // Debug-specific GL flags
        if config.build_config.to_lowercase().as_str() == "debug" {
            emcc_args.push("-sGL_ASSERTIONS=1".to_string());
        }

        // Add user-defined flags
        if let Some(user_flags) = &config.emcc_flags {
            for flag in user_flags.split_whitespace() {
                if !emcc_args.contains(&flag.to_string()) {
                    emcc_args.push(flag.to_string());
                }
            }
        }

        // Output files
        let output_js = config.output_dir.join(format!("{}.js", config.output_name));
        emcc_args.push("-o".to_string());
        emcc_args.push(output_js.to_string_lossy().to_string());
        // Note: WASM_BINARY_NAME is not a valid setting, the .wasm file will be automatically named based on the .js output

        log::debug!("Running emcc with args: {:?}", emcc_args.join(" "));
        
        // Run emcc directly using the resolved tool name
        EmscriptenRunner::run_emscripten_tool(
            &resolve_emscripten_tool("em++"),
            &emcc_args,
            project_path,
            config,
        )?;

        log::info!("Successfully compiled ImGui project. Output in {:?}", config.output_dir);
        Ok(())
    }
    
    fn priority(&self) -> u32 {
        10 // High priority for ImGui projects
    }
}
