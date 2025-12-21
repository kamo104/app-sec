#!/bin/bash

# Unified build script for the entire application
# Builds password validator, WebAssembly, protobuf types, frontend, and backend

set -e  # Exit on error

echo "Starting unified build process..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check prerequisites
print_status "Checking prerequisites..."

# Check Rust
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo not found. Please install Rust."
    exit 1
fi
print_success "Rust found: $(cargo --version)"

# Check wasm-bindgen
if ! command -v wasm-bindgen &> /dev/null; then
    print_error "wasm-bindgen not found. Please install it with: cargo install wasm-bindgen-cli"
    exit 1
fi
print_success "wasm-bindgen found"

# Check deno
if ! command -v deno &> /dev/null; then
    print_error "Deno not found. Please install Deno."
    exit 1
fi
print_success "Deno found: $(deno --version | head -1)"

# Check protoc
if ! command -v protoc &> /dev/null; then
    print_error "protoc not found. Please install protobuf compiler."
    exit 1
fi
print_success "protoc found"

# Build field validator library
print_status "Building field validator library..."
cd field-validator

# Run tests first
print_status "Running field validator tests..."
cargo test --quiet
if [ $? -eq 0 ]; then
    print_success "Field validator tests passed"
else
    print_error "Field validator tests failed"
    exit 1
fi

# Build for native (backend use)
print_status "Building field validator for native use..."
cargo build --release --quiet
if [ $? -eq 0 ]; then
    print_success "Native build completed"
else
    print_error "Native build failed"
    exit 1
fi

# Build for WebAssembly
print_status "Building field validator for WebAssembly..."
cargo build --target wasm32-unknown-unknown --release --features wasm --quiet
if [ $? -eq 0 ]; then
    print_success "WebAssembly build completed"
else
    print_error "WebAssembly build failed"
    exit 1
fi

# Generate WebAssembly bindings
print_status "Generating WebAssembly bindings..."
cd ../frontend
mkdir -p src/wasm
cd ../field-validator
wasm-bindgen target/wasm32-unknown-unknown/release/field_validator.wasm --target web --out-dir ../frontend/src/wasm --out-name field-validator
if [ $? -eq 0 ]; then
    print_success "WebAssembly bindings generated"
else
    print_error "WebAssembly binding generation failed"
    exit 1
fi

# Generate protobuf types for frontend
print_status "Generating protobuf types for frontend..."
cd ../frontend
if [ -f "node_modules/.bin/protoc-gen-ts_proto" ]; then
    mkdir -p src/generated
    protoc --plugin=./node_modules/.bin/protoc-gen-ts_proto --ts_proto_out=./src/generated --proto_path=../proto api.proto
    if [ $? -eq 0 ]; then
        print_success "Frontend protobuf types generated"
    else
        print_warning "Frontend protobuf generation failed (may not be critical)"
    fi
else
    print_warning "protoc-gen-ts_proto not found, skipping frontend protobuf generation"
fi

# Build backend
print_status "Building backend server..."
cd ../backend
cargo build --release --quiet
if [ $? -eq 0 ]; then
    print_success "Backend build completed"
else
    print_error "Backend build failed"
    exit 1
fi

# Build frontend
print_status "Building frontend..."
cd ../frontend
deno run build
if [ $? -eq 0 ]; then
    print_success "Frontend build completed"
else
    print_error "Frontend build failed"
    exit 1
fi

print_success "All builds completed successfully!"
echo ""
echo "To run the application:"
echo "  cd backend && cargo run --release"
echo ""
echo "The backend will serve the frontend from the dist directory automatically."
