use std::path::Path;
use crate::app_config::AppConfig;

/// Determines if the application is a GUI application that needs a webapp wrapper
pub fn is_gui_application(config: &AppConfig) -> bool {
    // Check if ImGui is enabled
    if config.with_imgui {
        return true;
    }
    
    // Check project path for GUI-related keywords
    let project_path_str = config.project_path.to_string_lossy().to_lowercase();
    if project_path_str.contains("imgui") || 
       project_path_str.contains("gui") || 
       project_path_str.contains("graphics") ||
       project_path_str.contains("opengl") ||
       project_path_str.contains("sdl") ||
       project_path_str.contains("glfw") {
        return true;
    }
    
    // Check if we're using WebGL/OpenGL flags
    if let Some(flags) = &config.emcc_flags {
        if flags.contains("WEBGL") || flags.contains("USE_SDL") || flags.contains("USE_GLFW") {
            return true;
        }
    }
    
    false
}

/// Creates a complete webapp in the output directory for GUI applications
pub fn create_webapp(config: &AppConfig) -> Result<(), std::io::Error> {
    if !is_gui_application(config) {
        log::debug!("Not a GUI application, skipping webapp creation");
        return Ok(());
    }
    
    log::info!("Creating webapp for GUI application: {}", config.output_name);
    
    create_html_file(&config.output_dir, &config.output_name)?;
    create_css_file(&config.output_dir)?;
    create_python_server(&config.output_dir, &config.output_name)?;
    create_readme(&config.output_dir, &config.output_name)?;
    
    log::info!("Webapp created successfully in: {:?}", config.output_dir);
    log::info!("To serve the webapp, run: python serve.py");
    
    Ok(())
}

