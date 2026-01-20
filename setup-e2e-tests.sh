#!/bin/bash
set -e

echo "🔧 Setting up E2E Test Environment..."

# Check if node is installed
if ! command -v node > /dev/null; then
    echo "❌ Node.js is not installed."
    echo ""
    echo "Please install Node.js first. You can:"
    echo "  1. Use your package manager: sudo apt install nodejs npm"
    echo "  2. Use nvm: curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash"
    echo "  3. Download from: https://nodejs.org/"
    exit 1
fi

echo "✅ Node.js found: $(node --version)"
echo "✅ npm found: $(npm --version)"

# Install dependencies
echo ""
echo "📦 Installing npm dependencies..."
npm install

# Install Playwright browsers
echo ""
echo "🌐 Installing Playwright browsers..."
npx playwright install

echo ""
echo "✅ Setup complete!"
echo ""
echo "You can now run tests with:"
echo "  npm run test:e2e         # Run all tests"
echo "  npm run test:e2e:ui      # Run in UI mode"
echo "  npm run test:e2e:headed  # Run in headed mode"
echo ""
