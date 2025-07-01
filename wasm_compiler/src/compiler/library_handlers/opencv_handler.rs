use std::path::Path;
use crate::app_config::AppConfig;
use super::LibraryHandler;

pub struct OpenCVHandler;

impl OpenCVHandler {
    pub fn new() -> Self {
        OpenCVHandler
    }
}

impl LibraryHandler for OpenCVHandler {
    fn library_name(&self) -> &'static str {
        "OpenCV"
    }
    
    fn detect(&self, project_path: &Path) -> bool {
        // Check for OpenCV includes in source files
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "cpp" || extension == "cxx" || extension == "cc" || extension == "c" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("#include <opencv2/") || 
                               content.contains("#include \"opencv2/") ||
                               content.contains("cv::") ||
                               content.contains("CV_") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        // Check for OpenCV in CMakeLists.txt or Makefile
        let cmake_file = project_path.join("CMakeLists.txt");
        if cmake_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&cmake_file) {
                if content.contains("OpenCV") || content.contains("opencv") {
                    return true;
                }
            }
        }
        
        let makefile = project_path.join("Makefile");
        if makefile.exists() {
            if let Ok(content) = std::fs::read_to_string(&makefile) {
                if content.contains("opencv") || content.contains("OpenCV") {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn compile(&self, _project_path: &Path, _config: &AppConfig) -> Result<(), String> {
        Err(format!(
            "OpenCV compilation to WASM is not yet implemented. \
            OpenCV support for WebAssembly requires special configuration and is currently not supported by this compiler. \
            Consider using OpenCV.js for web-based computer vision applications."
        ))
    }
    
    fn priority(&self) -> u32 {
        20 // High priority as it's a commonly used library
    }
}
