#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Initializing Environment...${NC}"

# 1. Load NVM (Node Version Manager)
# This fixes the "npx not found" error by ensuring Node is in the PATH
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

# Verify Node is loaded
if ! command -v node &> /dev/null; then
    echo -e "${YELLOW}Node.js not detected. Attempting to load default version...${NC}"
    nvm use default &> /dev/null || nvm use stable &> /dev/null
fi

if ! command -v npx &> /dev/null; then
    echo "❌ Error: 'npx' command is still missing. setup failed."
    echo "Please try restarting your terminal session."
    exit 1
fi

echo -e "${GREEN}✅ Node.js $(node -v) and npx $(npx -v) are ready.${NC}"

# 2. Install Playwright System Dependencies
echo -e "\n${BLUE}📦 Installing Playwright System Dependencies...${NC}"
echo -e "${YELLOW}⚠️  This step requires sudo privileges. Please enter your password if prompted.${NC}"

npx playwright install-deps

echo -e "${GREEN}✅ Dependencies installed.${NC}"

# 3. Running Tests
echo -e "\n${BLUE}🚀 Running E2E Tests...${NC}"
npm run test:e2e

echo -e "\n${GREEN}✨ All steps completed!${NC}"
