#!/bin/bash

# Test the exact login flow that the browser is using

echo "Testing Browser Login Flow..."
echo "=============================="
echo ""

# Step 1: Login via /api/v1 (what the browser uses)
echo "1. Logging in via /api/v1/auth/login..."
LOGIN_RESPONSE=$(curl -s -X POST "http://localhost:3000/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"TestPass123!"}')

echo "Response: $LOGIN_RESPONSE"
echo ""

# Check if we got a token
TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token' 2>/dev/null)

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
  echo "❌ Login failed - no token received"
  echo ""
  echo "Trying direct API endpoint..."
  LOGIN_RESPONSE=$(curl -s -X POST "http://localhost:8080/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username":"testuser","password":"TestPass123!"}')
  echo "Direct API Response: $LOGIN_RESPONSE"
  TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token' 2>/dev/null)
fi

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
  echo "❌ Still no token!"
  exit 1
fi

echo "✓ Got token: ${TOKEN:0:20}..."
echo ""

# Step 2: Try to fetch company data
echo "2. Fetching company data via frontend proxy..."
COMPANY_ID="10000000-0000-0000-0000-000000000001"

COMPANY_RESPONSE=$(curl -s "http://localhost:3000/api/v1/companies/$COMPANY_ID" \
  -H "Authorization: Bearer $TOKEN")

echo "Response: $COMPANY_RESPONSE"
echo ""

# Try direct API
echo "3. Fetching company data via direct API..."
COMPANY_RESPONSE=$(curl -s "http://localhost:8080/api/v1/companies/$COMPANY_ID" \
  -H "Authorization: Bearer $TOKEN")

echo "Response: $COMPANY_RESPONSE"
echo ""

echo "=============================="
echo "Analysis:"
echo ""
echo "• Frontend is running on: http://localhost:3000"
echo "• API is running on: http://localhost:8080"
echo "• Frontend expects API at: /api/v1 (proxied or direct)"
echo ""
echo "Check if Vite proxy is configured in vite.config.ts"
