#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting Detailed CI Simulation...${NC}"

# ==========================================
# Frontend CI Checks
# ==========================================
echo -e "\n${GREEN}=== Running Frontend Checks ===${NC}"
if [ -d "frontend" ]; then
    cd frontend
    
    echo "Installing dependencies..."
    npm ci
    
    echo "Checking formatting..."
    npx prettier --check .
    
    echo "Running ESLint..."
    npm run lint
    
    echo "Running Frontend Tests..."
    npm test -- --passWithNoTests
    
    cd ..
else
    echo -e "${RED}Frontend directory not found! Skipping...${NC}"
fi

# ==========================================
# Backend CI Checks
# ==========================================
echo -e "\n${GREEN}=== Running Backend Checks ===${NC}"
cd backend

echo "Checking Code Formatting..."
cargo fmt --all -- --check

echo "Running Clippy..."
cargo clippy --all-targets -- -D warnings

# Database Setup for Tests
echo -e "\n${GREEN}=== Setting up Test Database ===${NC}"
# Assumes docker container 'postgres' or similar is running.
# We try to detect the container name from docker compose if possible, or assume 'iap-alpha-postgres-1' or similar depending on project name.
# Use 'docker compose exec' to be safe.

# Source the .env from parent to get DB credentials if needed, but CI config uses defaults.
# Local environment might differ. We'll use the .env values but override the DB name to test_db.

# Extract DB info from .env
DB_HOST=localhost
DB_PORT=5432
DB_USER=postgres
DB_PASSWORD=dev 
# Note: CI uses 'password' but local .env uses 'dev'. We must match the local running container's password.

export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/test_db"
echo "Targeting Test DB: $DATABASE_URL"

# Create test_db if it doesn't exist
# We use docker compose to execute psql inside the container
echo "Creating test_db if needed..."
docker compose -f ../docker-compose.yml exec -T postgres psql -U $DB_USER -c "DROP DATABASE IF EXISTS test_db;"
docker compose -f ../docker-compose.yml exec -T postgres psql -U $DB_USER -c "CREATE DATABASE test_db;"

echo "Running Migrations on test_db..."
cargo run --example migrate

echo "Running Backend Tests..."
cargo test --workspace -- --test-threads=1

echo -e "\n${GREEN}✅ CI Simulation Completed Successfully!${NC}"
