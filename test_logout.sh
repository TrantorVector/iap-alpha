#!/bin/bash
# Test script for Logout Handler (Step 6.4.3)

set -e

echo "================================================="
echo "Testing Logout Handler (Step 6.4.3)"
echo "================================================="
echo ""

# Check if server is running
echo "Checking if API server is running on port 8080..."
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "ERROR: API server is not responding on port 8080"
    exit 1
fi

echo "✓ Server is running"
echo ""

# Test data
USERNAME="testuser"
PASSWORD="TestPass123!"

# Step 1: Login to get tokens
echo "Step 1: Login"
echo "-------------"
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"$USERNAME\",
    \"password\": \"$PASSWORD\"
  }")

ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token')
REFRESH_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.refresh_token')

if [ "$ACCESS_TOKEN" == "null" ] || [ "$REFRESH_TOKEN" == "null" ]; then
    echo "✗ ERROR: Login failed"
    echo "$LOGIN_RESPONSE"
    exit 1
fi

echo "✓ Login successful"
echo "  Access Token: ${ACCESS_TOKEN:0:20}..."
echo "  Refresh Token: ${REFRESH_TOKEN:0:20}..."
echo ""

# Step 2: Verify Refresh Token works
echo "Step 2: Verify Refresh Token works before logout"
echo "-----------------------------------------------"
REFRESH_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{
    \"refresh_token\": \"$REFRESH_TOKEN\"
  }")

NEW_ACCESS_TOKEN=$(echo "$REFRESH_RESPONSE" | jq -r '.access_token')

if [ "$NEW_ACCESS_TOKEN" == "null" ]; then
    echo "✗ ERROR: Refresh failed before logout"
    echo "$REFRESH_RESPONSE"
    exit 1
fi

echo "✓ Refresh successful"
echo ""

# Step 3: Logout
echo "Step 3: Logout (Revoke all tokens)"
echo "----------------------------------"
LOGOUT_RESPONSE=$(curl -s -w "%{http_code}" -X POST http://localhost:8080/api/v1/auth/logout \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{}")

if [ "$LOGOUT_RESPONSE" == "204" ]; then
    echo "✓ Logout successful (HTTP 204)"
else
    echo "✗ ERROR: Logout failed (HTTP $LOGOUT_RESPONSE)"
    exit 1
fi
echo ""

# Step 4: Verify Refresh Token is now invalid
echo "Step 4: Verify Refresh Token is invalid after logout"
echo "---------------------------------------------------"
REFRESH_RESPONSE_2=$(curl -s -w "\n%{http_code}" -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{
    \"refresh_token\": \"$REFRESH_TOKEN\"
  }")

HTTP_CODE=$(echo "$REFRESH_RESPONSE_2" | tail -n 1)
BODY=$(echo "$REFRESH_RESPONSE_2" | head -n-1)

if [ "$HTTP_CODE" == "401" ]; then
    echo "✓ Test PASSED: Refresh token rejected after logout"
else
    echo "✗ ERROR: Refresh token still valid after logout (HTTP $HTTP_CODE)"
    echo "$BODY"
    exit 1
fi
echo ""

# Step 5: Test Specific Token Logout
echo "Step 5: Test Specific Token Logout"
echo "----------------------------------"
# Login again
LOGIN_RESPONSE_2=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"$USERNAME\",
    \"password\": \"$PASSWORD\"
  }")

AT1=$(echo "$LOGIN_RESPONSE_2" | jq -r '.access_token')
RT1=$(echo "$LOGIN_RESPONSE_2" | jq -r '.refresh_token')

# Login a second time (another session)
LOGIN_RESPONSE_3=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"$USERNAME\",
    \"password\": \"$PASSWORD\"
  }")

AT2=$(echo "$LOGIN_RESPONSE_3" | jq -r '.access_token')
RT2=$(echo "$LOGIN_RESPONSE_3" | jq -r '.refresh_token')

echo "Two sessions created."

# Logout specific session (RT1) using AT2 (should be allowed since it's the same user)
# Actually, logout handler takes the token from body to revoke.
echo "Revoking session 1..."
LOGOUT_SPECIFIC=$(curl -s -w "%{http_code}" -X POST http://localhost:8080/api/v1/auth/logout \
  -H "Authorization: Bearer $AT2" \
  -H "Content-Type: application/json" \
  -d "{ \"refresh_token\": \"$RT1\" }")

if [ "$LOGOUT_SPECIFIC" == "204" ]; then
    echo "✓ Specific logout successful"
else
    echo "✗ ERROR: Specific logout failed (HTTP $LOGOUT_SPECIFIC)"
    exit 1
fi

# Verify RT1 is invalid
echo "Verifying session 1 is revoked..."
R1=$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{ \"refresh_token\": \"$RT1\" }")

if [ "$R1" == "401" ]; then
    echo "✓ Session 1 revoked"
else
    echo "✗ ERROR: Session 1 still valid"
    exit 1
fi

# Verify RT2 is still valid
echo "Verifying session 2 is still valid..."
R2=$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{ \"refresh_token\": \"$RT2\" }")

if [ "$R2" == "200" ]; then
    echo "✓ Session 2 still valid"
else
    echo "✗ ERROR: Session 2 revoked unexpectedly (HTTP $R2)"
    exit 1
fi

echo ""
echo "================================================="
echo "All Logout Tests PASSED ✓"
echo "================================================="
