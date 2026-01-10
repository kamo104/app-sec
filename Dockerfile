# Multi-stage Dockerfile for building the full application

# =============================================================================
# Stage 1: Builder
# =============================================================================
FROM rust:1.85-bookworm AS builder

RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install wasm-opt (binaryen)
RUN curl -L https://github.com/WebAssembly/binaryen/releases/download/version_121/binaryen-version_121-x86_64-linux.tar.gz | tar xzf - \
    && mv binaryen-version_121/bin/wasm-opt /usr/local/bin/ \
    && rm -rf binaryen-version_121

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Add wasm32 target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy source
COPY . .

# Build WASM libraries
RUN wasm-pack build field-validator --target web --out-dir ../frontend/src/wasm --out-name field-validator --release -- --features wasm
RUN wasm-pack build translator --target web --out-dir ../frontend/src/wasm --out-name translator --release -- --features wasm
RUN rm -f frontend/src/wasm/package.json frontend/src/wasm/.gitignore frontend/src/wasm/README.md

# Build backend
RUN cargo build --release -p appsec-server

# Fetch OpenAPI spec
RUN ./target/release/appsec-server --dev & \
    sleep 3 && \
    curl -s --max-time 5 http://localhost:4000/api/openapi.json > frontend/src/generated/openapi.json && \
    pkill -f appsec-server || true

# Build frontend
WORKDIR /app/frontend
RUN npm ci
RUN npm run generate:api
RUN npm run build

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1000 -s /bin/bash appuser

WORKDIR /app
COPY --from=builder /app/target/release/appsec-server ./appsec-server
COPY --from=builder /app/frontend/dist ./dist

RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 4000
ENV RUST_LOG=info
CMD ["./appsec-server", "--web-bind-addr", "0.0.0.0", "--web-port", "4000"]
