# E2E Tests for Investment Analysis Platform

This directory contains end-to-end tests for the IAP application using Playwright.

## Setup

### 1. Install Dependencies

From the project root:

```bash
npm install
npx playwright install
```

This will install Playwright and the required browser binaries.

### 2. Configure Test Environment

Create a `.env.test` file in the project root (optional):

```bash
# Frontend and API URLs
FRONTEND_URL=http://localhost:3000
API_URL=http://localhost:8080

# Test user credentials
TEST_USER_EMAIL=test@example.com
TEST_USER_PASSWORD=Test123!

# Test company IDs (replace with actual UUIDs from test data)
TEST_AAPL_ID=your-aapl-uuid-here
TEST_NO_VERDICT_ID=your-no-verdict-company-uuid-here
```

### 3. Ensure Test Data Exists

Before running tests, ensure:

1. **Test user exists**: Create a user with the credentials specified in `.env.test` or use the defaults
2. **AAPL company exists**: Have a company (preferably AAPL) seeded in the database
3. **Company without verdict**: Have at least one company that doesn't have a verdict yet

You can seed test data using the backend seed scripts.

## Running Tests

### Prerequisites

Make sure the application is running:

```bash
docker compose up
```

Wait for all services to be healthy (frontend at http://localhost:3000, backend at http://localhost:8080).

### Run All Tests

```bash
npm run test:e2e
```

### Run Tests in UI Mode (Interactive)

```bash
npm run test:e2e:ui
```

### Run Tests in Headed Mode (See Browser)

```bash
npm run test:e2e:headed
```

### Run Specific Test File

```bash
npx playwright test tests/e2e/analyzer.spec.ts
```

### Debug Tests

```bash
npm run test:e2e:debug
```

## Test Structure

```
tests/e2e/
├── config.ts              # Test configuration and constants
├── helpers/
│   └── auth.ts           # Authentication helpers
├── pages/
│   └── AnalyzerPage.ts   # Page Object Model for Analyzer
└── analyzer.spec.ts      # E2E tests for Analyzer module
```

## Writing New Tests

### Using Page Objects

Page objects encapsulate page interactions and locators:

```typescript
import { AnalyzerPage } from './pages/AnalyzerPage';

test('my test', async ({ page }) => {
  const analyzerPage = new AnalyzerPage(page);
  await analyzerPage.goto('company-uuid');
  await analyzerPage.saveVerdict('INVEST', 'Great company!');
});
```

### Authentication

Use the `loginUser` helper to authenticate before tests:

```typescript
import { loginUser } from './helpers/auth';

test.beforeEach(async ({ page }) => {
  await loginUser(page);
});
```

## Test Coverage

The analyzer.spec.ts file covers:

- ✅ Loading company metrics
- ✅ Toggling period type (Quarterly/Annual)
- ✅ Document grid display and availability
- ✅ Saving new verdicts
- ✅ Optimistic lock conflict handling
- ✅ Close without verdict warning
- ✅ Keyboard shortcuts (Ctrl+S)
- ✅ Metrics heat map visualization
- ✅ Period count selector
- ✅ Document upload functionality
- ✅ Strengths/weaknesses editing
- ✅ Refresh button functionality

## Troubleshooting

### Tests Fail with "Connection Refused"

- Ensure Docker Compose is running: `docker compose up`
- Verify services are healthy: `docker compose ps`
- Check that frontend is accessible at http://localhost:3000

### Authentication Fails

- Verify test user exists in the database
- Check credentials in config.ts or .env.test
- Ensure the backend API is running and accessible

### Company UUID Not Found

- Update the test configuration with actual company UUIDs
- Seed test data in the database
- Check the TEST_AAPL_ID and TEST_NO_VERDICT_ID environment variables

### Timeout Errors

- Increase timeout in playwright.config.ts
- Check network performance
- Ensure the application is not under heavy load

## CI/CD Integration

These tests can be run in CI/CD pipelines. Example GitHub Actions workflow:

```yaml
name: E2E Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: npm ci
      - run: npx playwright install --with-deps
      - run: docker compose up -d
      - run: npm run test:e2e
      - uses: actions/upload-artifact@v3
        if: always()
        with:
          name: playwright-report
          path: playwright-report/
```

## Additional Resources

- [Playwright Documentation](https://playwright.dev)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Page Object Model Pattern](https://playwright.dev/docs/pom)
