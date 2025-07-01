# C++ to WASM Compiler (`wasm_compiler`)

`wasm_compiler` is a command-line tool written in Rust to simplify the compilation of C++ projects to WebAssembly (WASM) using the Emscripten toolchain. It supports projects built with CMake, Makefiles, or can compile single C++ files directly.

## Features

- **Multiple Build Systems**:
    - Compiles C++ projects using `CMakeLists.txt`.
    - Compiles C++ projects using `Makefile`.
    - Compiles standalone C++ source files.
- **Emscripten Integration**: Wraps `emcc`, `emcmake`, and `emmake` for WASM compilation.
- **Configurable Builds**:
    - Debug and Release build types.
    - Customizable output directory and final binary name.
    - Pass arbitrary flags directly to Emscripten via `--emcc-flags`.
- **ImGui Support**: Includes a `--with-imgui` flag to automatically add common Emscripten flags required for ImGui-based applications (WebGL, GLFW/SDL emulation).
- **Modern JavaScript Output**: Generates ES6 modules for easy integration with modern web projects.
- **Cross-Platform**: Being Rust-based, it can be compiled on Windows, macOS, and Linux (provided Rust and Emscripten SDK are set up).

## Prerequisites

1.  **Rust**: Install Rust from [rustup.rs](https://rustup.rs/).
2.  **Emscripten SDK**: Install and configure the Emscripten SDK. Ensure that `emcc`, `emcmake`, etc., are in your system's PATH. Follow the instructions at [emscripten.org](https://emscripten.org/docs/getting_started/downloads.html).

## Building `wasm_compiler`

```bash
git clone <repository_url> # Or your project source
cd wasm_compiler
cargo build # For debug build
cargo build --release # For release build
```
The executable will be in `target/debug/wasm_compiler` or `target/release/wasm_compiler`.

## Usage

```bash
# Using the compiled binary (assuming it's in PATH or referenced directly)
./target/release/wasm_compiler --project-path /path/to/your/cpp_project [options]

# Or using cargo run (from the wasm_compiler project directory)
cargo run -- --project-path /path/to/your/cpp_project [options]
```

### Options

-   `-p, --project-path <PATH>`: Path to the C++ project directory.
-   `-o, --output-dir <PATH>`: Output directory for the WASM build (default: `dist`).
-   `-c, --build-config <STRING>`: Build configuration (e.g., `Debug`, `Release`) (default: `Release`).
-   `-t, --target-env <STRING>`: Target WASM environment (e.g., `web`, `node`, `wasi`) (default: `web`).
-   `    --output-name <STRING>`: Name of the final .wasm / .js file (without extension) (default: `output`).
-   `    --with-imgui`: Enable support for ImGui (adds necessary WebGL/GLFW flags).
-   `    --emcc-flags <STRING>`: Additional space-separated flags to pass to Emscripten/emcc. (e.g., `--emcc-flags="-sFOO=1 -sBAR=0"`)
-   `    --emscripten-config <PATH>`: Optional: Path to a specific Emscripten config file (feature not fully implemented yet).
-   `-h, --help`: Print help information.
-   `-V, --version`: Print version information.

### Examples

1.  **Compile a CMake project:**
    ```bash
    wasm_compiler --project-path ./my_cmake_app --output-dir ./wasm_out
    ```

2.  **Compile a Makefile project in Debug mode:**
    ```bash
    wasm_compiler --project-path ./my_make_app --build-config Debug
    ```

3.  **Compile an ImGui CMake project:**
    ```bash
    wasm_compiler --project-path ./my_imgui_cmake_app --with-imgui
    ```

4.  **Compile a single C++ file with custom emcc flags:**
    ```bash
    wasm_compiler --project-path ./src/hello_world.cpp --emcc-flags="-O1 -sASSERTIONS=1" --output-name hello
    ```
    *(Note: For single files, pass the file itself as `project-path`. The parent directory will be used as context for includes if needed by the C++ code, but generally single files should be self-contained or have includes managed by emcc's default search paths or additional `-I` flags passed via `--emcc-flags`)*


## Project Structure (Simplified)

-   `src/main.rs`: Entry point, CLI argument parsing.
-   `src/lib.rs`: Main library logic, orchestrates compilation.
-   `src/app_config.rs`: Defines `AppConfig` struct for CLI arguments.
-   `src/compiler/`: Module for build system handlers and Emscripten interaction.
    -   `cmake_handler.rs`: Logic for CMake projects.
    -   `make_handler.rs`: Logic for Makefile projects.
    -   `emscripten_runner.rs`: Core Emscripten command execution and flag generation.
-   `src/utils/`: Utility modules.
    -   `command_runner.rs`: For running external commands.
    -   `file_system.rs`: For file system operations.

## Contributing

(Placeholder for contribution guidelines if this were a public project)

## License

(Placeholder for license information - e.g., MIT, Apache 2.0)
```
