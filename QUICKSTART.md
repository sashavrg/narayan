# Quick Start Guide

## Running Narayan Web Converter

### Option 1: Docker (Recommended)

1. **Build and start the container:**
```bash
cd /root/narayan-web
docker-compose up -d --build
```

2. **Access the web interface:**
Open your browser and navigate to:
```
http://localhost:3000
```

3. **Monitor logs (optional):**
```bash
docker-compose logs -f
```

4. **Stop the service:**
```bash
docker-compose down
```

### Option 2: Local Development

1. **Ensure FFmpeg is installed:**
```bash
ffmpeg -version
```

2. **Build and run:**
```bash
cd /root/narayan-web
cargo run --release
```

3. **Access the web interface:**
```
http://localhost:3000
```

## How to Use

1. **Upload Files:**
   - Drag and drop FLAC files onto the upload zone, or
   - Click "Browse Files" to select files

2. **Monitor Progress:**
   - Watch real-time conversion progress
   - Each file shows individual progress

3. **Download:**
   - Click "Download All (ZIP)" to get all converted MP3 files
   - Files are converted at 320kbps with metadata preserved

## Configuration

Edit `.env` file or set environment variables:
```bash
BIND_ADDRESS=0.0.0.0:3000
MAX_CONCURRENT_CONVERSIONS=4
MAX_UPLOAD_SIZE=524288000  # 500MB
```

## Troubleshooting

**Docker build fails:**
- Ensure Docker has enough resources (2GB RAM minimum)

**Cannot access http://localhost:3000:**
- Check if the container is running: `docker ps`
- Check logs: `docker-compose logs`

**Conversion fails:**
- Ensure FFmpeg is working in the container
- Check container logs for errors

## Performance Tips

- Adjust `MAX_CONCURRENT_CONVERSIONS` based on your CPU cores
- For better performance, run on SSD storage
- Allow at least 2GB RAM for the container
