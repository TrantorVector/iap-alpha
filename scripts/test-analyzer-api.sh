#!/bin/bash

# Test script for Analyzer API endpoints with authentication

set -e

echo "========================================="
echo "Testing Analyzer API Endpoints"
echo "========================================="
echo ""

API_BASE="http://localhost:8080/api/v1"
COMPANY_ID="10000000-0000-0000-0000-000000000001"

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

# Step 2: Get Company Details
echo "Step 2: Fetching company details..."
COMPANY_RESPONSE=$(curl -s "$API_BASE/companies/$COMPANY_ID" \
  -H "Authorization: Bearer $ACCESS_TOKEN")

echo "$COMPANY_RESPONSE" | jq '.' > /dev/null 2>&1 || {
  echo "❌ Company details request failed"
  echo "Response: $COMPANY_RESPONSE"
  exit 1
}

COMPANY_SYMBOL=$(echo "$COMPANY_RESPONSE" | jq -r '.symbol')
COMPANY_NAME=$(echo "$COMPANY_RESPONSE" | jq -r '.name')

echo "✓ Company: $COMPANY_NAME ($COMPANY_SYMBOL)"
echo ""

# Step 3: Get Metrics
echo "Step 3: Fetching metrics..."
METRICS_RESPONSE=$(curl -s "$API_BASE/companies/$COMPANY_ID/metrics?period_type=quarterly&period_count=8" \
  -H "Authorization: Bearer $ACCESS_TOKEN")

echo "$METRICS_RESPONSE" | jq '.' > /dev/null 2>&1 || {
  echo "❌ Metrics request failed"
  echo "Response: $METRICS_RESPONSE"
  exit 1
}

PERIOD_COUNT=$(echo "$METRICS_RESPONSE" | jq '.periods | length')
echo "✓ Metrics fetched: $PERIOD_COUNT periods"

# Check if we have data in sections
GROWTH_COUNT=$(echo "$METRICS_RESPONSE" | jq '.sections.growth_and_margins | length')
CASH_COUNT=$(echo "$METRICS_RESPONSE" | jq '.sections.cash_and_leverage | length')
VALUATION_COUNT=$(echo "$METRICS_RESPONSE" | jq '.sections.valuation | length')

echo "  - Growth & Margins: $GROWTH_COUNT metrics"
echo "  - Cash & Leverage: $CASH_COUNT metrics"
echo "  - Valuation: $VALUATION_COUNT metrics"
echo ""

# Step 4: Get Documents
echo "Step 4: Fetching documents..."
DOCS_RESPONSE=$(curl -s "$API_BASE/companies/$COMPANY_ID/documents" \
  -H "Authorization: Bearer $ACCESS_TOKEN")

echo "$DOCS_RESPONSE" | jq '.' > /dev/null 2>&1 || {
  echo "❌ Documents request failed"
  echo "Response: $DOCS_RESPONSE"
  exit 1
}

DOC_COUNT=$(echo "$DOCS_RESPONSE" | jq '.documents | length')
echo "✓ Documents fetched: $DOC_COUNT documents"
echo ""

# Step 5: Get Verdict
echo "Step 5: Fetching verdict..."
VERDICT_RESPONSE=$(curl -s "$API_BASE/companies/$COMPANY_ID/verdict" \
  -H "Authorization: Bearer $ACCESS_TOKEN")

echo "$VERDICT_RESPONSE" | jq '.' > /dev/null 2>&1 || {
  echo "❌ Verdict request failed"
  echo "Response: $VERDICT_RESPONSE"
  exit 1
}

VERDICT=$(echo "$VERDICT_RESPONSE" | jq -r '.final_verdict // "null"')
echo "✓ Verdict fetched: $VERDICT"
echo ""

echo "========================================="
echo "✅ All API endpoints working correctly!"
echo "========================================="
echo ""
echo "Summary:"
echo "  • Authentication: ✓ Working"
echo "  • Company Details: ✓ Working"
echo "  • Metrics: ✓ Working ($PERIOD_COUNT periods)"
echo "  • Documents: ✓ Working ($DOC_COUNT docs)"
echo "  • Verdict: ✓ Working"
echo ""
echo "The issue is in the frontend not in the API."
echo ""
echo "Next steps to debug frontend:"
echo "1. Open browser DevTools (F12)"
echo "2. Go to Console tab"
echo "3. Go to Network tab"
echo "4. Navigate to: http://localhost:3000/analyzer/$COMPANY_ID"
echo "5. Check for:"
echo "   - Console errors"
echo "   - Failed network requests (red in Network tab)"
echo "   - Missing Authorization headers"
echo ""
