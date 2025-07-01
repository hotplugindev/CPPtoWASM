use std::path::Path;
use crate::app_config::AppConfig;
use super::LibraryHandler;

pub struct CefHandler;

impl CefHandler {
    pub fn new() -> Self {
        CefHandler
    }
}

impl LibraryHandler for CefHandler {
    fn library_name(&self) -> &'static str {
        "CEF"
    }
    
    fn detect(&self, project_path: &Path) -> bool {
        // Check for CEF includes in source files
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "cpp" || extension == "cxx" || extension == "cc" || extension == "h" || extension == "hpp" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("#include \"include/cef") || 
                               content.contains("#include <include/cef") ||
                               content.contains("CefApp") ||
                               content.contains("CefClient") ||
                               content.contains("CefBrowser") ||
                               content.contains("cef_") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        // Check for CEF in build files
        if project_path.join("CMakeLists.txt").exists() {
            if let Ok(content) = std::fs::read_to_string(project_path.join("CMakeLists.txt")) {
                if content.contains("CEF") || content.contains("chromium") {
                    return true;
                }
            }
        }
        
        // Check for CEF directory structure
        if project_path.join("include").join("cef_version.h").exists() ||
           project_path.join("..").join("include").join("cef_version.h").exists() {
            return true;
        }
        
        false
    }
    
    fn compile(&self, _project_path: &Path, _config: &AppConfig) -> Result<(), String> {
        Err(format!(
            "CEF (Chromium Embedded Framework) compilation to WASM is not supported and makes no conceptual sense. \
            CEF is designed to embed a web browser in native applications, but WASM runs inside a web browser. \
            If you need web content in a WASM application, consider using iframe elements or direct DOM manipulation."
        ))
    }
    
    fn priority(&self) -> u32 {
        50 // Lower priority as it's less common
    }
}
