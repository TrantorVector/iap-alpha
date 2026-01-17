#!/bin/bash
set -e

# Load environment variables
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
fi

# 1. Build API to ensure it's ready
echo "Building API..."
cargo build --manifest-path backend/Cargo.toml -p api

# 2. Insert test data
echo "Inserting test user and company..."
# Fix: User schema uses username, display_name, timezone. No role.
# Fix: Handle conflict on symbol/exchange for company
docker exec -i iap-alpha-postgres-1 psql -U postgres -d irp_dev <<EOF
-- Insert User if not exists (password: password123)
INSERT INTO users (id, username, email, password_hash, display_name, timezone, is_active, created_at, updated_at)
VALUES ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'testuser', 'testuser@example.com', '\$argon2id\$v=19\$m=65536,t=3,p=4\$c236Pt+gD8usnvIe3ZJqqw\$r9Q/yNFUsqR8BK7UdRYIdHtAcy6iJkd4qkkZKI/47hY', 'Test User', 'UTC', true, NOW(), NOW())
ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash;

-- Insert Company
INSERT INTO companies (id, symbol, exchange, name, market_cap, currency, is_active, updated_at, created_at)
VALUES ('550e8400-e29b-41d4-a716-446655440000', 'AAPL', 'NASDAQ', 'Apple Inc.', 3000000000000, 'USD', true, NOW(), NOW())
ON CONFLICT DO NOTHING;
EOF

# 3. Start API in background
echo "Starting API..."
cargo run --manifest-path backend/Cargo.toml -p api > api_log.txt 2>&1 &
API_PID=$!

cleanup() {
    echo "Stopping API..."
    kill $API_PID || true
    rm -f api_log.txt
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

# 4. Login to get token
echo "Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "TestPass123!"}')

echo "Login Response: $LOGIN_RESPONSE"

TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r .access_token)


if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
    echo "Login failed. Output:"
    cat api_log.txt
    exit 1
fi

echo "Got token: ${TOKEN:0:10}..."

# 5. Fetch Company ID
COMPANY_ID=$(docker exec -i iap-alpha-postgres-1 psql -U postgres -d irp_dev -t -c "SELECT id FROM companies WHERE symbol='AAPL' AND exchange='NASDAQ'" | xargs)
echo "Company ID: $COMPANY_ID"

# 6. Test GET /api/v1/companies/{id}
echo "Testing Company Details Endpoint..."
RESPONSE=$(curl -s -X GET http://localhost:8080/api/v1/companies/$COMPANY_ID \
  -H "Authorization: Bearer $TOKEN")

echo "Response Body: $RESPONSE"

# 7. Validate response
# Expecting JSON with Apple Inc. and $3.0T formatting
if echo "$RESPONSE" | grep -q "Apple Inc." && echo "$RESPONSE" | grep -q "\$3.0T"; then
    echo "Verification Passed: Company Details returned correctly."
else
    echo "Verification Failed. Response did not match expectations."
    exit 1
fi
