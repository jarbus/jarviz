#!/bin/bash
# Clean all artifacts
echo "Cleaning build artifacts..."
cargo clean
rm -rf pkg/

# Rebuild with wasm-pack
echo "Rebuilding WebAssembly module..."
wasm-pack build --target web

# Start the server
echo "Starting server at http://localhost:8000"
python3 -m http.server
# navigate to http://localhost:8000
