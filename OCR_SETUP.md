# OCR Setup Guide

## Overview

The Code Snippet Designer supports **Optical Character Recognition (OCR)** to extract code from uploaded screenshots. This feature is **optional** and requires additional system dependencies.

## Current Status

When you upload an image **without OCR enabled**, you'll receive a response like this:

```json
{
  "success": true,
  "files": [{
    "file_id": "abc123",
    "filename": "code.png",
    "ocr": {
      "success": false,
      "error": "OCR not available",
      "message": "Text extraction from images is not enabled. Please manually enter your code or enable the OCR feature.",
      "confidence": 0.0,
      "help": "To enable OCR, rebuild the application with: cargo build --features tesseract"
    }
  }]
}
```

This is **expected behavior** when OCR is not enabled. Users can still manually enter their code.

## Enabling OCR (Optional)

If you want to enable automatic text extraction from images, follow these steps:

### Step 1: Install System Dependencies

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
    tesseract-ocr \
    libtesseract-dev \
    libleptonica-dev \
    pkg-config
```

#### macOS
```bash
brew install tesseract
brew install leptonica
brew install pkg-config
```

#### Fedora/RHEL
```bash
sudo dnf install -y \
    tesseract \
    tesseract-devel \
    leptonica-devel \
    pkgconfig
```

#### Arch Linux
```bash
sudo pacman -S tesseract leptonica pkgconf
```

### Step 2: Install Language Data (Optional)

For better OCR accuracy with different languages:

```bash
# Ubuntu/Debian
sudo apt-get install tesseract-ocr-eng tesseract-ocr-spa tesseract-ocr-fra

# macOS
brew install tesseract-lang

# Fedora
sudo dnf install tesseract-langpack-eng tesseract-langpack-spa
```

### Step 3: Build with OCR Feature

```bash
# Clean previous build
cargo clean

# Build with tesseract feature
cargo build --release --features tesseract

# Run with tesseract feature
cargo run --release --features tesseract
```

### Step 4: Verify OCR is Working

Upload an image and check the response:

```bash
curl -X POST http://localhost:3000/api/upload \
  -F "file=@your-code-screenshot.png"
```

With OCR enabled, you should see:

```json
{
  "success": true,
  "files": [{
    "file_id": "abc123",
    "filename": "code.png",
    "ocr": {
      "success": true,
      "text": "fn main() {\n    println!(\"Hello, world!\");\n}",
      "confidence": 0.95,
      "detected_language": "eng",
      "needs_review": false,
      "validation": {
        "is_valid": true,
        "issues": [],
        "suggestions": []
      }
    }
  }]
}
```

## Running Without OCR

The application works perfectly fine **without OCR**. Simply run:

```bash
cargo build --release --no-default-features
cargo run --release --no-default-features
```

Users will be prompted to manually enter their code when they upload images.

## Troubleshooting

### Error: "lept.pc not found"

This means Leptonica is not installed or pkg-config can't find it.

**Solution:**
1. Install leptonica-dev (see Step 1 above)
2. Verify installation:
   ```bash
   pkg-config --modversion lept
   ```

### Error: "tesseract not found"

**Solution:**
1. Install tesseract-ocr (see Step 1 above)
2. Verify installation:
   ```bash
   tesseract --version
   ```

### Low Confidence Scores

If OCR is working but returning low confidence scores:

1. **Improve image quality**: Use higher resolution screenshots
2. **Better contrast**: Ensure good contrast between text and background
3. **Clean images**: Avoid blurry or distorted screenshots
4. **Proper lighting**: Screenshots should be clear and well-lit

### OCR Returns Wrong Text

Common issues:
- **Similar characters**: OCR may confuse `0` (zero) with `O` (letter O), or `1` with `l`
- **Special characters**: Code symbols like `{}[]()` may be misrecognized
- **Indentation**: Whitespace may not be preserved perfectly

**Solution**: The application allows users to review and edit extracted text before generating the final image.

## Performance Considerations

- OCR processing adds 1-3 seconds to upload time
- Larger images take longer to process
- The application uses async processing to avoid blocking
- Consider setting a timeout for OCR operations (default: 30 seconds)

## Docker Deployment

If deploying with Docker, add these to your Dockerfile:

```dockerfile
# Install OCR dependencies
RUN apt-get update && apt-get install -y \
    tesseract-ocr \
    libtesseract-dev \
    libleptonica-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Build with tesseract feature
RUN cargo build --release --features tesseract
```

## Production Recommendations

For production deployments:

1. **Enable OCR** for better user experience
2. **Set appropriate timeouts** to prevent long-running OCR operations
3. **Monitor confidence scores** and alert on consistently low scores
4. **Provide manual fallback** (already implemented)
5. **Cache OCR results** for frequently uploaded images
6. **Rate limit** upload endpoints to prevent abuse

## Alternative: Cloud OCR Services

If you don't want to manage Tesseract dependencies, consider using cloud OCR services:

- Google Cloud Vision API
- AWS Textract
- Azure Computer Vision
- OCR.space API

These would require modifying the `src/services/ocr.rs` file to integrate with the chosen service.

## Summary

- **OCR is optional** - the app works fine without it
- **Users can manually enter code** when OCR is not available
- **Enable OCR** by installing system dependencies and building with `--features tesseract`
- **Production deployments** should enable OCR for better UX
