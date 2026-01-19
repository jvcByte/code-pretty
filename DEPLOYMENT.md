# Deployment Guide

This guide covers deploying the Code Snippet Designer application to various hosting platforms.

## Quick Start with Docker

The easiest way to run the application with all dependencies (including OCR) is using Docker:

```bash
# Build and run with Docker Compose
docker-compose up -d

# Or build and run manually
docker build -t code-snippet-designer .
docker run -p 3000:3000 code-snippet-designer
```

The application will be available at `http://localhost:3000` with OCR fully enabled.

## Deployment Options

### 1. Railway (Recommended - Easiest)

Railway provides free hosting with automatic Docker deployment.

**Steps:**

1. **Create account** at [railway.app](https://railway.app)

2. **Install Railway CLI** (optional):
   ```bash
   npm install -g @railway/cli
   railway login
   ```

3. **Deploy from GitHub**:
   - Push your code to GitHub
   - Go to Railway dashboard
   - Click "New Project" → "Deploy from GitHub repo"
   - Select your repository
   - Railway will automatically detect the Dockerfile and deploy

4. **Or deploy via CLI**:
   ```bash
   railway init
   railway up
   ```

5. **Set environment variables** (optional):
   ```bash
   railway variables set MAX_FILE_SIZE=10485760
   railway variables set REQUEST_TIMEOUT_SECONDS=30
   ```

6. **Get your URL**:
   - Railway will provide a public URL like `https://your-app.railway.app`

**Cost**: Free tier includes 500 hours/month and $5 credit

---

### 2. Render

Render offers free hosting with Docker support.

**Steps:**

1. **Create account** at [render.com](https://render.com)

2. **Create `render.yaml`** (already included below)

3. **Deploy**:
   - Connect your GitHub repository
   - Render will automatically deploy using the Dockerfile

4. **Access your app**:
   - Render provides a URL like `https://your-app.onrender.com`

**Cost**: Free tier available (spins down after inactivity)

---

### 3. Fly.io

Fly.io provides excellent Docker support and global deployment.

**Steps:**

1. **Install Fly CLI**:
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

2. **Login**:
   ```bash
   fly auth login
   ```

3. **Launch app**:
   ```bash
   fly launch
   ```
   - Follow the prompts
   - Fly will detect your Dockerfile

4. **Deploy**:
   ```bash
   fly deploy
   ```

5. **Set secrets** (optional):
   ```bash
   fly secrets set MAX_FILE_SIZE=10485760
   ```

6. **Access your app**:
   - Fly provides a URL like `https://your-app.fly.dev`

**Cost**: Free tier includes 3 shared VMs

---

### 4. DigitalOcean App Platform

**Steps:**

1. **Create account** at [digitalocean.com](https://digitalocean.com)

2. **Create new app**:
   - Go to App Platform
   - Connect your GitHub repository
   - Select "Dockerfile" as build method

3. **Configure**:
   - Set port to `3000`
   - Add environment variables if needed

4. **Deploy**:
   - DigitalOcean will build and deploy automatically

**Cost**: Starts at $5/month

---

### 5. AWS (Advanced)

#### Option A: AWS App Runner (Easiest)

```bash
# Build and push to ECR
aws ecr create-repository --repository-name code-snippet-designer
docker build -t code-snippet-designer .
docker tag code-snippet-designer:latest <account-id>.dkr.ecr.<region>.amazonaws.com/code-snippet-designer:latest
docker push <account-id>.dkr.ecr.<region>.amazonaws.com/code-snippet-designer:latest

# Create App Runner service via console or CLI
aws apprunner create-service --service-name code-snippet-designer \
  --source-configuration ImageRepository={...}
```

#### Option B: ECS Fargate

See `aws-deployment/` directory for CloudFormation templates.

**Cost**: Pay per use, typically $10-30/month for small apps

---

### 6. Google Cloud Run

**Steps:**

1. **Install gcloud CLI**:
   ```bash
   curl https://sdk.cloud.google.com | bash
   ```

2. **Login and set project**:
   ```bash
   gcloud auth login
   gcloud config set project YOUR_PROJECT_ID
   ```

3. **Build and deploy**:
   ```bash
   gcloud run deploy code-snippet-designer \
     --source . \
     --platform managed \
     --region us-central1 \
     --allow-unauthenticated
   ```

4. **Access your app**:
   - Cloud Run provides a URL like `https://code-snippet-designer-xxx.run.app`

**Cost**: Free tier includes 2 million requests/month

---

### 7. Heroku

**Steps:**

1. **Install Heroku CLI**:
   ```bash
   curl https://cli-assets.heroku.com/install.sh | sh
   ```

2. **Login**:
   ```bash
   heroku login
   ```

3. **Create app**:
   ```bash
   heroku create your-app-name
   ```

4. **Deploy**:
   ```bash
   heroku container:push web
   heroku container:release web
   ```

5. **Open app**:
   ```bash
   heroku open
   ```

**Cost**: Free tier discontinued, starts at $5/month

---

### 8. Self-Hosted (VPS)

For VPS providers like Linode, Vultr, or Hetzner:

**Steps:**

1. **SSH into your server**:
   ```bash
   ssh user@your-server-ip
   ```

2. **Install Docker**:
   ```bash
   curl -fsSL https://get.docker.com -o get-docker.sh
   sh get-docker.sh
   ```

3. **Clone repository**:
   ```bash
   git clone <your-repo-url>
   cd code-snippet-designer
   ```

4. **Run with Docker Compose**:
   ```bash
   docker-compose up -d
   ```

5. **Setup reverse proxy** (Nginx):
   ```nginx
   server {
       listen 80;
       server_name your-domain.com;
       
       location / {
           proxy_pass http://localhost:3000;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
       }
   }
   ```

6. **Setup SSL** with Let's Encrypt:
   ```bash
   sudo apt install certbot python3-certbot-nginx
   sudo certbot --nginx -d your-domain.com
   ```

**Cost**: $5-10/month for basic VPS

---

## Comparison Table

| Platform | Free Tier | Ease of Use | OCR Support | Best For |
|----------|-----------|-------------|-------------|----------|
| **Railway** | ✅ 500hrs | ⭐⭐⭐⭐⭐ | ✅ | Quick deployment |
| **Render** | ✅ Limited | ⭐⭐⭐⭐⭐ | ✅ | Free hosting |
| **Fly.io** | ✅ 3 VMs | ⭐⭐⭐⭐ | ✅ | Global deployment |
| **Cloud Run** | ✅ 2M req | ⭐⭐⭐⭐ | ✅ | Serverless |
| **DigitalOcean** | ❌ $5/mo | ⭐⭐⭐⭐ | ✅ | Reliable hosting |
| **AWS** | ❌ Complex | ⭐⭐⭐ | ✅ | Enterprise |
| **VPS** | ❌ $5/mo | ⭐⭐⭐ | ✅ | Full control |

---

## Recommended: Railway Deployment

For most users, Railway is the best option:

### Quick Deploy to Railway

1. **One-click deploy**:
   [![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/new/template)

2. **Or via CLI**:
   ```bash
   npm install -g @railway/cli
   railway login
   railway init
   railway up
   ```

3. **Done!** Your app is live with OCR enabled.

---

## Environment Variables

Configure these in your hosting platform:

```bash
# Server Configuration
HOST=0.0.0.0
PORT=3000

# File Upload
MAX_FILE_SIZE=10485760  # 10MB in bytes

# Performance
REQUEST_TIMEOUT_SECONDS=30

# Storage
TEMP_DIR=/tmp/code-snippet-designer

# CORS (optional)
CORS_ORIGINS=*  # Or specific domains: https://yourdomain.com

# Logging
RUST_LOG=info  # Options: error, warn, info, debug, trace
```

---

## Monitoring and Maintenance

### Health Checks

All platforms support health checks. The app provides:
- **Endpoint**: `GET /health`
- **Response**: `{"status":"healthy","service":"code-snippet-designer"}`

### Logs

View logs on each platform:
```bash
# Railway
railway logs

# Render
# View in dashboard

# Fly.io
fly logs

# Docker
docker-compose logs -f
```

### Scaling

Most platforms auto-scale. For manual scaling:

```bash
# Fly.io
fly scale count 3

# Railway
# Use dashboard

# Docker (manual)
docker-compose up -d --scale app=3
```

---

## Custom Domain

### Railway
1. Go to Settings → Domains
2. Add your custom domain
3. Update DNS records as shown

### Render
1. Go to Settings → Custom Domain
2. Add domain and update DNS

### Fly.io
```bash
fly certs add yourdomain.com
```

---

## Troubleshooting

### OCR Not Working

Check if Tesseract is installed in container:
```bash
docker exec -it code-snippet-designer tesseract --version
```

### Out of Memory

Increase memory limits:
```yaml
# docker-compose.yml
services:
  app:
    deploy:
      resources:
        limits:
          memory: 1G
```

### Slow Performance

- Enable caching
- Use CDN for static files
- Scale horizontally

---

## Security Checklist

- [ ] Set `MAX_FILE_SIZE` to prevent abuse
- [ ] Configure `CORS_ORIGINS` for production
- [ ] Use HTTPS (automatic on most platforms)
- [ ] Set up rate limiting
- [ ] Monitor logs for suspicious activity
- [ ] Keep dependencies updated

---

## Cost Estimates

**Free Tier Options:**
- Railway: Free for hobby projects
- Render: Free with limitations
- Fly.io: Free for 3 small VMs
- Cloud Run: Free for 2M requests/month

**Paid Options:**
- Railway: ~$5-10/month
- DigitalOcean: $5-12/month
- VPS: $5-10/month
- AWS/GCP: $10-30/month (varies)

---

## Next Steps

1. Choose a platform from the table above
2. Follow the deployment steps
3. Configure environment variables
4. Test the OCR functionality
5. Set up a custom domain (optional)
6. Monitor and scale as needed

For questions or issues, check the troubleshooting section or open an issue on GitHub.
