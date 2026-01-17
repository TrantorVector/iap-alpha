# Section 12: Testing & CI/CD

**Time Required**: ~2 hours  
**Difficulty**: Medium  
**Goal**: Expand testing and CI/CD for production deployment

---

## Overview

> [!NOTE]
> **Minimum CI was already set up in Section 2** (format, lint, test). This section expands on that foundation with:
> - E2E testing with Playwright
> - Integration test infrastructure
> - Deployment automation

This section establishes:
- Comprehensive test organization and coverage
- Expanded GitHub Actions workflows
- Deployment pipelines for staging/production
- Automated acceptance gates

Reference: Architecture sections 11, 12, 13

---

## Test Structure

```
tests/
├── unit/              # Rust unit tests (in src files)
├── integration/       # API integration tests
│   ├── auth_test.rs
│   ├── analyzer_test.rs
│   ├── screener_test.rs
│   └── jobs_test.rs
└── e2e/              # Playwright browser tests
    ├── login.spec.ts
    ├── analyzer.spec.ts
    ├── screener.spec.ts
    └── tracker.spec.ts
```

---

## Step-by-Step

### Step 12.1: Test Configuration

---

#### 📋 PROMPT 12.1.1: Configure Test Infrastructure

```
Set up comprehensive test configuration.

1. Backend test configuration:
   Create `backend/tests/common/mod.rs`:
   - Test database setup helper
   - Test app creation helper
   - Auth token generation helper
   - Cleanup utilities

2. Create `.env.test`:
   - DATABASE_URL pointing to test database
   - Test-specific JWT keys (can use generated ones)

3. Update `backend/Cargo.toml`:
   Add test dependencies:
   - assert_eq (assertions)
   - wiremock (HTTP mocking)
   - proptest (optional, property testing)

4. Frontend test configuration:
   Create `frontend/vitest.config.ts`:
   - Configure React Testing Library
   - Setup file for mocks

   Create `frontend/playwright.config.ts`:
   - Base URL: http://localhost:3000
   - Browsers: Chrome, Firefox
   - Screenshot on failure
   - Video recording

5. Create test database setup script:
   `scripts/setup-test-db.sh`:
   - Creates test database
   - Runs migrations
   - Seeds test data
```

**Verification**: Test commands work.

---

### Step 12.2: Unit Tests

---

#### 📋 PROMPT 12.2.1: Add Comprehensive Unit Tests

```
Add unit tests to all core modules.

Target 80% coverage on:

1. `backend/core/src/metrics/calculator.rs`:
   - Test each metric calculation
   - Test edge cases (zeros, negatives, nulls)
   - Test formatting functions

2. `backend/core/src/periods/mod.rs`:
   - Test period generation
   - Test fiscal year alignment
   - Test label formatting

3. `backend/api/src/auth/password.rs`:
   - Test hashing produces valid output
   - Test verification of correct password
   - Test verification of wrong password
   - Test salt is different each time

4. `backend/api/src/auth/jwt.rs`:
   - Test token creation
   - Test token validation
   - Test expiration handling
   - Test invalid signatures
   - Test wrong algorithm rejection (HS256 must fail)

5. Run with: `cargo test --workspace`

Add test documentation explaining what each test verifies.
```

**Verification**: `cargo test` passes with >80% coverage.

---

### Step 12.3: Integration Tests

---

#### 📋 PROMPT 12.3.1: Complete Integration Test Suite

```
Complete the integration test suite for all API endpoints.

Create/update test files:

1. `tests/integration/mod.rs` - Common setup

2. `tests/integration/auth_test.rs`:
   - Login success/failure
   - Token refresh
   - Logout
   - Protected route access

3. `tests/integration/company_test.rs`:
   - Get company details
   - Get metrics
   - Get documents

4. `tests/integration/screener_test.rs`:
   - CRUD operations
   - Screener execution
   - Filter validation

5. `tests/integration/verdict_test.rs`:
   - Create verdict
   - Update with locking
   - Conflict handling
   - History tracking

Test execution:
- Start test database before tests
- Use transactions for isolation
- Rollback after each test
```

**Verification**: `cargo test --test '*'` passes.

---

### Step 12.4: E2E Tests

---

#### 📋 PROMPT 12.4.1: Create E2E Test Suite

```
Create Playwright E2E test suite for critical user flows.

Install Playwright:
```bash
cd frontend
npm install -D @playwright/test
npx playwright install
```

Create `tests/e2e/`:

1. `login.spec.ts`:
   - Successful login flow
   - Invalid credentials error
   - Redirect to originally requested page

2. `analyzer.spec.ts`:
   - Load metrics for company
   - Toggle period type
   - Save verdict
   - Conflict dialog flow

3. `screener.spec.ts`:
   - Create screener
   - Run screener
   - Filter results
   - Open company from results

4. `navigation.spec.ts`:
   - Navigate between modules
   - Breadcrumb functionality
   - Close warning on unsaved changes

Each test should:
- Use page objects for reusability
- Take screenshots on failure
- Run in headless mode for CI
```

**Verification**: `npx playwright test` passes.

