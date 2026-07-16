use wasm_bindgen::prelude::*;

/// Adds two numbers together in WebAssembly.
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Returns a welcoming greeting from the Rust WASM module.
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! This message is powered by Rust WebAssembly.", name)
}
