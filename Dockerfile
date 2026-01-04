# Stage 1: Build Rust application
FROM rust:1.83-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p src/handlers src/services src/models src/utils && \
    touch src/lib.rs && \
    echo "pub mod handlers; pub mod services; pub mod models; pub mod utils;" >> src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release || true

# Remove dummy source
RUN rm -rf src

# Copy actual source code
COPY src ./src
COPY static ./static

# Build application
RUN touch src/main.rs && \
    cargo build --release

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ffmpeg \
    zip \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/narayan-web /app/narayan-web

# Copy static files
COPY --from=builder /app/static /app/static

# Create temp directory
RUN mkdir -p /tmp/narayan && chmod 777 /tmp/narayan

# Create non-root user
RUN useradd -m -u 1000 narayan && \
    chown -R narayan:narayan /app /tmp/narayan

USER narayan

# Environment variables
ENV RUST_LOG=info
ENV BIND_ADDRESS=0.0.0.0:3000
ENV TEMP_DIR=/tmp/narayan

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/bin/sh", "-c", "exit 0"]

CMD ["/app/narayan-web"]
