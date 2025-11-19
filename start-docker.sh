#!/bin/bash

# FLAC to MP3 Converter - Docker Startup Script
# This script makes it easy to start the web-based converter

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║   FLAC to MP3 Converter - Web Server                  ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    echo "Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    echo "Visit: https://docs.docker.com/compose/install/"
    exit 1
fi

# Create output directory if it doesn't exist
mkdir -p output

echo "🔨 Building Docker image..."
docker-compose build

echo ""
echo "🚀 Starting the converter service..."
docker-compose up -d

echo ""
echo "✅ Service started successfully!"
echo ""
echo "🌐 Access the web interface at:"
echo "   • Local:   http://localhost:8080"
echo "   • Network: http://$(hostname -I | awk '{print $1}'):8080"
echo ""
echo "📝 Useful commands:"
echo "   • View logs:     docker-compose logs -f"
echo "   • Stop service:  docker-compose down"
echo "   • Restart:       docker-compose restart"
echo ""
echo "📁 Converted files will be saved to: ./output/"
echo ""
