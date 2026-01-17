#!/bin/bash
# Test script for Login Handler (Step 6.4.1)
# This script tests the login endpoint against the running API server

set -e

echo "================================================="
echo "Testing Login Handler (Step 6.4.1)"
echo "================================================="
echo ""

# Change to project root
cd /home/preetham/Documents/iap-alpha

# Check if server is already running
echo "Checking if API server is running on port 8080..."
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "ERROR: API server is not responding on port 8080"
    echo "Please start the server first (e.g., docker-compose up or cargo run -p api)"
    exit 1
fi

echo "✓ Server is running"
echo ""

# Test 1: Valid login
echo "Test 1: Valid login with correct credentials"
echo "--------------------------------------------"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "TestPass123!"
  }')

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | head -n-1)

echo "HTTP Status: $HTTP_CODE"
echo "Response Body:"
echo "$BODY" | jq '.' 2>/dev/null || echo "$BODY"
echo ""

if [ "$HTTP_CODE" = "200" ]; then
    echo "✓ Test 1 PASSED: Login successful"
    
    # Verify response structure
    ACCESS_TOKEN=$(echo "$BODY" | jq -r '.access_token' 2>/dev/null)
    REFRESH_TOKEN=$(echo "$BODY" | jq -r '.refresh_token' 2>/dev/null)
    TOKEN_TYPE=$(echo "$BODY" | jq -r '.token_type' 2>/dev/null)
    EXPIRES_IN=$(echo "$BODY" | jq -r '.expires_in' 2>/dev/null)
    USER_ID=$(echo "$BODY" | jq -r '.user.id' 2>/dev/null)
    USERNAME=$(echo "$BODY" | jq -r '.user.username' 2>/dev/null)
    
    echo "  - Access Token: ${ACCESS_TOKEN:0:20}..."
    echo "  - Refresh Token: ${REFRESH_TOKEN:0:20}..."
    echo "  - Token Type: $TOKEN_TYPE"
    echo "  - Expires In: $EXPIRES_IN seconds"
    echo "  - User ID: $USER_ID"
    echo "  - Username: $USERNAME"
    
    if [ "$TOKEN_TYPE" != "Bearer" ]; then
        echo "✗ ERROR: token_type should be 'Bearer', got '$TOKEN_TYPE'"
        exit 1
    fi
    
    if [ "$USERNAME" != "testuser" ]; then
        echo "✗ ERROR: username should be 'testuser', got '$USERNAME'"
        exit 1
    fi
else
    echo "✗ Test 1 FAILED: Expected HTTP 200, got $HTTP_CODE"
    exit 1
fi

echo ""

# Test 2: Invalid password
echo "Test 2: Invalid password"
echo "--------------------------------------------"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "WrongPassword"
  }')

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | head -n-1)

echo "HTTP Status: $HTTP_CODE"
echo "Response Body:"
echo "$BODY" | jq '.' 2>/dev/null || echo "$BODY"
echo ""

if [ "$HTTP_CODE" = "401" ]; then
    echo "✓ Test 2 PASSED: Invalid credentials properly rejected"
else
    echo "✗ Test 2 FAILED: Expected HTTP 401, got $HTTP_CODE"
    exit 1
fi

echo ""

# Test 3: Non-existent user
echo "Test 3: Non-existent user"
echo "--------------------------------------------"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "nonexistent",
    "password": "SomePassword"
  }')

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | head -n-1)

echo "HTTP Status: $HTTP_CODE"
echo "Response Body:"
echo "$BODY" | jq '.' 2>/dev/null || echo "$BODY"
echo ""

if [ "$HTTP_CODE" = "401" ]; then
    echo "✓ Test 3 PASSED: Non-existent user properly rejected"
else
    echo "✗ Test 3 FAILED: Expected HTTP 401, got $HTTP_CODE"
    exit 1
fi

echo ""
echo "================================================="
echo "All Tests PASSED ✓"
echo "================================================="
echo ""
echo "Step 6.4.1 (Create Login Handler) is COMPLETE"
echo ""
