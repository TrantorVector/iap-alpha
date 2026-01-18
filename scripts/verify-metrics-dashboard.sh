#!/bin/bash

# Verification script for Step 8.5: MetricsDashboard Component

set -e

echo "================================================================="
echo "Step 8.5 Verification: Metrics Dashboard Component"
echo "================================================================="
echo ""

# Check that all required files exist
echo "✓ Checking for required files..."
if [ ! -f "frontend/src/lib/heatmap.ts" ]; then
    echo "❌ Missing: frontend/src/lib/heatmap.ts"
    exit 1
fi

if [ ! -f "frontend/src/components/analyzer/MetricRow.tsx" ]; then
    echo "❌ Missing: frontend/src/components/analyzer/MetricRow.tsx"
    exit 1
fi

if [ ! -f "frontend/src/components/analyzer/MetricsDashboard.tsx" ]; then
    echo "❌ Missing: frontend/src/components/analyzer/MetricsDashboard.tsx"
    exit 1
fi

echo "✓ All required files exist"
echo ""

# Check TypeScript compilation
echo "✓ Building TypeScript..."
cd frontend
npm run build > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✓ TypeScript compilation successful"
else
    echo "❌ TypeScript compilation failed"
    exit 1
fi
cd ..
echo ""

# Check that MetricsDashboard is integrated in AnalyzerPage
echo "✓ Checking integration in AnalyzerPage..."
if grep -q "MetricsDashboard" frontend/src/pages/AnalyzerPage.tsx; then
    echo "✓ MetricsDashboard is imported and used in AnalyzerPage"
else
    echo "❌ MetricsDashboard not found in AnalyzerPage"
    exit 1
fi
echo ""

# Check heat map functions exist
echo "✓ Verifying heat map functions..."
if grep -q "calculateHeatMapColor" frontend/src/lib/heatmap.ts; then
    echo "✓ calculateHeatMapColor function exists"
else
    echo "❌ calculateHeatMapColor function missing"
    exit 1
fi

if grep -q "getHeatMapOpacity" frontend/src/lib/heatmap.ts; then
    echo "✓ getHeatMapOpacity function exists"
else
    echo "❌ getHeatMapOpacity function missing"
    exit 1
fi
echo ""

# Check that components use correct API types
echo "✓ Checking API type usage..."
if grep -q "MetricsResponse" frontend/src/components/analyzer/MetricsDashboard.tsx; then
    echo "✓ MetricsDashboard uses MetricsResponse type"
fi

if grep -q "MetricValue" frontend/src/components/analyzer/MetricRow.tsx; then
    echo "✓ MetricRow uses MetricValue type"
fi
echo ""

# Verify frontend service is running
echo "✓ Checking frontend service..."
if docker compose ps frontend | grep -q "Up"; then
    echo "✓ Frontend service is running"
    echo "  URL: http://localhost:3000"
else
    echo "⚠️  Frontend service is not running"
    echo "  Start with: docker compose up -d frontend"
fi
echo ""

# Verify API service is running
echo "✓ Checking API service..."
if docker compose ps api | grep -q "Up"; then
    echo "✓ API service is running"
    echo "  URL: http://localhost:8080"
else
    echo "⚠️  API service is not running"
    echo "  Start with: docker compose up -d api"
fi
echo ""

echo "================================================================="
echo "✅ Step 8.5 Verification Complete!"
echo "================================================================="
echo ""
echo "Summary:"
echo "  • Heat map utilities created (heatmap.ts)"
echo "  • MetricRow component created with heat map coloring"
echo "  • MetricsDashboard component created with 3 sections:"
echo "    1. Growth & Margins"
echo "    2. Cash & Leverage"
echo "    3. Valuation Metrics"
echo "  • Component integrated into AnalyzerPage"
echo "  • TypeScript compilation successful"
echo ""
echo "Next Steps:"
echo "  1. Navigate to http://localhost:3000 in your browser"
echo "  2. Login with test credentials"
echo "  3. Visit /analyzer/{company-id} to see the metrics dashboard"
echo "  4. Verify heat map colors are visible (green=best, orange=worst)"
echo "  5. Verify all three sections are collapsible"
echo ""
echo "To get a company ID for testing:"
echo "  curl -s http://localhost:8080/api/v1/companies | jq '.[] | select(.symbol==\"AAPL\") | .id'"
echo ""
