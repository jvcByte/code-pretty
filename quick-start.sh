#!/bin/bash

# Quick Start Script for Code Snippet Designer
# This script helps you get started quickly

set -e

echo "🎨 Code Snippet Designer - Quick Start"
echo "======================================"
echo ""

# Check if Docker is installed
if command -v docker &> /dev/null; then
    echo "✅ Docker is installed"
    
    echo ""
    echo "Starting application with Docker (includes OCR)..."
    echo ""
    
    # Check if docker-compose is available
    if command -v docker-compose &> /dev/null; then
        docker-compose up -d
        echo ""
        echo "✅ Application started with Docker Compose!"
    else
        docker build -t code-snippet-designer .
        docker run -d -p 3000:3000 --name code-snippet-designer code-snippet-designer
        echo ""
        echo "✅ Application started with Docker!"
    fi
    
    echo ""
    echo "🌐 Application is running at: http://localhost:3000"
    echo "❤️  Health check: http://localhost:3000/health"
    echo ""
    echo "📝 To view logs:"
    if command -v docker-compose &> /dev/null; then
        echo "   docker-compose logs -f"
    else
        echo "   docker logs -f code-snippet-designer"
    fi
    echo ""
    echo "🛑 To stop:"
    if command -v docker-compose &> /dev/null; then
        echo "   docker-compose down"
    else
        echo "   docker stop code-snippet-designer"
    fi
    
else
    echo "❌ Docker is not installed"
    echo ""
    echo "Option 1: Install Docker (Recommended)"
    echo "  Visit: https://docs.docker.com/get-docker/"
    echo ""
    echo "Option 2: Run without Docker (No OCR)"
    echo "  cargo build --release --no-default-features"
    echo "  cargo run --release --no-default-features"
    echo ""
    echo "Option 3: Deploy to cloud (Free)"
    echo "  See DEPLOYMENT.md for Railway, Render, Fly.io, etc."
    exit 1
fi
