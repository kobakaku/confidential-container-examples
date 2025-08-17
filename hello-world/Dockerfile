# Build stage
FROM mcr.microsoft.com/devcontainers/base:dev-ubuntu-24.04 AS builder
WORKDIR /build

# Download verbose-report
RUN curl -L https://github.com/microsoft/confidential-sidecar-containers/releases/latest/download/verbose-report > verbose-report

# Rust build stage
FROM rust:1.82-slim AS rust-builder
WORKDIR /app

# Copy the source code
COPY Cargo.toml .
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary and verbose-report
COPY --from=rust-builder /app/target/release/hello-world-cc /app/
COPY --from=builder /build/verbose-report /app/verbose-report

# Make verbose-report executable
RUN chmod +x /app/verbose-report

EXPOSE 80

# Run the application
CMD ["./hello-world-cc"]