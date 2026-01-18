#!/bin/bash
set -e

# Load environment variables
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
fi
export SERVER_PORT=8081

echo "================================================="
echo "Verifying Verdict History Endpoint (Step 7.4.3)"
echo "================================================="

if [ -z "$USE_DOCKER" ]; then
    # 1. Build API
    echo "Building API..."
    cargo build --manifest-path backend/Cargo.toml -p api

    # 3. Start API
    echo "Starting API on port $SERVER_PORT..."
    cargo run --manifest-path backend/Cargo.toml -p api > api_log.txt 2>&1 &
    API_PID=$!

    cleanup() {
        echo "Stopping API..."
        kill $API_PID || true
        # rm -f api_log.txt
    }
    trap cleanup EXIT
else
    echo "Using running Docker container for API..."
    # Note: If reusing docker, mapping might be 8080. But we assume we are running locally here.
fi

# 1. Login
echo "Logging in..."
MAX_RETRIES=10
for i in $(seq 1 $MAX_RETRIES); do
    if curl -s http://localhost:8081/health > /dev/null; then
        break
    fi
    echo "Waiting for API... ($i/$MAX_RETRIES)"
    sleep 2
done

LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "TestPass123!"}')
TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r .access_token)

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
    echo "Login failed."
    echo $LOGIN_RESPONSE
    exit 1
fi

# Fetch Company ID
COMPANY_ID=$(docker exec -i iap-alpha-postgres-1 psql -U postgres -d irp_dev -t -c "SELECT id FROM companies WHERE symbol='AAPL' AND exchange='NASDAQ'" | xargs)
echo "Using Company ID: $COMPANY_ID"

if [ -z "$COMPANY_ID" ]; then
    echo "Error: Could not find company AAPL"
    exit 1
fi

# Reset verdicts
echo "Resetting verdicts..."
docker exec -i iap-alpha-postgres-1 psql -U postgres -d irp_dev <<EOF
DELETE FROM verdicts WHERE company_id = '$COMPANY_ID';
DELETE FROM verdict_history WHERE verdict_id IN (SELECT id FROM verdicts WHERE company_id = '$COMPANY_ID');
EOF

# 2. Create Initial Verdict (V0)
echo "Creating V0 Verdict..."
RESPONSE=$(curl -s -X PUT http://localhost:8081/api/v1/companies/$COMPANY_ID/verdict \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "lock_version": 0,
    "final_verdict": "WATCHLIST",
    "summary_text": "Initial V0",
    "strengths": ["S1"],
    "weaknesses": ["W1"]
  }')
# This works differently depending on if it exists. If not exists, it creates.

# 3. Update Verdict to V1 (Should Snapshot V0)
echo "Updating to V1..."
RESPONSE=$(curl -s -X PUT http://localhost:8081/api/v1/companies/$COMPANY_ID/verdict \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "lock_version": 0,
    "final_verdict": "INVEST",
    "summary_text": "Update V1",
    "strengths": ["S2"],
    "weaknesses": ["W2"]
  }')

VERSION=$(echo "$RESPONSE" | jq -r .lock_version)
echo "Current Version: $VERSION"

if [ "$VERSION" != "1" ]; then
    echo "Failed to update to V1"
    echo $RESPONSE
    exit 1
fi

# 4. Update Verdict to V2 (Should Snapshot V1)
echo "Updating to V2..."
RESPONSE=$(curl -s -X PUT http://localhost:8081/api/v1/companies/$COMPANY_ID/verdict \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "lock_version": 1,
    "final_verdict": "PASS",
    "summary_text": "Update V2",
    "strengths": ["S3"],
    "weaknesses": ["W3"]
  }')

VERSION=$(echo "$RESPONSE" | jq -r .lock_version)
echo "Current Version: $VERSION"

if [ "$VERSION" != "2" ]; then
    echo "Failed to update to V2"
    echo $RESPONSE
    exit 1
fi

# 5. Get History
echo "Fetching History..."
HISTORY=$(curl -s -X GET http://localhost:8081/api/v1/companies/$COMPANY_ID/verdict/history \
  -H "Authorization: Bearer $TOKEN")

echo "History Response: $HISTORY"

# Validate History
# Expecting entries with version 1 and 0 (descending)
ENTRY_0_VER=$(echo "$HISTORY" | jq -r '.history[0].version')
ENTRY_0_TXT=$(echo "$HISTORY" | jq -r '.history[0].summary_text')

ENTRY_1_VER=$(echo "$HISTORY" | jq -r '.history[1].version')
ENTRY_1_TXT=$(echo "$HISTORY" | jq -r '.history[1].summary_text')

echo "History[0]: Version $ENTRY_0_VER, Text: $ENTRY_0_TXT"
echo "History[1]: Version $ENTRY_1_VER, Text: $ENTRY_1_TXT"

if [ "$ENTRY_0_VER" == "1" ] && [ "$ENTRY_0_TXT" == "Update V1" ] && \
   [ "$ENTRY_1_VER" == "0" ] && [ "$ENTRY_1_TXT" == "Initial V0" ]; then
    echo "✓ History verification passed!"
else
    echo "✗ History verification failed."
    if [ -f api_log.txt ]; then
        echo "=== API LOG ==="
        cat api_log.txt
        echo "==============="
    fi
    exit 1
fi

echo "ALL TESTS PASSED"
