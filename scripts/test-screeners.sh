#!/bin/bash

# Test script for Screener API endpoints

set -e

echo "========================================="
echo "Testing Screener API Endpoints"
echo "========================================="
echo ""

API_BASE="http://localhost:8080/api/v1"

# Step 1: Login
echo "Step 1: Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"TestPass123!"}')

echo "$LOGIN_RESPONSE" | jq '.' > /dev/null 2>&1 || {
  echo "❌ Login failed or returned invalid JSON"
  echo "Response: $LOGIN_RESPONSE"
  exit 1
}

ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token')

if [ "$ACCESS_TOKEN" = "null" ] || [ -z "$ACCESS_TOKEN" ]; then
  echo "❌ Failed to get access token"
  echo "Response: $LOGIN_RESPONSE"
  exit 1
fi

echo "✓ Login successful"
echo ""

# Step 2: Create Screener
echo "Step 2: Creating screener..."
CREATE_PAYLOAD='{
  "title": "Test Screener",
  "description": "A test screener",
  "filter_criteria": {
    "exchanges": ["NASDAQ"],
    "market_cap_min": 1000000000.0
  },
  "sort_config": {
    "column": "market_cap",
    "direction": "desc"
  }
}'

CREATE_RESPONSE=$(curl -s -X POST "$API_BASE/screeners" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$CREATE_PAYLOAD")

echo "$CREATE_RESPONSE" | jq '.' > /dev/null 2>&1 || {
  echo "❌ Create screener failed"
  echo "Response: $CREATE_RESPONSE"
  exit 1
}

SCREENER_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
echo "✓ Screener created with ID: $SCREENER_ID"
echo ""

# Step 3: List Screeners
echo "Step 3: Listing screeners..."
LIST_RESPONSE=$(curl -s "$API_BASE/screeners" \
  -H "Authorization: Bearer $ACCESS_TOKEN")

echo "$LIST_RESPONSE" | jq '.' > /dev/null 2>&1 || {
  echo "❌ List screeners failed"
  echo "Response: $LIST_RESPONSE"
  exit 1
}

COUNT=$(echo "$LIST_RESPONSE" | jq length)
echo "✓ Screeners found: $COUNT"
echo ""

# Step 4: Get Screener
echo "Step 4: Get screener details..."
GET_RESPONSE=$(curl -s "$API_BASE/screeners/$SCREENER_ID" \
  -H "Authorization: Bearer $ACCESS_TOKEN")

echo "$GET_RESPONSE" | jq '.' > /dev/null 2>&1 || {
  echo "❌ Get screener failed"
  echo "Response: $GET_RESPONSE"
  exit 1
}

TITLE=$(echo "$GET_RESPONSE" | jq -r '.title')
if [ "$TITLE" != "Test Screener" ]; then
    echo "❌ Title mismatch: $TITLE"
    exit 1
fi
echo "✓ Screener details verified"
echo ""

# Step 5: Update Screener
echo "Step 5: Updating screener..."
UPDATE_PAYLOAD='{
  "title": "Updated Screener"
}'

UPDATE_RESPONSE=$(curl -s -X PUT "$API_BASE/screeners/$SCREENER_ID" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$UPDATE_PAYLOAD")

echo "$UPDATE_RESPONSE" | jq '.' > /dev/null 2>&1 || {
  echo "❌ Update screener failed"
  echo "Response: $UPDATE_RESPONSE"
  exit 1
}

NEW_TITLE=$(echo "$UPDATE_RESPONSE" | jq -r '.title')
if [ "$NEW_TITLE" != "Updated Screener" ]; then
    echo "❌ Update failed, title mismatch: $NEW_TITLE"
    exit 1
fi
echo "✓ Screener updated"
echo ""

# Step 6: Run Screener
echo "Step 6: Running screener..."
RUN_RESPONSE=$(curl -s -X POST "$API_BASE/screeners/$SCREENER_ID/run" \
  -H "Authorization: Bearer $ACCESS_TOKEN")

echo "$RUN_RESPONSE" | jq '.' > /dev/null 2>&1 || {
  echo "❌ Run screener failed"
  echo "Response: $RUN_RESPONSE"
  exit 1
}

TOTAL_RESULTS=$(echo "$RUN_RESPONSE" | jq '.total_results')
echo "✓ Screener ran successfully. Results: $TOTAL_RESULTS"

# Test run with override
echo "Running content override..."
RUN_OVERRIDE_RESPONSE=$(curl -s -X POST "$API_BASE/screeners/$SCREENER_ID/run" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{ "override_criteria": { "market_cap_min": 5000000000000.0 } }') # Very high, should return 0

echo "$RUN_OVERRIDE_RESPONSE" | jq '.' > /dev/null 2>&1 || {
    echo "❌ Run override failed"
    echo "Response: $RUN_OVERRIDE_RESPONSE"
    exit 1
}
OVERRIDE_RESULTS=$(echo "$RUN_OVERRIDE_RESPONSE" | jq '.total_results')
echo "✓ Run with override successful. Results: $OVERRIDE_RESULTS"
echo ""

# Step 7: Delete Screener
echo "Step 7: Deleting screener..."
DELETE_RESPONSE=$(curl -s -X DELETE "$API_BASE/screeners/$SCREENER_ID" \
  -H "Authorization: Bearer $ACCESS_TOKEN" -w "%{http_code}")

if [ "$DELETE_RESPONSE" != "204" ]; then
    echo "❌ Delete screener failed. Code: $DELETE_RESPONSE"
    exit 1
fi

echo "✓ Screener deleted"
echo ""

# Verify deletion
VERIFY_DELETE=$(curl -s "$API_BASE/screeners/$SCREENER_ID" \
  -H "Authorization: Bearer $ACCESS_TOKEN" -w "%{http_code}" -o /dev/null)

if [ "$VERIFY_DELETE" != "404" ]; then
    echo "❌ Screener still exists after delete (Code: $VERIFY_DELETE)"
    exit 1
fi
echo "✓ Deletion verified"
echo ""

echo "✅ All Screener API tests passed!"
