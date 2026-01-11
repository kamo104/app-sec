# Multi-stage Dockerfile for building the full application

# =============================================================================
# Stage 1: Builder
# =============================================================================
FROM rust:1.85-bookworm AS builder

RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install Deno
RUN curl -fsSL https://deno.land/install.sh | sh
ENV DENO_INSTALL="/root/.deno"
ENV PATH="${DENO_INSTALL}/bin:${PATH}"

# Add wasm32 target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy source
COPY . .

# Build using build.sh
RUN ./build.sh

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
ENV ASSETS_DIR=/app/dist
CMD ["./appsec-server", "--web-bind-addr", "0.0.0.0", "--web-port", "4000"]
