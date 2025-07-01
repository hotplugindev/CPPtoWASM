use std::path::Path;
use crate::app_config::AppConfig;

/// Trait for handling specific UI libraries in C++ projects
pub trait LibraryHandler {
    /// Returns the name of the library this handler manages
    fn library_name(&self) -> &'static str;
    
    /// Detects if this library is used in the project
    fn detect(&self, project_path: &Path) -> bool;
    
    /// Compiles the project using this library's specific requirements
    fn compile(&self, project_path: &Path, config: &AppConfig) -> Result<(), String>;
    
    /// Returns the priority of this handler (lower numbers have higher priority)
    /// Used when multiple libraries are detected
    fn priority(&self) -> u32 {
        100 // Default priority
    }
}

pub mod imgui_handler;
pub mod opencv_handler;
pub mod qt_handler;
pub mod gtkmm_handler;
pub mod juce_handler;
pub mod wxwidgets_handler;
pub mod fltk_handler;
pub mod cef_handler;
pub mod ultimate_handler;

use imgui_handler::ImGuiHandler;
use opencv_handler::OpenCVHandler;
use qt_handler::QtHandler;
use gtkmm_handler::GtkmmHandler;
use juce_handler::JuceHandler;
use wxwidgets_handler::WxWidgetsHandler;
use fltk_handler::FltkHandler;
use cef_handler::CefHandler;
use ultimate_handler::UltimatePlusPlusHandler;

/// Get all available library handlers
pub fn get_all_handlers() -> Vec<Box<dyn LibraryHandler>> {
    vec![
        Box::new(ImGuiHandler::new()),
        Box::new(OpenCVHandler::new()),
        Box::new(QtHandler::new()),
        Box::new(GtkmmHandler::new()),
        Box::new(JuceHandler::new()),
        Box::new(WxWidgetsHandler::new()),
        Box::new(FltkHandler::new()),
        Box::new(CefHandler::new()),
        Box::new(UltimatePlusPlusHandler::new()),
    ]
}

/// Detect which library handler should be used for the project
pub fn detect_library_handler(project_path: &Path) -> Option<Box<dyn LibraryHandler>> {
    let handlers = get_all_handlers();
    
    // Find all handlers that detect the project
    let mut detected_handlers: Vec<Box<dyn LibraryHandler>> = handlers
        .into_iter()
        .filter(|handler| handler.detect(project_path))
        .collect();
    
    if detected_handlers.is_empty() {
        return None;
    }
    
    // Sort by priority and return the highest priority handler
    detected_handlers.sort_by_key(|handler| handler.priority());
    detected_handlers.into_iter().next()
}
