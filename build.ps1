# Build script for TREX WASM
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

Write-Host "Build complete! Output generated in the './pkg/' directory." -ForegroundColor Green
