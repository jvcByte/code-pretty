# Architecture Overview

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         User's Browser                       │
│  (No installation needed - just visit the website)          │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ HTTPS
                         │
┌────────────────────────▼────────────────────────────────────┐
│                    Your Hosted App                           │
│              (Railway/Render/Fly.io/etc.)                    │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Docker Container                         │  │
│  │                                                       │  │
│  │  ┌─────────────────────────────────────────────┐    │  │
│  │  │   Rust Web Server (Axum)                    │    │  │
│  │  │   - Handles HTTP requests                   │    │  │
│  │  │   - Serves static files                     │    │  │
│  │  │   - Manages sessions                        │    │  │
│  │  └──────────────┬──────────────────────────────┘    │  │
│  │                 │                                    │  │
│  │  ┌──────────────▼──────────────────────────────┐    │  │
│  │  │   OCR Service (Tesseract)                   │    │  │
│  │  │   - Extracts text from images               │    │  │
│  │  │   - Returns confidence scores               │    │  │
│  │  │   - Bundled in Docker - no setup needed!    │    │  │
│  │  └──────────────┬──────────────────────────────┘    │  │
│  │                 │                                    │  │
│  │  ┌──────────────▼──────────────────────────────┐    │  │
│  │  │   Image Generator                           │    │  │
│  │  │   - Syntax highlighting                     │    │  │
│  │  │   - Theme application                       │    │  │
│  │  │   - PNG/JPG/SVG export                      │    │  │
│  │  └─────────────────────────────────────────────┘    │  │
│  │                                                       │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## User Flow

### 1. Upload Screenshot (with OCR)

```
User uploads image
       │
       ▼
Upload Handler receives file
       │
       ▼
OCR Service extracts text
       │
       ├─ Success: Returns text + confidence
       │           User can review/edit
       │
       └─ Failure: User enters code manually
       │
       ▼
User customizes theme
       │
       ▼
Image Generator creates visual
       │
       ▼
User downloads PNG/JPG/SVG
```

### 2. Direct Code Entry (no OCR needed)

```
User pastes/types code
       │
       ▼
Language auto-detected
       │
       ▼
User customizes theme
       │
       ▼
Image Generator creates visual
       │
       ▼
User downloads PNG/JPG/SVG
```

## Deployment Architecture

### Development (Local)

```
┌─────────────────────────────────────┐
│  Your Computer                      │
│                                     │
│  docker-compose up -d               │
│         │                           │
│         ▼                           │
│  ┌──────────────────────┐          │
│  │  Docker Container    │          │
│  │  - Rust App          │          │
│  │  - Tesseract OCR     │          │
│  │  - All dependencies  │          │
│  └──────────────────────┘          │
│         │                           │
│         ▼                           │
│  http://localhost:3000              │
└─────────────────────────────────────┘
```

### Production (Cloud)

```
┌─────────────────────────────────────────────────────────┐
│  Railway/Render/Fly.io                                  │
│                                                         │
│  1. Reads Dockerfile                                    │
│  2. Builds Docker image with Tesseract                  │
│  3. Deploys container                                   │
│  4. Provides public URL                                 │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Docker Container (Auto-built)                   │  │
│  │  ✅ Rust app                                     │  │
│  │  ✅ Tesseract OCR                                │  │
│  │  ✅ All dependencies                             │  │
│  │  ✅ Health checks                                │  │
│  │  ✅ Auto-restart                                 │  │
│  └──────────────────────────────────────────────────┘  │
│                                                         │
│  https://your-app.railway.app                           │
└─────────────────────────────────────────────────────────┘
                         │
                         │ Users access via browser
                         ▼
                  ┌──────────────┐
                  │   Internet   │
                  │    Users     │
                  └──────────────┘
```

## Why Docker Solves Everything

### Without Docker ❌

```
Developer's Machine:
1. Install Rust ✅
2. Install Tesseract ❌ (complex)
3. Install Leptonica ❌ (complex)
4. Configure paths ❌ (error-prone)
5. Build app ❌ (might fail)

Production Server:
1. Install Rust ✅
2. Install Tesseract ❌ (complex)
3. Install Leptonica ❌ (complex)
4. Configure paths ❌ (error-prone)
5. Deploy app ❌ (might fail)

Result: Lots of manual work, many failure points
```

### With Docker ✅

```
Developer's Machine:
1. docker-compose up -d ✅

Production Server:
1. railway up ✅

Result: Everything just works!
```

## Technology Stack

### Backend
- **Rust** - Fast, safe, concurrent
- **Axum** - Modern web framework
- **Tokio** - Async runtime

### OCR
- **Tesseract** - Industry-standard OCR
- **Leptonica** - Image processing
- **Bundled in Docker** - No manual setup

### Image Processing
- **image-rs** - Image manipulation
- **resvg** - SVG rendering
- **syntect** - Syntax highlighting

### Frontend
- **Vanilla JavaScript** - No framework needed
- **Modern CSS** - Responsive design
- **Progressive Web App** - Works offline

### Deployment
- **Docker** - Containerization
- **Railway/Render/Fly.io** - Cloud hosting
- **Automatic CI/CD** - Push to deploy

## Security

```
┌─────────────────────────────────────┐
│  Security Layers                    │
│                                     │
│  1. HTTPS (automatic on platforms)  │
│  2. File size limits                │
│  3. File type validation            │
│  4. Rate limiting                   │
│  5. Input sanitization              │
│  6. Non-root Docker user            │
│  7. Temporary file cleanup          │
└─────────────────────────────────────┘
```

## Scalability

### Horizontal Scaling

```
Load Balancer
      │
      ├─── Container 1 (Rust + OCR)
      │
      ├─── Container 2 (Rust + OCR)
      │
      └─── Container 3 (Rust + OCR)
```

All platforms support auto-scaling:
- Railway: Automatic
- Render: Automatic
- Fly.io: `fly scale count 3`
- Cloud Run: Automatic

### Performance

- **Async I/O**: Non-blocking operations
- **Connection pooling**: Efficient resource use
- **Caching**: Theme and syntax definitions
- **Streaming**: Large file uploads
- **CDN**: Static file delivery

## Monitoring

```
Application
    │
    ├─ Health Check (/health)
    │  └─ Monitored every 30s
    │
    ├─ Logs (stdout/stderr)
    │  └─ Aggregated by platform
    │
    └─ Metrics
       ├─ Request count
       ├─ Response time
       ├─ Error rate
       └─ OCR success rate
```

## Cost Optimization

### Free Tier Strategy

```
Railway Free Tier
├─ 500 hours/month
├─ $5 credit
└─ Auto-sleep when idle

Render Free Tier
├─ Unlimited hours
├─ Spins down after 15min idle
└─ Spins up on request

Fly.io Free Tier
├─ 3 shared VMs
├─ 160GB bandwidth
└─ Always on
```

### Paid Tier ($5-10/month)

```
Better Performance
├─ No auto-sleep
├─ More memory
├─ Faster CPU
└─ More bandwidth
```

## Summary

✅ **Simple Architecture**: Web server + OCR + Image generator  
✅ **Docker Bundles Everything**: No manual installation  
✅ **Easy Deployment**: One command to production  
✅ **Scalable**: Horizontal scaling supported  
✅ **Secure**: Multiple security layers  
✅ **Cost-Effective**: Free tier available  

The key insight: **Docker solves the Tesseract installation problem** by bundling everything into a single container that works everywhere.
