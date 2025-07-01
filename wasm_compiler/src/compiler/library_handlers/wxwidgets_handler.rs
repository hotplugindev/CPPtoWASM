use std::path::Path;
use crate::app_config::AppConfig;
use super::LibraryHandler;

pub struct WxWidgetsHandler;

impl WxWidgetsHandler {
    pub fn new() -> Self {
        WxWidgetsHandler
    }
}

impl LibraryHandler for WxWidgetsHandler {
    fn library_name(&self) -> &'static str {
        "wxWidgets"
    }
    
    fn detect(&self, project_path: &Path) -> bool {
        // Check for wxWidgets includes in source files
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "cpp" || extension == "cxx" || extension == "cc" || extension == "h" || extension == "hpp" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("#include <wx/") || 
                               content.contains("#include \"wx/") ||
                               content.contains("wxApp") ||
                               content.contains("wxFrame") ||
                               content.contains("wxWidget") ||
                               content.contains("wx") && (content.contains("IMPLEMENT_APP") || content.contains("wxDECLARE_")) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        // Check for wxWidgets in build files
        if project_path.join("CMakeLists.txt").exists() {
            if let Ok(content) = std::fs::read_to_string(project_path.join("CMakeLists.txt")) {
                if content.contains("wxWidgets") || content.contains("find_package.*wx") {
                    return true;
                }
            }
        }
        
        let makefile = project_path.join("Makefile");
        if makefile.exists() {
            if let Ok(content) = std::fs::read_to_string(&makefile) {
                if content.contains("wx-config") || content.contains("wxwidgets") {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn compile(&self, _project_path: &Path, _config: &AppConfig) -> Result<(), String> {
        Err(format!(
            "wxWidgets compilation to WASM is not yet implemented. \
            wxWidgets relies on native windowing systems and is not designed for WebAssembly. \
            Consider using web-based UI frameworks or ImGui for WASM applications."
        ))
    }
    
    fn priority(&self) -> u32 {
        25 // Medium-high priority
    }
}
