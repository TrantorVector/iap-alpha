#!/bin/bash
set -e

# Load environment variables
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
fi

echo "================================================="
echo "Verifying Verdict Update Endpoint (Step 7.4.2)"
echo "================================================="

if [ -z "$USE_DOCKER" ]; then
    # 1. Build API
    echo "Building API..."
    cargo build --manifest-path backend/Cargo.toml -p api

    # 3. Start API
    echo "Starting API..."
    cargo run --manifest-path backend/Cargo.toml -p api > api_log.txt 2>&1 &
    API_PID=$!

    cleanup() {
        echo "Stopping API..."
        kill $API_PID || true
        rm -f api_log.txt
    }
    trap cleanup EXIT
else
    echo "Using running Docker container for API..."
fi

# 2. Insert test data (Idempotent)
echo "Ensuring test data..."
docker exec -i iap-alpha-postgres-1 psql -U postgres -d irp_dev <<EOF
INSERT INTO users (id, username, email, password_hash, display_name, timezone, is_active, created_at, updated_at)
VALUES ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'testuser', 'testuser@example.com', '\$argon2id\$v=19\$m=65536,t=3,p=4\$c236Pt+gD8usnvIe3ZJqqw\$r9Q/yNFUsqR8BK7UdRYIdHtAcy6iJkd4qkkZKI/47hY', 'Test User', 'UTC', true, NOW(), NOW())
ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash;

INSERT INTO companies (id, symbol, exchange, name, market_cap, currency, is_active, updated_at, created_at)
VALUES ('550e8400-e29b-41d4-a716-446655440000', 'AAPL', 'NASDAQ', 'Apple Inc.', 3000000000000, 'USD', true, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

EOF

echo "Waiting for API..."
MAX_RETRIES=30
for i in $(seq 1 $MAX_RETRIES); do
    if curl -s http://localhost:8080/health > /dev/null; then
        echo "API is up!"
        break
    fi
    echo "Waiting... ($i/$MAX_RETRIES)"
    sleep 2
done

# 4. Login
echo "Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "TestPass123!"}')
TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r .access_token)

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
    echo "Login failed."
    exit 1
fi

# Fetch actual Company ID
COMPANY_ID=$(docker exec -i iap-alpha-postgres-1 psql -U postgres -d irp_dev -t -c "SELECT id FROM companies WHERE symbol='AAPL' AND exchange='NASDAQ'" | xargs)
echo "Using Company ID: $COMPANY_ID"

if [ -z "$COMPANY_ID" ]; then
    echo "Error: Could not find company AAPL"
    exit 1
fi

# Clean up any existing verdict for clean slate (using fetched ID)
docker exec -i iap-alpha-postgres-1 psql -U postgres -d irp_dev <<EOF
DELETE FROM verdicts WHERE company_id = '$COMPANY_ID';
DELETE FROM analysis_reports WHERE verdict_id IN (SELECT id FROM verdicts WHERE company_id = '$COMPANY_ID');
EOF

# 5. Create new verdict (PUT with lock_version: 0)
echo "Test 1: Create Verdict (lock_version: 0)"
RESPONSE=$(curl -s -X PUT http://localhost:8080/api/v1/companies/$COMPANY_ID/verdict \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "lock_version": 0,
    "final_verdict": "INVEST",
    "summary_text": "Strong buy",
    "strengths": ["Strong Brand", "Cash Flow"],
    "weaknesses": ["Market Saturation"],
    "guidance_summary": "Upward revision"
  }')

echo "Response Body: $RESPONSE"

VERDICT_ID=$(echo "$RESPONSE" | jq -r .verdict_id)
LOCK_VERSION=$(echo "$RESPONSE" | jq -r .lock_version)

if [ "$VERDICT_ID" != "null" ] && [ "$LOCK_VERSION" == "0" ]; then
    echo "✓ Created successfully"
else
    echo "✗ Failed to create"
    exit 1
fi

# 6. Update verdict (PUT with correct lock_version)
echo "Test 2: Update Verdict (lock_version: 0 -> 1)"
RESPONSE=$(curl -s -X PUT http://localhost:8080/api/v1/companies/$COMPANY_ID/verdict \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "lock_version": 0,
    "final_verdict": "WATCHLIST",
    "summary_text": "Updated summary",
    "strengths": ["New Strength"],
    "weaknesses": [],
    "guidance_summary": "Neutral"
  }')

NEW_VERSION=$(echo "$RESPONSE" | jq -r .lock_version)
FINAL_VERDICT=$(echo "$RESPONSE" | jq -r .final_verdict)

if [ "$NEW_VERSION" == "1" ] && [ "$FINAL_VERDICT" == "WATCHLIST" ]; then
    echo "✓ Updated successfully"
else
    echo "✗ Update failed. Expected version 1, got $NEW_VERSION"
    exit 1
fi

# 7. Conflict Test (PUT with old lock_version: 0)
echo "Test 3: Optimistic Locking Conflict (lock_version: 0)"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X PUT http://localhost:8080/api/v1/companies/$COMPANY_ID/verdict \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "lock_version": 0,
    "final_verdict": "PASS",
    "strengths": [],
    "weaknesses": []
  }')

echo "HTTP Code: $HTTP_CODE"

if [ "$HTTP_CODE" == "409" ]; then
    echo "✓ Conflict handled correctly (409)"
else
    echo "✗ Expected 409, got $HTTP_CODE"
    exit 1
fi

echo "ALL TESTS PASSED"
