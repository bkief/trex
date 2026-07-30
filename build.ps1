# Build script for TREX WASM & Test Suite
Write-Host "Running TREX Rust Unit Test Suite (IEEE 738 & Sag)..." -ForegroundColor Cyan
cargo test
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Rust unit tests failed." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "Building TREX WebAssembly module..." -ForegroundColor Cyan

# Compile Rust to WASM
cargo build --target wasm32-unknown-unknown --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Cargo build failed." -ForegroundColor Red
    exit $LASTEXITCODE
}

# Generate bindings
Write-Host "Generating wasm-bindgen ES Module bindings..." -ForegroundColor Cyan
wasm-bindgen target/wasm32-unknown-unknown/release/trex.wasm --out-dir pkg --target web --no-typescript

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: wasm-bindgen generation failed." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "Build complete! All tests passed and output generated in './pkg/'." -ForegroundColor Green
