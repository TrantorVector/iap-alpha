#!/bin/bash
set -e

# Load environment variables
if [ -f .env ]; then
  # Use a more robust way to export env vars
  set -a
  source .env
  set +a
fi

# 1. Build API to ensure it's ready
echo "Building API..."
cargo build --manifest-path backend/Cargo.toml -p api

# 2. Start API in background
echo "Starting API..."
# Set absolute paths for keys
export JWT_PRIVATE_KEY_PATH="$(pwd)/secrets/private_key.pem"
export JWT_PUBLIC_KEY_PATH="$(pwd)/secrets/public_key.pem"

cd backend
cargo run -p api > api_log.txt 2>&1 &
API_PID=$!
cd ..

cleanup() {
    echo "Stopping API..."
    kill $API_PID || true
}
trap cleanup EXIT

echo "Waiting for API to start..."
# Retry loop for health check
MAX_RETRIES=30
for i in $(seq 1 $MAX_RETRIES); do
    if curl -s http://localhost:8080/health > /dev/null; then
        echo "API is up!"
        break
    fi
    echo "Waiting for API... ($i/$MAX_RETRIES)"
    sleep 2
done

# 3. Login to get token (using seed data user)
# Test user in seed data: testuser / TestPass123!
echo "Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "TestPass123!"}')

TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r .access_token)

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
    echo "Login failed. Output:"
    cat backend/api_log.txt
    exit 1
fi

echo "Got token."

# 4. Fetch Company ID for AAPL (seed data)
# ID: 10000000-0000-0000-0000-000000000001
COMPANY_ID="10000000-0000-0000-0000-000000000001"

# 5. Test GET /api/v1/companies/{id}/metrics
echo "Testing Company Metrics Endpoint..."
RESPONSE=$(curl -s -X GET "http://localhost:8080/api/v1/companies/$COMPANY_ID/metrics?period_type=quarterly&period_count=4" \
  -H "Authorization: Bearer $TOKEN")

# echo "Response Body: $RESPONSE"

# 6. Validate response
# Check for key sections and some seed data values
# AAPL Q4 2024 revenue was 94930000000 -> $94.93B or USD94.93B
if echo "$RESPONSE" | jq -e '.sections.growth_and_margins[] | select(.metric_name=="revenue")' > /dev/null; then
    echo "Found revenue metric row."
    if echo "$RESPONSE" | jq -e '.sections.growth_and_margins[] | select(.metric_name=="revenue") | .values[] | select(.formatted=="USD94.93B" or .formatted=="$94.93B")' > /dev/null; then
        echo "Found expected revenue value ($94.93B) in response."
    else
        echo "Expected revenue value ($94.93B) not found in response."
        echo "Actual values for revenue:"
        echo "$RESPONSE" | jq '.sections.growth_and_margins[] | select(.metric_name=="revenue") | .values'
        exit 1
    fi
else
    echo "Verification Failed: Growth and Margins section or revenue metric missing."
    exit 1
fi

if echo "$RESPONSE" | jq -e '.sections.cash_and_leverage' > /dev/null && echo "$RESPONSE" | jq -e '.sections.valuation' > /dev/null; then
    echo "Found cash_and_leverage and valuation sections."
else
    echo "Verification Failed: Sections missing."
    exit 1
fi

echo "Verification Passed: Company Metrics returned correctly with seed data."
