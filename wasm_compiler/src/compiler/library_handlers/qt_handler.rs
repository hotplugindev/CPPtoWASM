use std::path::Path;
use crate::app_config::AppConfig;
use super::LibraryHandler;

pub struct QtHandler;

impl QtHandler {
    pub fn new() -> Self {
        QtHandler
    }
}

impl LibraryHandler for QtHandler {
    fn library_name(&self) -> &'static str {
        "Qt"
    }
    
    fn detect(&self, project_path: &Path) -> bool {
        // Check for Qt includes in source files
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "cpp" || extension == "cxx" || extension == "cc" || extension == "h" || extension == "hpp" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("#include <Q") || 
                               content.contains("#include \"Q") ||
                               content.contains("QWidget") ||
                               content.contains("QApplication") ||
                               content.contains("Q_OBJECT") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        // Check for Qt project files
        if project_path.join("CMakeLists.txt").exists() {
            if let Ok(content) = std::fs::read_to_string(project_path.join("CMakeLists.txt")) {
                if content.contains("find_package(Qt") || content.contains("Qt5::") || content.contains("Qt6::") {
                    return true;
                }
            }
        }
        
        // Check for .pro files (qmake)
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "pro" {
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    fn compile(&self, _project_path: &Path, _config: &AppConfig) -> Result<(), String> {
        Err(format!(
            "Qt compilation to WASM is not yet implemented. \
            Qt for WebAssembly requires Qt 5.12+ with special configuration and is currently not supported by this compiler. \
            Please refer to Qt's official WebAssembly documentation for manual compilation."
        ))
    }
    
    fn priority(&self) -> u32 {
        15 // High priority as it's a major GUI framework
    }
}
