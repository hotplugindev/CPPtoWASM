use std::path::Path;
use crate::app_config::AppConfig;
use super::LibraryHandler;

pub struct UltimatePlusPlusHandler;

impl UltimatePlusPlusHandler {
    pub fn new() -> Self {
        UltimatePlusPlusHandler
    }
}

impl LibraryHandler for UltimatePlusPlusHandler {
    fn library_name(&self) -> &'static str {
        "Ultimate++"
    }
    
    fn detect(&self, project_path: &Path) -> bool {
        // Check for Ultimate++ includes in source files
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "cpp" || extension == "cxx" || extension == "cc" || extension == "h" || extension == "hpp" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("#include <CtrlLib/") || 
                               content.contains("#include \"CtrlLib/") ||
                               content.contains("#include <Core/") ||
                               content.contains("NAMESPACE_UPP") ||
                               content.contains("using namespace Upp;") ||
                               content.contains("Upp::") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        // Check for Ultimate++ project files
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "upp" {
                        return true;
                    }
                }
            }
        }
        
        // Check for Ultimate++ workspace file
        if project_path.join("*.wsc").exists() {
            return true;
        }
        
        false
    }
    
    fn compile(&self, _project_path: &Path, _config: &AppConfig) -> Result<(), String> {
        Err(format!(
            "Ultimate++ compilation to WASM is not yet implemented. \
            Ultimate++ is a C++ cross-platform rapid application development suite that relies on native windowing systems. \
            WebAssembly support would require significant framework modifications. \
            Consider using web-based UI frameworks or ImGui for WASM applications."
        ))
    }
    
    fn priority(&self) -> u32 {
        45 // Lower priority as it's less common
    }
}
