#!/usr/bin/env bash
#
# LOC Counter Script
# Counts lines of code in the codebase while excluding generated files and assets
#

set -e

echo "Counting Lines of Code..."
echo "========================="
echo ""

# Run cloc with exclusions
cloc . \
    --exclude-dir=node_modules,target,dist,assets,generated,wasm,.git,pkg,.vscode,.claude \
    --exclude-ext=lock,sum,json \
    --quiet

echo ""
echo "Excluded directories:"
echo "  - node_modules (dependencies)"
echo "  - target (Rust build artifacts)"
echo "  - dist (build output)"
echo "  - assets (static assets)"
echo "  - generated (protobuf generated code)"
echo "  - wasm (WebAssembly generated bindings)"
echo "  - pkg (wasm-pack output)"
echo "  - .vscode (editor config)"
echo "  - .claude (Claude Code config)"
echo ""
echo "Excluded file types:"
echo "  - .lock files (lockfiles)"
echo "  - .sum files (checksums)"
echo "  - .json files (configs and package files)"
