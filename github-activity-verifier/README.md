# GitHub Activity Verifier

A confidential computing application that verifies GitHub user activity and provides cryptographic proof of the verification using Azure Attestation in a Trusted Execution Environment (TEE).

## Features

- **GitHub Activity Verification**: Verify various GitHub metrics:
  - Annual commit count (365+ commits/year)
  - Consecutive activity days (100+ day streaks)
  - Total repository stars (1000+ stars)
  - Public repository count (10+ repos)
- **TEE Attestation**: Cryptographic proof that verification was performed in a secure environment
- **Web Interface**: User-friendly single-page application
- **Certificate Sharing**: Share verification certificates via URLs

## Architecture

This application runs in an Azure Confidential Container with:
- **Main Container**: Rust/Actix-web application handling GitHub API calls and verification logic
- **SKR Sidecar**: Microsoft's Secure Key Release service for MAA integration
- **Web UI**: JavaScript SPA for user interaction

## Environment Variables

### Required
- `MAA_ENDPOINT`: Your Azure Attestation endpoint URL

### Optional
- `PORT`: Server port (default: 9000)
- `SKR_PORT`: SKR sidecar port (default: 8080)
- `GITHUB_TOKEN`: GitHub Personal Access Token for higher rate limits
- `LOG_LEVEL`: Logging level (default: info)

## Quick Start

### Local Development
```bash
# Set environment variables
export MAA_ENDPOINT="https://your-maa-instance.attest.azure.net"
export GITHUB_TOKEN="your-github-token"  # Optional but recommended

# Run the application
cargo run

# Open browser
open http://localhost:9000
```

### Docker Build
```bash
docker build -t github-activity-verifier .
docker run -p 9000:9000 \
  -e MAA_ENDPOINT="https://your-maa-instance.attest.azure.net" \
  github-activity-verifier
```

## API Endpoints

### POST /api/verify
Verify GitHub user activity.

**Request:**
```json
{
  "github_username": "octocat",
  "verification_type": "yearly_commits",
  "threshold": 365
}
```

**Response:**
```json
{
  "username": "octocat",
  "verification_type": "yearly_commits",
  "meets_criteria": true,
  "attestation_token": "eyJ...",
  "verified_at": "2025-01-18T10:30:00Z",
  "proof_hash": "abc123...xyz"
}
```

### GET /proof/{proof_hash}
Retrieve verification certificate by hash.

### GET /
Serve the web application.

## Verification Types

| Type | Default Threshold | Description |
|------|------------------|-------------|
| `yearly_commits` | 365 | Commits made in the last 365 days |
| `consecutive_days` | 100 | Longest streak of consecutive activity days |
| `total_stars` | 1000 | Total stars across all public repositories |
| `public_repos` | 10 | Number of public repositories |

## Security Features

- **Input Validation**: GitHub usernames are validated against GitHub's naming rules
- **Rate Limiting**: Respects GitHub API rate limits with exponential backoff
- **Data Privacy**: GitHub data is processed in memory only, never persisted
- **TEE Attestation**: Cryptographic proof of execution environment integrity
- **Certificate Expiry**: Verification certificates expire after 24 hours

## Error Handling

The application handles various error scenarios:
- **GitHub API Errors**: User not found, rate limiting, network issues
- **MAA Errors**: Attestation failures, sidecar unavailable
- **Validation Errors**: Invalid usernames, threshold out of range

## Development

### Project Structure
```
src/
├── main.rs              # Application entry point
├── api/                 # HTTP API handlers
│   ├── handlers.rs      # Request handlers
│   └── types.rs         # API data types
├── github/              # GitHub API integration
│   ├── client.rs        # GitHub API client
│   └── types.rs         # GitHub data types
├── verification/        # Verification logic
│   └── engine.rs        # Activity verification algorithms
├── attestation/         # MAA integration
│   └── client.rs        # MAA/SKR client
└── utils/               # Utilities
    ├── errors.rs        # Error handling
    ├── storage.rs       # In-memory proof storage
    └── validation.rs    # Input validation

static/
├── index.html           # Web application
├── style.css            # Styles
└── app.js              # JavaScript logic
```

### Testing
```bash
# Run unit tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Test specific GitHub user
curl -X POST http://localhost:9000/api/verify \
  -H "Content-Type: application/json" \
  -d '{"github_username": "octocat", "verification_type": "yearly_commits"}'
```

## Deployment

This application is designed to run in Azure Confidential Containers. See the deployment guide for full setup instructions including:
- Azure Container Registry setup
- CCE policy generation
- Container Instance deployment with MAA integration

## License

This project is part of the Azure Confidential Computing examples and follows the same license terms.