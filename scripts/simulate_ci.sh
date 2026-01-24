#!/bin/bash
set -e

echo "=========================================="
echo "      RUNNING LOCAL CI SIMULATION"
echo "=========================================="

echo ""
echo ">>> [1/5] Checking Backend Formatting..."
cd backend
if cargo fmt -- --check; then
    echo "✅ Backend Formatting Passed"
else
    echo "❌ Backend Formatting Failed"
    exit 1
fi

echo ""
echo ">>> [2/5] Checking Backend Lints..."
if cargo clippy -- -D warnings; then
    echo "✅ Backend Lints Passed"
else
    echo "❌ Backend Lints Failed"
    exit 1
fi

echo ""
echo ">>> [3/5] Running Backend Tests..."
if cargo test --workspace -- --test-threads=1; then
    echo "✅ Backend Tests Passed"
else
    echo "❌ Backend Tests Failed"
    exit 1
fi

cd ..

echo ""
echo ">>> [4/5] Checking Frontend Formatting..."
cd frontend
if npx prettier --check .; then
    echo "✅ Frontend Formatting Passed"
else
    echo "❌ Frontend Formatting Failed"
    echo "👉 Run 'npx prettier --write .' in frontend/ to fix."
    exit 1
fi

echo ""
echo ">>> [5/5] Checking Frontend Types & Build..."
if npm run build; then
    echo "✅ Frontend Build Passed"
else
    echo "❌ Frontend Build Failed"
    exit 1
fi

echo ""
echo "=========================================="
echo "🎉 ALL CHECKS PASSED! READY TO PUSH."
echo "=========================================="
