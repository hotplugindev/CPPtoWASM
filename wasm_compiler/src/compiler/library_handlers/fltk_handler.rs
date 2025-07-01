use std::path::Path;
use crate::app_config::AppConfig;
use super::LibraryHandler;

pub struct FltkHandler;

impl FltkHandler {
    pub fn new() -> Self {
        FltkHandler
    }
}

impl LibraryHandler for FltkHandler {
    fn library_name(&self) -> &'static str {
        "FLTK"
    }
    
    fn detect(&self, project_path: &Path) -> bool {
        // Check for FLTK includes in source files
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "cpp" || extension == "cxx" || extension == "cc" || extension == "h" || extension == "hpp" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("#include <FL/") || 
                               content.contains("#include \"FL/") ||
                               content.contains("Fl_") ||
                               content.contains("Fl::") ||
                               content.contains("FLTK") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        // Check for FLTK in build files
        if project_path.join("CMakeLists.txt").exists() {
            if let Ok(content) = std::fs::read_to_string(project_path.join("CMakeLists.txt")) {
                if content.contains("FLTK") || content.contains("fltk") {
                    return true;
                }
            }
        }
        
        let makefile = project_path.join("Makefile");
        if makefile.exists() {
            if let Ok(content) = std::fs::read_to_string(&makefile) {
                if content.contains("fltk") || content.contains("fltk-config") {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn compile(&self, _project_path: &Path, _config: &AppConfig) -> Result<(), String> {
        Err(format!(
            "FLTK compilation to WASM is not yet implemented. \
            FLTK relies on native windowing systems and OpenGL contexts that are not directly available in WebAssembly. \
            Consider using web-based UI frameworks or ImGui for WASM applications."
        ))
    }
    
    fn priority(&self) -> u32 {
        40 // Medium priority
    }
}
