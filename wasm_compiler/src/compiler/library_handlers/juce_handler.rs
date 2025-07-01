use std::path::Path;
use crate::app_config::AppConfig;
use super::LibraryHandler;

pub struct JuceHandler;

impl JuceHandler {
    pub fn new() -> Self {
        JuceHandler
    }
}

impl LibraryHandler for JuceHandler {
    fn library_name(&self) -> &'static str {
        "JUCE"
    }
    
    fn detect(&self, project_path: &Path) -> bool {
        // Check for JUCE includes in source files
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "cpp" || extension == "cxx" || extension == "cc" || extension == "h" || extension == "hpp" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("#include <juce_") || 
                               content.contains("#include \"juce_") ||
                               content.contains("JUCE_") ||
                               content.contains("juce::") ||
                               content.contains("JUCEApplication") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        // Check for JUCE project files
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "jucer" {
                        return true;
                    }
                }
            }
        }
        
        // Check for JUCE in CMakeLists.txt
        if project_path.join("CMakeLists.txt").exists() {
            if let Ok(content) = std::fs::read_to_string(project_path.join("CMakeLists.txt")) {
                if content.contains("JUCE") || content.contains("juce_") {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn compile(&self, _project_path: &Path, _config: &AppConfig) -> Result<(), String> {
        Err(format!(
            "JUCE compilation to WASM is not yet implemented. \
            JUCE is primarily designed for audio applications and desktop/mobile platforms. \
            WebAssembly support for JUCE is experimental and requires special configuration. \
            Consider using Web Audio API for web-based audio applications."
        ))
    }
    
    fn priority(&self) -> u32 {
        35 // Medium priority
    }
}
