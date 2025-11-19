# Multi-stage build for efficient Docker image

# Stage 1: Build the Rust application
FROM rust:1.75-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the web server binary (release mode for optimization)
RUN cargo build --release --bin flac-to-mp3-web

# Stage 2: Create the runtime image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ffmpeg \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder
COPY --from=builder /app/target/release/flac-to-mp3-web /app/flac-to-mp3-web

# Copy static web files
COPY static ./static

# Create directories for file processing
RUN mkdir -p /tmp/flac_uploads /tmp/mp3_output && \
    chmod 777 /tmp/flac_uploads /tmp/mp3_output

# Expose the web server port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/flac-to-mp3-web", "--version"] || exit 1

# Run the web server
CMD ["/app/flac-to-mp3-web"]
