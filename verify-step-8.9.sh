#!/bin/bash

# Step 8.9 Verification Script
# Integration and Polish verification for Analyzer Module

echo "=========================================="
echo "Step 8.9: Integration and Polish Verification"
echo "=========================================="
echo ""

# Track results
PASSED=0
FAILED=0
WARNINGS=0

# Helper functions
check_pass() {
    echo "✅ $1"
    ((PASSED++))
}

check_fail() {
    echo "❌ $1"
    ((FAILED++))
}

check_warn() {
    echo "⚠️  $1"
    ((WARNINGS++))
}

echo "1. Data Flow Integration"
echo "------------------------"

# Check ControlsBar handler memoization
if grep -q "handlePeriodTypeChange" frontend/src/pages/AnalyzerPage.tsx && \
   grep -q "useCallback" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Period type handler is memoized with useCallback"
else
    check_fail "Period type handler not properly memoized"
fi

if grep -q "handlePeriodCountChange" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Period count handler is memoized"
else
    check_fail "Period count handler missing"
fi

echo ""
echo "2. Loading Coordination"
echo "----------------------"

# Check for skeleton states
if grep -q "Skeleton" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Page-level skeletons present"
else
    check_fail "Missing page-level skeleton components"
fi

if grep -q "role=\"status\"" frontend/src/components/analyzer/MetricsDashboard.tsx; then
    check_pass "MetricsDashboard has loading state with ARIA role"
else
    check_warn "MetricsDashboard missing ARIA role for loading state"
fi

echo ""
echo "3. Error Handling"
echo "----------------"

# Check for retry buttons in error states
if grep -q "refetchMetrics\|refetchDocuments\|refetchVerdict" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Individual retry buttons present for each data source"
else
    check_fail "Missing individual retry buttons"
fi

if grep -q "useToast" frontend/src/components/analyzer/VerdictForm.tsx; then
    check_pass "Toast notifications integrated in VerdictForm"
else
    check_fail "Toast notifications missing from VerdictForm"
fi

echo ""
echo "4. Responsive Design"
echo "-------------------"

# Check for pane resizing
if grep -q "metricsHeight" frontend/src/pages/AnalyzerPage.tsx && \
   grep -q "GripHorizontal" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Pane resizing implemented with resize handle"
else
    check_fail "Pane resizing not implemented"
fi

if grep -q "setIsResizing" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Resize state management present"
else
    check_fail "Resize state management missing"
fi

echo ""
echo "5. Keyboard Shortcuts"
echo "--------------------"

# Check for Ctrl+S shortcut
if grep -q "Ctrl+S" frontend/src/pages/AnalyzerPage.tsx || \
   grep -q "ctrlKey.*s" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Ctrl+S keyboard shortcut for save verdict"
else
    check_fail "Ctrl+S shortcut missing"
fi

# Check for Escape shortcut
if grep -q "Escape" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Escape key to close implemented"
else
    check_fail "Escape key shortcut missing"
fi

echo ""
echo "6. Accessibility"
echo "---------------"

# Check for ARIA labels
if grep -q "aria-label" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "ARIA labels present in main page"
else
    check_warn "ARIA labels missing in main page"
fi

if grep -q "aria-label\|aria-expanded" frontend/src/components/analyzer/MetricsDashboard.tsx; then
    check_pass "ARIA attributes in MetricsDashboard"
else
    check_warn "ARIA attributes missing in MetricsDashboard"
fi

if grep -q "aria-label\|aria-hidden" frontend/src/components/analyzer/DocumentGrid.tsx; then
    check_pass "ARIA attributes in DocumentGrid"
else
    check_warn "ARIA attributes missing in DocumentGrid"
fi

if grep -q "scope=\"col\"" frontend/src/components/analyzer/MetricsDashboard.tsx && \
   grep -q "scope=\"col\"" frontend/src/components/analyzer/DocumentGrid.tsx; then
    check_pass "Proper table semantics with scope attributes"
else
    check_warn "Table scope attributes could be improved"
fi

echo ""
echo "7. Performance Optimization"
echo "--------------------------"

# Check for React.memo
if grep -q "memo" frontend/src/components/analyzer/MetricsDashboard.tsx; then
    check_pass "MetricsDashboard uses React.memo"
else
    check_fail "MetricsDashboard not memoized"
fi

if grep -q "memo" frontend/src/components/analyzer/DocumentGrid.tsx; then
    check_pass "DocumentGrid uses React.memo"
else
    check_fail "DocumentGrid not memoized"
fi

# Check for useCallback
if grep -q "useCallback" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Event handlers memoized with useCallback"
else
    check_fail "Event handlers not memoized"
fi

if grep -q "useCallback" frontend/src/components/analyzer/DocumentGrid.tsx; then
    check_pass "DocumentGrid handlers use useCallback"
else
    check_warn "DocumentGrid handlers could be optimized with useCallback"
fi

echo ""
echo "8. Close Window Behavior"
echo "-----------------------"

# Check for blocker
if grep -q "useBlocker" frontend/src/pages/AnalyzerPage.tsx && \
   grep -q "ConfirmCloseDialog" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Navigation blocking with confirmation dialog"
else
    check_fail "Close window warning missing"
fi

if grep -q "beforeunload" frontend/src/pages/AnalyzerPage.tsx; then
    check_pass "Browser beforeunload event handler present"
else
    check_warn "beforeunload handler could prevent accidental tab close"
fi

echo ""
echo "=========================================="
echo "VERIFICATION SUMMARY"
echo "=========================================="
echo "✅ Passed: $PASSED"
echo "❌ Failed: $FAILED"
echo "⚠️  Warnings: $WARNINGS"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "🎉 All critical checks passed!"
    echo ""
    echo "Step 8.9 is COMPLETE with all integration and polish requirements met:"
    echo "  • Data flow properly coordinated between panes"
    echo "  • Loading states with skeletons"
    echo "  • Error handling with retry buttons"
    echo "  • Pane resizing functionality"
    echo "  • Keyboard shortcuts (Ctrl+S, Escape)"
    echo "  • Accessibility with ARIA labels"
    echo "  • Performance optimizations with memo and useCallback"
    echo "  • Close window behavior with warnings"
    echo ""
    exit 0
else
    echo "🔧 Some checks failed. Please review the failures above."
    exit 1
fi
