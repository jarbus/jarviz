#!/bin/bash
wasm-pack build --target web
python3 -m http.server
# navigate to http://localhost:8000
