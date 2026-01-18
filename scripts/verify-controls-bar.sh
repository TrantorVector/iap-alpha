#!/bin/bash

# Step 8.4 - Controls Bar Verification Script
# This script verifies that the ControlsBar component is accessible in the browser

set -e

echo "================================================"
echo "Step 8.4: Controls Bar Verification"
echo "================================================"
echo ""

# Check if services are running
echo "1. Checking if services are running..."
if ! docker compose ps | grep -q "frontend.*Up"; then
    echo "   ❌ Frontend service is not running!"
    echo "   Starting services..."
    docker compose up -d
    echo "   ⏳ Waiting for services to be ready (15s)..."
    sleep 15
else
    echo "   ✅ Frontend service is running"
fi

# Check if API is accessible
echo ""
echo "2. Checking if API is accessible..."
API_HEALTH=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health || echo "000")
if [ "$API_HEALTH" = "200" ]; then
    echo "   ✅ API is healthy (HTTP 200)"
else
    echo "   ℹ️  API returned: $API_HEALTH (may be expected if no /health endpoint)"
fi

# Check if frontend is accessible
echo ""
echo "3. Checking if frontend is accessible..."
FRONTEND_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000 || echo "000")
if [ "$FRONTEND_STATUS" = "200" ]; then
    echo "   ✅ Frontend is accessible (HTTP 200)"
else
    echo "   ❌ Frontend returned: $FRONTEND_STATUS"
    exit 1
fi

# Get a company ID from database
echo ""
echo "4. Getting test company ID from database..."
COMPANY_ID=$(docker compose exec -T postgres psql -U postgres -d irp_dev -t -c "SELECT id FROM companies LIMIT 1;" | xargs)
if [ -n "$COMPANY_ID" ]; then
    COMPANY_SYMBOL=$(docker compose exec -T postgres psql -U postgres -d irp_dev -t -c "SELECT symbol FROM companies WHERE id = '$COMPANY_ID';" | xargs)
    echo "   ✅ Found company: $COMPANY_SYMBOL (ID: $COMPANY_ID)"
else
    echo "   ❌ No companies found in database!"
    exit 1
fi

# Check if the component files exist
echo ""
echo "5. Verifying component files exist..."
if [ -f "frontend/src/components/analyzer/ControlsBar.tsx" ]; then
    echo "   ✅ ControlsBar.tsx exists"
    LINES=$(wc -l < frontend/src/components/analyzer/ControlsBar.tsx)
    echo "      📝 File has $LINES lines"
else
    echo "   ❌ ControlsBar.tsx NOT found!"
    exit 1
fi

if [ -f "frontend/src/pages/AnalyzerPage.tsx" ]; then
    echo "   ✅ AnalyzerPage.tsx exists"
    # Check if ControlsBar is imported
    if grep -q "import.*ControlsBar" frontend/src/pages/AnalyzerPage.tsx; then
        echo "      ✅ ControlsBar is imported in AnalyzerPage"
    else
        echo "      ❌ ControlsBar NOT imported in AnalyzerPage!"
        exit 1
    fi
else
    echo "   ❌ AnalyzerPage.tsx NOT found!"
    exit 1
fi

# Check TypeScript compilation
echo ""
echo "6. Checking TypeScript compilation..."
cd frontend
if npm run build > /tmp/build-output.log 2>&1; then
    echo "   ✅ TypeScript compiles without errors"
else
    echo "   ❌ TypeScript compilation failed!"
    echo "   See /tmp/build-output.log for details"
    exit 1
fi
cd ..

# Check for required UI components
echo ""
echo "7. Verifying Shadcn UI components..."
REQUIRED_COMPONENTS=("button" "select" "tabs")
for component in "${REQUIRED_COMPONENTS[@]}"; do
    if [ -d "frontend/src/components/ui/$component" ] || [ -f "frontend/src/components/ui/$component.tsx" ]; then
        echo "   ✅ Component '$component' exists"
    else
        echo "   ⚠️  Component '$component' may be missing"
    fi
done

# Summary
echo ""
echo "================================================"
echo "✅ VERIFICATION COMPLETE"
echo "================================================"
echo ""
echo "📊 Summary:"
echo "   • ControlsBar component: ✅ Implemented"
echo "   • Integration with AnalyzerPage: ✅ Complete"
echo "   • TypeScript compilation: ✅ Passing"
echo "   • Services: ✅ Running"
echo ""
echo "🌐 Manual Testing:"
echo "   Open in browser: http://localhost:3000/analyzer/$COMPANY_ID"
echo "   Test company: $COMPANY_SYMBOL"
echo ""
echo "📋 Test the following in browser:"
echo "   1. Controls bar renders at top with all elements"
echo "   2. Hover behavior (auto-hide when unpinned)"
echo "   3. Pin/unpin functionality"
echo "   4. Period type toggle (Quarterly/Annual)"
echo "   5. Period count dropdown (4-10)"
echo "   6. Refresh button functionality"
echo "   7. Close button navigation"
echo ""
echo "================================================"
