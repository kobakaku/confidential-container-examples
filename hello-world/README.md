# Hello World Rust - Azure Confidential Container

A minimal Rust web application demonstrating Azure Confidential Container deployment with attestation report display.

## Overview

This application provides:
- Simple HTTP server running on port 80
- Display of attestation report from confidential container
- Minimal dependencies (actix-web)

## Prerequisites

- Rust 1.75 or later
- Docker
- Azure CLI
- Azure subscription with Confidential Container support

## Building

### Local Development

```bash
cargo build
cargo run
```

### Docker Build

```bash
docker build -t hello-world-rust .
```

## Deployment

1. Push image to your container registry:

```bash
docker tag hello-world-rust <your-dockerhub-username>/hello-world-rust:latest
docker push <your-dockerhub-username>/hello-world-rust:latest
```

2. Generate CCE policy for the ARM template:
```bash
az confcom acipolicygen -a arm-template.json
```
This command automatically generates the CCE policy and writes it to the `arm-template.json` file.

3. Deploy using Azure Portal:
   - Navigate to Azure Container Instances in the Azure Portal
   - Create a new container instance
   - Use the generated `arm-template.json` for deployment

## Configuration

Update `arm-template.json` parameters:
- `name`: Container instance name
- `location`: Azure region (must support Confidential Containers)
- `image`: Your container image URI
- `cpuCores`: CPU allocation (default: 1)
- `memoryInGb`: Memory allocation (default: 1GB)

## Architecture

The application:
1. Starts actix-web server on port 80
2. Executes `verbose-report` to get attestation data
3. Formats and displays the report as HTML
4. Shows Azure Container Instances logo

## Notes

- The `verbose-report` binary is downloaded during Docker build
- Requires Confidential SKU in Azure Container Instances
- Port 80 must be exposed for public access