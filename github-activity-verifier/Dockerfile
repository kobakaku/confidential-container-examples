# Build stage
FROM mcr.microsoft.com/devcontainers/base:dev-ubuntu-24.04 AS builder
WORKDIR /build

# Download verbose-report
RUN curl -L https://github.com/microsoft/confidential-sidecar-containers/releases/latest/download/verbose-report > verbose-report

# Rust build stage
FROM rust:1.82-slim AS rust-builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the source code
COPY Cargo.toml .
COPY src ./src
COPY static ./static

# Build the application
RUN cargo build --release

# Runtime stage
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    openssl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary, verbose-report, and static files
COPY --from=rust-builder /app/target/release/github-activity-verifier /app/
COPY --from=rust-builder /app/static /app/static
COPY --from=builder /build/verbose-report /app/verbose-report

# Make verbose-report executable
RUN chmod +x /app/verbose-report

# Default environment variables
ENV SKR_PORT=8080
ENV MAA_ENDPOINT=""
ENV PORT=9000

EXPOSE 9000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9000/ || exit 1

# Run the application
CMD ["./github-activity-verifier"]