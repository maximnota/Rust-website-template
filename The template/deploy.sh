#!/bin/bash

# Railway Deployment Script for Rust Web App
set -e

echo "🚀 Railway Deployment Script"
echo "============================"

# Check if Railway CLI is installed
if ! command -v railway &> /dev/null; then
    echo "❌ Railway CLI not found. Installing..."
    npm install -g @railway/cli
fi

# Function to deploy to Railway
deploy_to_railway() {
    echo "🚂 Deploying to Railway..."

    # Check if user is logged in
    if ! railway whoami &> /dev/null; then
        echo "🔑 Please login to Railway first:"
        railway login
    fi

    # Check if project exists
    if ! railway status &> /dev/null; then
        echo "📦 Initializing new Railway project..."
        railway init
    fi

    # Deploy
    echo "⬆️  Uploading and deploying..."
    railway up

    echo "✅ Deployment complete!"
    echo "🌐 Your app should be available at the Railway URL shown above"
}

# Function to test locally with Docker
test_local() {
    echo "🐳 Testing locally with Docker..."

    # Build Docker image
    echo "🔨 Building Docker image..."
    docker build -t rust-web-app .

    # Run container
    echo "🏃 Starting container on http://localhost:3030..."
    echo "Press Ctrl+C to stop the container"
    docker run -p 3030:3030 -e PORT=3030 rust-web-app
}

# Function to clean up
cleanup() {
    echo "🧹 Cleaning up build artifacts..."
    cargo clean
    docker image rm rust-web-app 2>/dev/null || true
    echo "✅ Cleanup complete!"
}

# Main menu
echo ""
echo "Choose an option:"
echo "1) Deploy to Railway"
echo "2) Test locally with Docker"
echo "3) Clean up build artifacts"
echo "4) Exit"
echo ""

read -p "Enter your choice (1-4): " choice

case $choice in
    1)
        deploy_to_railway
        ;;
    2)
        test_local
        ;;
    3)
        cleanup
        ;;
    4)
        echo "👋 Goodbye!"
        exit 0
        ;;
    *)
        echo "❌ Invalid option. Please choose 1-4."
        exit 1
        ;;
esac
