#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Building WASM package ==="
cd "$ROOT_DIR"
wasm-pack build jtd-wasm-validator --target web --out-dir ../wasm-pkg

echo "=== Running WASM e2e test via bun ==="
bun run examples/e2e/test-wasm.mjs

echo "=== Building Docker image (browser example) ==="
cd "$SCRIPT_DIR"
rm -rf pkg
cp -r "$ROOT_DIR/wasm-pkg" pkg
docker build -t jtd-wasm-e2e .

echo "=== Running browser example in Docker (serves on :8080) ==="
echo "To test interactively: docker run --rm -p 8080:8080 jtd-wasm-e2e"
echo "Then open http://localhost:8080 in a browser."

echo "=== Done ==="