---

### Step 12.5: Expand GitHub Actions CI Pipeline

---

#### 📋 PROMPT 12.5.1: Expand CI Workflow for Deployment

```
Expand the GitHub Actions CI/CD pipeline for deployment.

Update `.github/workflows/ci.yml` to add deployment stages:

```yaml
name: CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  # ==== EXISTING JOBS (from Section 2) ====
  check-backend:
    # ... (keep existing)
  
  check-frontend:
    # ... (keep existing)

  # ==== NEW JOBS ====
  
  integration-tests:
    runs-on: ubuntu-latest
    needs: [check-backend, check-frontend]
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_DB: test
          POSTGRES_PASSWORD: test
        ports:
          - 5432:5432
        options: --health-cmd pg_isready --health-interval 10s
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      - name: Run Integration Tests
        run: cd backend && cargo test --test '*'
        env:
          DATABASE_URL: postgres://postgres:test@localhost/test

  e2e-tests:
    runs-on: ubuntu-latest
    needs: [check-backend, check-frontend]
    steps:
      - uses: actions/checkout@v4
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      - name: Install Playwright
        run: cd frontend && npm ci && npx playwright install --with-deps
      - name: Start services
        run: docker compose up -d
      - name: Wait for services
        run: sleep 30
      - name: Run E2E tests
        run: cd frontend && npx playwright test
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: playwright-report
          path: frontend/playwright-report/

  build-and-push:
    runs-on: ubuntu-latest
    needs: [integration-tests, e2e-tests]
    if: github.ref == 'refs/heads/main' || github.ref == 'refs/heads/develop'
    steps:
      - uses: actions/checkout@v4
      - name: Configure AWS
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ap-south-1
      - name: Login to ECR
        uses: aws-actions/amazon-ecr-login@v2
      - name: Build and push API image
        run: |
          docker build -t $ECR_REGISTRY/iap-api:$GITHUB_SHA -f backend/Dockerfile .
          docker push $ECR_REGISTRY/iap-api:$GITHUB_SHA
      - name: Build and push Frontend image
        run: |
          docker build -t $ECR_REGISTRY/iap-frontend:$GITHUB_SHA -f frontend/Dockerfile .
          docker push $ECR_REGISTRY/iap-frontend:$GITHUB_SHA

  deploy-staging:
    runs-on: ubuntu-latest
    needs: build-and-push
    if: github.ref == 'refs/heads/develop'
    environment: staging
    steps:
      - uses: actions/checkout@v4
      - name: Deploy to staging
        run: |
          cd infra
          npm ci
          npx pulumi up --stack staging --yes
```

This workflow:
- Runs on every push to main/develop
- Integration tests after lint checks pass
- E2E tests in parallel with integration tests
- Build Docker images after all tests pass
- Deploy to staging automatically from develop branch
- Deploy to production manually from main branch (add approval)
```

**Verification**: Push to GitHub triggers full CI pipeline.

---

### Step 12.6: Acceptance Gates

---

#### 📋 PROMPT 12.6.1: Configure Branch Protection

```
Document branch protection rules for GitHub.

Create `.github/BRANCH_PROTECTION.md`:

Recommended settings for `main` branch:
1. Require pull request before merging
2. Require status checks to pass:
   - check-backend
   - check-frontend
   - integration-tests
   - e2e-tests
   - build-and-push
3. Require branches to be up to date
4. Do not allow bypassing

For `develop` branch:
1. Require status checks to pass:
   - check-backend
   - check-frontend
2. Allow direct pushes (for now)

Note: Set these manually in GitHub → Settings → Branches

Since this is AI-built, green CI = automatically mergeable.
```

**Verification**: Branch protection is documented.

---

### Step 12.7: Git Checkpoint

```bash
# Run full test suite
cargo test --workspace
cd frontend && npm test
npx playwright test

# If all pass, commit
git add .

git commit -m "feat(ci): expanded testing and deployment pipeline

Testing:
- Unit tests for core calculations (80%+ coverage)
- Integration tests for all API endpoints
- E2E tests with Playwright for critical flows

CI/CD:
- Expanded GitHub Actions workflow
- Integration and E2E tests in pipeline
- Docker image builds on merge
- Staging deployment from develop branch
- Branch protection documentation

All tests passing gates commits for merge."

git push origin develop
```

---

## Verification Checklist

- [ ] `cargo test --workspace` passes
- [ ] Frontend tests pass
- [ ] Playwright E2E tests pass
- [ ] GitHub Actions workflow runs successfully
- [ ] Docker images build in CI
- [ ] Commit pushed to GitHub

---

## Running Tests Locally

```bash
# Backend unit + integration tests
cargo test --workspace

# Backend with verbose output
cargo test --workspace -- --nocapture

# Specific test
cargo test test_login_success

# Frontend unit tests
cd frontend && npm test

# E2E tests (requires app running)
docker compose up -d
npx playwright test

# E2E with UI mode (debugging)
npx playwright test --ui
```

---

## Next Step

**Proceed to**: [13-cloud-deployment.md](./13-cloud-deployment.md)