/// Creates the main HTML file
fn create_html_file(output_dir: &Path, output_name: &str) -> Result<(), std::io::Error> {
    let html_content = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ImGUI WebAssembly Application</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            height: 100vh;
            display: flex;
            flex-direction: column;
        }}
        .header {{
            background: rgba(0,0,0,0.1);
            color: white;
            text-align: center;
            padding: 10px;
            backdrop-filter: blur(10px);
        }}
        .header h1 {{
            margin: 0;
            font-size: 1.5em;
        }}
        .canvas-container {{
            flex: 1;
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 20px;
        }}
        canvas {{
            background: #f0f0f0;
            border-radius: 8px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.3);
            max-width: 100%;
            max-height: 100%;
        }}
        .loading {{
            color: white;
            text-align: center;
            font-size: 18px;
        }}
        .controls {{
            background: rgba(0,0,0,0.1);
            padding: 10px;
            text-align: center;
            backdrop-filter: blur(10px);
        }}
        .controls button {{
            background: rgba(255,255,255,0.2);
            color: white;
            border: 1px solid rgba(255,255,255,0.3);
            padding: 8px 16px;
            margin: 0 5px;
            border-radius: 4px;
            cursor: pointer;
            transition: all 0.3s ease;
        }}
        .controls button:hover {{
            background: rgba(255,255,255,0.3);
            transform: translateY(-1px);
        }}
        .log-output {{
            background: rgba(0,0,0,0.8);
            color: #00ff00;
            font-family: 'Courier New', monospace;
            padding: 15px;
            max-height: 200px;
            overflow-y: auto;
            font-size: 12px;
            white-space: pre-wrap;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🎮 ImGUI WebAssembly Application</h1>
        <p>Compiled with wasm_compiler</p>
    </div>
    
    <div class="canvas-container">
        <div id="loading" class="loading">
            <p>⏳ Loading WebAssembly module...</p>
            <p>This may take a few moments...</p>
        </div>
        <canvas id="canvas" style="display: none;" width="1280" height="720"></canvas>
    </div>
    
    <div class="controls">
        <button onclick="toggleFullscreen()">Toggle Fullscreen</button>
        <button onclick="toggleLog()">Toggle Debug Log</button>
        <button onclick="resizeCanvas()">Resize Canvas</button>
    </div>
    
    <div id="log-output" class="log-output" style="display: none;"></div>

    <script>
        let logVisible = false;
        let logMessages = [];
        
        function log(message) {{
            const timestamp = new Date().toLocaleTimeString();
            const logMessage = `[${{timestamp}}] ${{message}}`;
            logMessages.push(logMessage);
            if (logMessages.length > 100) {{
                logMessages.shift();
            }}
            if (logVisible) {{
                const logElement = document.getElementById('log-output');
                logElement.textContent = logMessages.join('\n');
                logElement.scrollTop = logElement.scrollHeight;
            }}
        }}
        
        function toggleLog() {{
            logVisible = !logVisible;
            const logElement = document.getElementById('log-output');
            if (logVisible) {{
                logElement.style.display = 'block';
                logElement.textContent = logMessages.join('\n');
                logElement.scrollTop = logElement.scrollHeight;
            }} else {{
                logElement.style.display = 'none';
            }}
        }}
        
        function toggleFullscreen() {{
            const canvas = document.getElementById('canvas');
            if (!document.fullscreenElement) {{
                canvas.requestFullscreen().catch(err => {{
                    log('Error attempting to enable fullscreen: ' + err.message);
                }});
            }} else {{
                document.exitFullscreen();
            }}
        }}
        
        function resizeCanvas() {{
            const canvas = document.getElementById('canvas');
            const container = document.querySelector('.canvas-container');
            const containerRect = container.getBoundingClientRect();
            
            // Set canvas size to fit container while maintaining aspect ratio
            const aspectRatio = 16 / 9;
            let width = Math.min(containerRect.width - 40, 1280);
            let height = width / aspectRatio;
            
            if (height > containerRect.height - 40) {{
                height = containerRect.height - 40;
                width = height * aspectRatio;
            }}
            
            canvas.width = width;
            canvas.height = height;
            canvas.style.width = width + 'px';
            canvas.style.height = height + 'px';
            
            log(`Canvas resized to ${{width}}x${{height}}`);
            
            // Notify the module about the canvas resize
            if (typeof Module !== 'undefined' && Module._main) {{
                // Force a redraw
                try {{
                    if (Module.canvas) {{
                        Module.canvas.width = width;
                        Module.canvas.height = height;
                    }}
                }} catch (e) {{
                    log('Error resizing canvas: ' + e.message);
                }}
            }}
        }}
        
        // Override console methods to capture logs
        const originalLog = console.log;
        const originalError = console.error;
        const originalWarn = console.warn;
        
        console.log = function(...args) {{
            log('LOG: ' + args.join(' '));
            originalLog.apply(console, args);
        }};
        
        console.error = function(...args) {{
            log('ERROR: ' + args.join(' '));
            originalError.apply(console, args);
        }};
        
        console.warn = function(...args) {{
            log('WARN: ' + args.join(' '));
            originalWarn.apply(console, args);
        }};
        
        // WebAssembly Module configuration
        var Module = {{
            canvas: (function() {{
                var canvas = document.getElementById('canvas');
                canvas.addEventListener("webglcontextlost", function(e) {{
                    log('WebGL context lost. You may need to reload the page.');
                    e.preventDefault();
                }}, false);
                return canvas;
            }})(),
            print: function(text) {{
                log('STDOUT: ' + text);
            }},
            printErr: function(text) {{
                log('STDERR: ' + text);
            }},
            setStatus: function(text) {{
                if (text) {{
                    log('STATUS: ' + text);
                    const loading = document.getElementById('loading');
                    if (loading) {{
                        loading.innerHTML = '<p>⏳ ' + text + '</p>';
                    }}
                }}
            }},
            totalDependencies: 0,
            monitorRunDependencies: function(left) {{
                this.totalDependencies = Math.max(this.totalDependencies, left);
                const status = left ? 
                    `Preparing... (${{this.totalDependencies-left}}/${{this.totalDependencies}})` : 
                    'All downloads complete.';
                Module.setStatus(status);
            }},
            onRuntimeInitialized: function() {{
                log('✅ WebAssembly runtime initialized successfully');
                log('🎮 ImGUI application should now be running');
                
                // Hide loading screen and show canvas
                const loading = document.getElementById('loading');
                const canvas = document.getElementById('canvas');
                
                if (loading) loading.style.display = 'none';
                if (canvas) {{
                    canvas.style.display = 'block';
                    resizeCanvas();
                }}
                
                // Try to call main function if it exists
                try {{
                    if (typeof Module._main === 'function') {{
                        log('Calling main function...');
                        Module._main();
                    }}
                }} catch (e) {{
                    log('Note: main() may be called automatically by Emscripten');
                }}
            }},
            onAbort: function(what) {{
                log('❌ ABORT: ' + what);
                const loading = document.getElementById('loading');
                if (loading) {{
                    loading.innerHTML = '<p style="color: #ff6666;">❌ Failed to load WebAssembly module</p><p>' + what + '</p>';
                }}
            }},
            locateFile: function(path, prefix) {{
                // Handle .wasm files
                if (path.endsWith('.wasm')) {{
                    log('Loading WASM file: ' + path);
                }}
                return prefix + path;
            }}
        }};
        
        // Initialize
        log('🚀 Starting WebAssembly module load...');
        
        // Handle window resize
        window.addEventListener('resize', function() {{
            setTimeout(resizeCanvas, 100);
        }});
        
        // Set initial status
        Module.setStatus('Downloading...');
        
        window.onerror = function(msg, url, lineNo, columnNo, error) {{
            log('❌ JavaScript Error: ' + msg + ' at ' + url + ':' + lineNo + ':' + columnNo);
            return false;
        }};
    </script>
    
    <script async type="text/javascript" src="{}.js"></script>
</body>
</html>"#, output_name);

    let html_path = output_dir.join("index.html");
    std::fs::write(&html_path, html_content)?;
    
    log::debug!("Created HTML file at: {:?}", html_path);
    Ok(())
}

/// Creates the CSS stylesheet
fn create_css_file(output_dir: &Path) -> Result<(), std::io::Error> {
    let css_content = r#"/* Modern CSS Reset and Base Styles */
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', sans-serif;
    background: linear-gradient(135deg, #1e1e2e 0%, #2d2d3f 100%);
    color: #e0e0e0;
    line-height: 1.6;
    min-height: 100vh;
    overflow-x: hidden;
}

.container {
    display: grid;
    grid-template-areas: 
        "header header"
        "main sidebar";
    grid-template-rows: auto 1fr;
    grid-template-columns: 1fr 300px;
    gap: 20px;
    padding: 20px;
    min-height: 100vh;
    max-width: 1600px;
    margin: 0 auto;
}

/* Header */
.header {
    grid-area: header;
    text-align: center;
    margin-bottom: 20px;
}

.header h1 {
    color: #6366f1;
    font-size: 2.5rem;
    font-weight: 700;
    margin-bottom: 15px;
    text-shadow: 0 2px 4px rgba(99, 102, 241, 0.3);
}

/* Status indicator */
.status {
    display: inline-block;
    padding: 12px 24px;
    border-radius: 25px;
    font-family: 'Consolas', 'Monaco', monospace;
    font-weight: 600;
    font-size: 0.9rem;
    transition: all 0.3s ease;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.status.loading {
    background: linear-gradient(90deg, #3b82f6, #6366f1);
    color: white;
    animation: pulse 2s infinite;
}

.status.ready {
    background: linear-gradient(90deg, #10b981, #059669);
    color: white;
}

.status.error {
    background: linear-gradient(90deg, #ef4444, #dc2626);
    color: white;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
}

/* Main content area */
.main-content {
    grid-area: main;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
}

.canvas-container {
    position: relative;
    border: 2px solid #4b5563;
    border-radius: 12px;
    background: #000;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    transition: border-color 0.3s ease;
}

.canvas-container:hover {
    border-color: #6366f1;
}

canvas {
    display: block;
    image-rendering: pixelated;
    max-width: 100%;
    height: auto;
}

/* Controls */
.controls {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
    justify-content: center;
}

button {
    display: flex;
    align-items: center;
    gap: 8px;
    background: linear-gradient(135deg, #6366f1, #4f46e5);
    color: white;
    border: none;
    padding: 12px 20px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 600;
    transition: all 0.2s ease;
    box-shadow: 0 2px 8px rgba(99, 102, 241, 0.3);
    min-width: 120px;
}

button:hover:not(:disabled) {
    background: linear-gradient(135deg, #5856eb, #4338ca);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
}

button:active:not(:disabled) {
    transform: translateY(0);
}

button:disabled {
    background: linear-gradient(135deg, #6b7280, #4b5563);
    cursor: not-allowed;
    opacity: 0.6;
    transform: none;
    box-shadow: none;
}

button .icon {
    font-size: 16px;
}

/* Sidebar */
.info-panel {
    grid-area: sidebar;
    background: rgba(45, 45, 63, 0.6);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(107, 114, 128, 0.2);
    border-radius: 12px;
    padding: 20px;
    height: fit-content;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
}

details {
    margin-bottom: 20px;
}

details:last-child {
    margin-bottom: 0;
}

summary {
    font-weight: 600;
    color: #6366f1;
    cursor: pointer;
    padding: 10px 0;
    border-bottom: 1px solid rgba(107, 114, 128, 0.2);
    margin-bottom: 15px;
    list-style: none;
    position: relative;
    transition: color 0.2s ease;
}

summary:hover {
    color: #5856eb;
}

summary::after {
    content: '+';
    position: absolute;
    right: 0;
    font-size: 18px;
    transition: transform 0.2s ease;
}

details[open] summary::after {
    content: '−';
    transform: rotate(0deg);
}

details ul {
    list-style: none;
    padding: 0;
}

details li {
    padding: 6px 0;
    color: #d1d5db;
    border-bottom: 1px solid rgba(107, 114, 128, 0.1);
}

details li:last-child {
    border-bottom: none;
}

details li strong {
    color: #f3f4f6;
}

/* Performance stats */
.performance-stats {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.stat {
    display: flex;
    justify-content: space-between;
    padding: 8px 12px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 6px;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 0.9rem;
}

.stat .label {
    color: #9ca3af;
}

.stat span:last-child {
    color: #10b981;
    font-weight: 600;
}

/* Responsive design */
@media (max-width: 1024px) {
    .container {
        grid-template-areas: 
            "header"
            "main"
            "sidebar";
        grid-template-columns: 1fr;
        gap: 15px;
        padding: 15px;
    }
    
    .header h1 {
        font-size: 2rem;
    }
    
    .info-panel {
        order: 3;
    }
}

@media (max-width: 640px) {
    .container {
        padding: 10px;
    }
    
    .header h1 {
        font-size: 1.5rem;
    }
    
    .controls {
        flex-direction: column;
        width: 100%;
    }
    
    button {
        width: 100%;
        justify-content: center;
    }
    
    .canvas-container {
        width: 100%;
        max-width: 100%;
    }
}

/* Fullscreen styles */
.canvas-container:fullscreen {
    border: none;
    border-radius: 0;
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
}

.canvas-container:fullscreen canvas {
    max-width: 100vw;
    max-height: 100vh;
    object-fit: contain;
}

/* Loading animation */
.loading-animation {
    display: inline-block;
    width: 20px;
    height: 20px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-radius: 50%;
    border-top-color: #6366f1;
    animation: spin 1s ease-in-out infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

/* Accessibility improvements */
@media (prefers-reduced-motion: reduce) {
    *, *::before, *::after {
        animation-duration: 0.01ms !important;
        animation-iteration-count: 1 !important;
        transition-duration: 0.01ms !important;
    }
}

button:focus-visible {
    outline: 2px solid #6366f1;
    outline-offset: 2px;
}

canvas:focus {
    outline: 2px solid #6366f1;
    outline-offset: 2px;
}
"#;

    let css_path = output_dir.join("style.css");
    std::fs::write(&css_path, css_content)?;
    log::debug!("Created CSS file at: {:?}", css_path);
    Ok(())
}

/// Creates the JavaScript module file (DEPRECATED - now embedded in HTML)
fn _create_js_module_deprecated(_output_dir: &Path, _output_name: &str) -> Result<(), std::io::Error> {
    // This function is deprecated since we now embed JS directly in HTML
    // Similar to how the working Python tool does it
    Ok(())
}

/// Creates a Python server script for serving the webapp
fn create_python_server(output_dir: &Path, output_name: &str) -> Result<(), std::io::Error> {
    let python_content = format!(r#"#!/usr/bin/env python3
"""
Simple HTTP server for serving WebAssembly applications
Generated for: {}

Usage:
    python serve.py [port]

Default port: 8080
"""

import http.server
import socketserver
import os
import sys
import webbrowser
import threading
import time
from urllib.parse import urlparse

class WAsmHandler(http.server.SimpleHTTPRequestHandler):
    """Custom handler for WebAssembly applications with proper MIME types and headers"""
    
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
    
    def guess_type(self, path):
        """Override to add proper MIME types for WebAssembly and modern web files"""
        # Add WebAssembly and JavaScript MIME types
        if path.endswith('.wasm'):
            return 'application/wasm'
        elif path.endswith('.js') or path.endswith('.mjs'):
            return 'application/javascript'
        elif path.endswith('.json'):
            return 'application/json'
        
        # Use the default implementation for other files
        # The base class returns just the mimetype string
        return super().guess_type(path)
    
    def end_headers(self):
        """Add necessary headers for WebAssembly and CORS"""
        # CORS headers for development
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        
        # Headers required for WebAssembly and SharedArrayBuffer
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        
        # Disable caching for development
        self.send_header('Cache-Control', 'no-cache, no-store, must-revalidate')
        self.send_header('Pragma', 'no-cache')
        self.send_header('Expires', '0')
        
        super().end_headers()
    
    def do_OPTIONS(self):
        """Handle OPTIONS requests for CORS preflight"""
        self.send_response(200)
        self.end_headers()
    
    def log_message(self, format, *args):
        """Override to provide better logging"""
        message = format % args
        sys.stdout.write(f"[{{time.strftime('%H:%M:%S')}}] {{message}}\n")
        sys.stdout.flush()

def open_browser(url, delay=1.5):
    """Open the browser after a delay to ensure server is ready"""
    time.sleep(delay)
    print(f"🌐 Opening browser at: {{url}}")
    try:
        webbrowser.open(url)
    except Exception as e:
        print(f"⚠️  Could not open browser automatically: {{e}}")
        print(f"   Please open your browser and navigate to: {{url}}")

def main():
    # Get port from command line argument or use default
    port = 8080
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            print("Invalid port number. Using default port 8080.")
    
    # Change to the directory containing this script
    script_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(script_dir)
    
    # Check if required files exist
    required_files = ['{}.js', '{}.wasm', 'index.html']
    missing_files = [f for f in required_files if not os.path.exists(f)]
    
    if missing_files:
        print("❌ Error: Missing required files:")
        for file in missing_files:
            print(f"   - {{file}}")
        print("\nPlease make sure the WebAssembly compilation completed successfully.")
        sys.exit(1)
    
    # Set up the server
    try:
        with socketserver.TCPServer(("", port), WAsmHandler) as httpd:
            url = f"http://localhost:{{port}}"
            
            print("🚀 WebAssembly Application Server Started!")
            print("=" * 50)
            print(f"📂 Serving directory: {{os.getcwd()}}")
            print(f"🌐 Server URL: {{url}}")
            print(f"📱 Application: {}")
            print(f"⏹️  Press Ctrl+C to stop the server")
            print("=" * 50)
            
            # Open browser in a separate thread
            browser_thread = threading.Thread(target=open_browser, args=(url,))
            browser_thread.daemon = True
            browser_thread.start()
            
            # Start serving
            httpd.serve_forever()
            
    except OSError as e:
        if e.errno == 98 or e.errno == 48:  # Address already in use
            print(f"❌ Error: Port {{port}} is already in use.")
            print(f"   Try using a different port: python serve.py {{port + 1}}")
        else:
            print(f"❌ Error starting server: {{e}}")
        sys.exit(1)
    except KeyboardInterrupt:
        print("\n🛑 Server stopped by user")
        sys.exit(0)

if __name__ == "__main__":
    main()
"#, output_name, output_name, output_name, output_name);

    let python_path = output_dir.join("serve.py");
    std::fs::write(&python_path, python_content)?;
    
    // Make the Python script executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&python_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&python_path, perms)?;
    }
    
    log::debug!("Created Python server at: {:?}", python_path);
    Ok(())
}

/// Creates a README file with instructions
fn create_readme(output_dir: &Path, output_name: &str) -> Result<(), std::io::Error> {
    let readme_content = format!(r#"# {} - WebAssembly Application

This directory contains a complete WebAssembly application compiled from C++ source code.

## Files

- `{}.js` - Emscripten-generated JavaScript loader
- `{}.wasm` - Compiled WebAssembly binary
- `index.html` - Main HTML page for the web application
- `style.css` - Stylesheet for the web interface
- `app.js` - JavaScript module for application logic
- `serve.py` - Python HTTP server for local development
- `README.md` - This file

## Running the Application

### Option 1: Using the included Python server (Recommended)

```bash
python serve.py
```

Or specify a custom port:

```bash
python serve.py 8080
```

The server will automatically:
- Set proper MIME types for WebAssembly files
- Add necessary CORS headers
- Open your default browser
- Provide helpful logging

### Option 2: Using Python's built-in server

```bash
python -m http.server 8080
```

Note: This may not set proper headers for WebAssembly files.

### Option 3: Using Node.js http-server

```bash
npm install -g http-server
http-server -p 8080 --cors
```

### Option 4: Using any other web server

Make sure your web server:
1. Serves `.wasm` files with MIME type `application/wasm`
2. Serves `.js` files with MIME type `application/javascript`
3. Includes CORS headers: `Cross-Origin-Embedder-Policy: require-corp`

## Browser Requirements

- Modern browser with WebAssembly support (Chrome 57+, Firefox 52+, Safari 11+, Edge 16+)
- JavaScript enabled
- For some features: WebGL support

## Troubleshooting

### "Failed to fetch dynamically imported module"
- Ensure you're serving the files through a web server (not opening index.html directly)
- Check that all files ({}.js, {}.wasm) are in the same directory
- Verify your web server supports proper MIME types

### "WebAssembly not supported"
- Update your browser to a recent version
- Enable JavaScript if disabled

### Performance Issues
- Use a release build for better performance
- Check browser console for WebGL/graphics-related errors
- Monitor the performance stats in the web interface

## Development

To rebuild this application:
1. Modify your C++ source code
2. Run the wasm_compiler again
3. Refresh your browser (the Python server disables caching)

## Browser Developer Tools

Open browser developer tools (F12) to:
- View console output from your application
- Monitor network requests for asset loading
- Debug WebAssembly code (in supported browsers)
- Check performance metrics

---

Generated by wasm_compiler
"#, output_name, output_name, output_name, output_name, output_name);

    let readme_path = output_dir.join("README.md");
    std::fs::write(&readme_path, readme_content)?;
    log::debug!("Created README at: {:?}", readme_path);
    Ok(())
}
