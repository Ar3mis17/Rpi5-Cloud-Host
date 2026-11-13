#!/bin/bash
# Vercel Deployment Script for Ar3mis Cloud Frontend

set -euo pipefail

echo "========================================"
echo "Ar3mis Cloud - Vercel Frontend Deployment"
echo "========================================"
echo ""

# Check if Vercel CLI is installed
if ! command -v vercel &> /dev/null; then
    echo "Installing Vercel CLI..."
    npm install -g vercel
fi

# Navigate to frontend directory
cd /opt/ar3mis-cloud/frontend

# Check if package.json exists
if [ ! -f "package.json" ]; then
    echo "Error: package.json not found in frontend directory"
    exit 1
fi

echo "Installing dependencies..."
npm install

echo "Building project..."
npm run build

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo "Build successful!"
echo ""

# Check if vercel.json exists, create if not
if [ ! -f "vercel.json" ]; then
    cat > vercel.json << 'EOF'
{
  "buildCommand": "npm run build",
  "outputDirectory": ".next",
  "devCommand": "npm run dev",
  "installCommand": "npm install",
  "framework": "nextjs",
  "regions": ["iad1"],
  "functions": {
    "src/app/api/**/*.ts": {
      "maxDuration": 30
    }
  },
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        {
          "key": "X-Content-Type-Options",
          "value": "nosniff"
        },
        {
          "key": "X-Frame-Options",
          "value": "DENY"
        },
        {
          "key": "X-XSS-Protection",
          "value": "1; mode=block"
        }
      ]
    }
  ],
  "rewrites": [
    {
      "source": "/api/backend/:path*",
      "destination": "https://cloud.ar3mis.com/api/v1/:path*"
    }
  ]
}
EOF
    echo "Created vercel.json"
fi

echo ""
echo "========================================"
echo "Ready for Vercel Deployment"
echo "========================================"
echo ""
echo "Next steps:"
echo "1. Run: vercel login"
echo "2. Run: vercel --prod"
echo ""
echo "Or deploy via GitHub:"
echo "1. Go to https://vercel.com/new"
echo "2. Import: Ar3mis17/Rpi5-Cloud-Host"
echo "3. Set Root Directory: frontend"
echo "4. Add Environment Variable:"
echo "   NEXT_PUBLIC_API_URL=https://cloud.ar3mis.com"
echo "5. Deploy"
echo ""
echo "After deployment, update Caddyfile to include your Vercel domain"
echo "and configure CORS in backend to allow your Vercel domain."