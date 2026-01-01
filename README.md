# Code Snippet Designer

A web application that transforms code snippets into visually appealing images for sharing and presentations.

## Features

- Upload screenshots and extract code using OCR
- Paste or type code directly
- Customize visual themes and styling
- Export high-quality images in multiple formats
- Responsive web interface

## Development

### Prerequisites

- Rust 1.70+
- Tesseract OCR library

### Setup

1. Clone the repository
2. Copy `.env.example` to `.env` and configure as needed
3. Install dependencies: `cargo build`
4. Run the development server: `cargo run`

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