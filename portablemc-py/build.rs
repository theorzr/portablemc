fn main() {
    // Allow linking the extension module with a plain cargo build on macOS, Python 
    // symbols are resolved when the module is loaded by the interpreter.
    pyo3_build_config::add_extension_module_link_args();
}
