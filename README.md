# Narayan Web - FLAC to MP3 Converter

A modern web-based FLAC to MP3 converter with real-time progress updates, built with Rust and Axum.

## Features

- 🎵 Convert FLAC files to MP3 (320kbps) with metadata preservation
- 🚀 Batch processing with concurrent conversion support
- 📊 Real-time progress updates via WebSocket
- 🖱️ Drag & drop file upload
- 📦 Batch download as ZIP
- 🐳 Docker containerized for easy deployment
- 🔒 Secure and efficient

## Quick Start with Docker

### Prerequisites

- Docker and Docker Compose installed
- At least 2GB of available RAM

### Running the Application

1. Clone the repository:
```bash
git clone https://github.com/sashavrg/narayan.git narayan-web
cd narayan-web
```

2. Start the application:
```bash
docker-compose up -d
```

3. Access the web interface:
```
http://localhost:3000
```

### Stopping the Application

```bash
docker-compose down
```

## Local Development

### Prerequisites

- Rust 1.75+ installed
- FFmpeg installed and in PATH
- Node.js (optional, for frontend development)

### Setup

1. Install dependencies:
```bash
cargo build
```

2. Run the application:
```bash
cargo run
```

3. Access the web interface:
```
http://localhost:3000
```

## Configuration

Configuration is done via environment variables. See `.env.example` for all available options.

Key settings:
- `BIND_ADDRESS`: Server bind address (default: `0.0.0.0:3000`)
- `MAX_CONCURRENT_CONVERSIONS`: Maximum concurrent FFmpeg processes (default: `4`)
- `MAX_UPLOAD_SIZE`: Maximum upload size in bytes (default: `524288000` = 500MB)
- `JOB_TIMEOUT`: Job timeout in seconds (default: `3600` = 1 hour)

## API Endpoints

### Upload Files
```
POST /api/upload
Content-Type: multipart/form-data

Response: { "job_id": "uuid", "file_count": 3 }
```

### Get Job Status
```
GET /api/jobs/:id

Response: {
  "id": "uuid",
  "status": { "type": "processing", "current_file": 1, "total": 3 },
  "files": [...],
  "created_at": "...",
  "updated_at": "..."
}
```

### List All Jobs
```
GET /api/jobs

Response: { "jobs": [...] }
```

### Download Single File
```
GET /api/download/:job_id
```

### Download Batch as ZIP
```
GET /api/download/batch/:job_id
```

### WebSocket
```
WS /ws

Messages:
- Subscribe: { "type": "subscribe", "job_id": "uuid" }
- Progress: { "type": "progress", "job_id": "uuid", "file": "...", "percent": 50.0 }
- Complete: { "type": "complete", "job_id": "uuid" }
- Error: { "type": "error", "job_id": "uuid", "message": "..." }
```

## Architecture

### Backend (Rust)
- **Axum**: Web framework
- **Tokio**: Async runtime
- **FFmpeg**: Audio conversion engine
- **WebSocket**: Real-time progress updates

### Frontend (Vanilla JS)
- Drag & drop file upload
- WebSocket client for live updates
- Responsive UI with CSS Grid

### Deployment
- Multi-stage Docker build
- Non-root user for security
- Resource limits and health checks

## Project Structure

```
narayan-web/
├── src/
│   ├── main.rs                 # Entry point
│   ├── config.rs               # Configuration
│   ├── error.rs                # Error types
│   ├── handlers/               # HTTP handlers
│   ├── services/               # Business logic
│   ├── models/                 # Data models
│   └── utils/                  # Utilities
├── static/
│   ├── index.html
│   ├── css/styles.css
│   └── js/                     # Frontend JS
├── Dockerfile
├── docker-compose.yml
└── README.md
```

## Performance

- Concurrent conversions: Up to 4 simultaneous FFmpeg processes
- Upload limit: 500MB per request
- Files per job: Up to 50 files
- Memory usage: ~512MB-2GB depending on load

## Troubleshooting

### FFmpeg Not Found
Ensure FFmpeg is installed:
```bash
# Debian/Ubuntu
apt-get install ffmpeg

# macOS
brew install ffmpeg

# Check installation
ffmpeg -version
```

### Port Already in Use
Change the `BIND_ADDRESS` in `.env` or docker-compose.yml

### Conversion Fails
Check FFmpeg logs in the console output with `RUST_LOG=debug`

## License

See original Narayan project for license information.

## Credits

Based on the original Narayan FLAC to MP3 converter by sashavrg.
