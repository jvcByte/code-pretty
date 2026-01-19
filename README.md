# Code Snippet Designer

A web application that transforms code snippets into visually appealing images for sharing and presentations.

## ✨ Features

- **Upload screenshots** and extract code using OCR (automatic with Docker)
- **Paste or type code** directly with syntax highlighting
- **Customize visual themes** and styling
- **Export high-quality images** in multiple formats (PNG, JPG, SVG)
- **Responsive web interface** for desktop and mobile
- **Auto-detect programming languages**

## 🚀 Quick Start

### Easiest Way: Docker (Includes Everything)

```bash
# One command to run everything
./quick-start.sh

# Or manually
docker-compose up -d
```

Visit `http://localhost:3000` - **OCR is fully enabled!** ✅

### Without Docker

```bash
cargo run --no-default-features
```

Visit `http://localhost:3000` - Users manually enter code (no OCR)

## 🌐 Deploy to Production (Free Options)

**No Tesseract installation needed - Docker handles everything!**

### Railway (Easiest - 1 minute)
```bash
npm install -g @railway/cli
railway login
railway up
```
**Done!** Your app is live with OCR enabled.

### Other Free Options
- **Render**: Push to GitHub, connect repo, deploy
- **Fly.io**: `fly launch && fly deploy`
- **Cloud Run**: `gcloud run deploy --source .`

**See [DEPLOYMENT.md](DEPLOYMENT.md) for complete guides**

## ❓ Common Questions

**Q: Do users need to install Tesseract?**  
A: No! Docker bundles everything. Users just visit your website.

**Q: Where should I host this?**  
A: Railway (free tier) is easiest. See [DEPLOYMENT.md](DEPLOYMENT.md) for all options.

**Q: Does OCR work automatically?**  
A: Yes, when using Docker. No configuration needed.

**See [QUICK_ANSWERS.md](QUICK_ANSWERS.md) for more**

## 📦 What's Included in Docker?

- ✅ Rust application
- ✅ Tesseract OCR (automatic text extraction)
- ✅ All system dependencies
- ✅ Optimized for production
- ✅ Health checks
- ✅ Auto-restart

**No manual installation required!**

## Development

### Prerequisites

- Rust 1.70+
- (Optional) Tesseract OCR library for image text extraction

### Setup

1. Clone the repository
   ```bash
   git clone <repository-url>
   cd code-snippet-designer
   ```

2. Build the project
   ```bash
   cargo build
   ```

3. Run the development server
   ```bash
   cargo run --no-default-features
   ```

4. Open your browser to `http://localhost:3000`

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test integration_tests
cargo test --test performance_tests
```

See [tests/README.md](tests/README.md) for more information about the test suites.

### Project Structure

```
src/
├── handlers/     # HTTP request handlers
├── services/     # Business logic services
├── models/       # Data models and structures
└── utils/        # Utility functions
```

## License

MIT License