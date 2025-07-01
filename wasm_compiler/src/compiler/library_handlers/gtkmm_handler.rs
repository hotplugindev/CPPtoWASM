use std::path::Path;
use crate::app_config::AppConfig;
use super::LibraryHandler;

pub struct GtkmmHandler;

impl GtkmmHandler {
    pub fn new() -> Self {
        GtkmmHandler
    }
}

impl LibraryHandler for GtkmmHandler {
    fn library_name(&self) -> &'static str {
        "GTKmm"
    }
    
    fn detect(&self, project_path: &Path) -> bool {
        // Check for GTKmm includes in source files
        for entry in std::fs::read_dir(project_path).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "cpp" || extension == "cxx" || extension == "cc" || extension == "h" || extension == "hpp" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("#include <gtkmm") || 
                               content.contains("#include \"gtkmm") ||
                               content.contains("Gtk::") ||
                               content.contains("Glib::") ||
                               content.contains("sigc::") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        // Check for GTKmm in build files
        if project_path.join("CMakeLists.txt").exists() {
            if let Ok(content) = std::fs::read_to_string(project_path.join("CMakeLists.txt")) {
                if content.contains("gtkmm") || content.contains("GTKmm") || content.contains("pkg_check_modules.*gtkmm") {
                    return true;
                }
            }
        }
        
        let makefile = project_path.join("Makefile");
        if makefile.exists() {
            if let Ok(content) = std::fs::read_to_string(&makefile) {
                if content.contains("gtkmm") || content.contains("pkg-config.*gtkmm") {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn compile(&self, _project_path: &Path, _config: &AppConfig) -> Result<(), String> {
        Err(format!(
            "GTKmm compilation to WASM is not yet implemented. \
            GTKmm relies on native GTK+ which is not available in WebAssembly environments. \
            Consider using web-based UI frameworks or ImGui for WASM applications."
        ))
    }
    
    fn priority(&self) -> u32 {
        30 // Medium-high priority
    }
}
