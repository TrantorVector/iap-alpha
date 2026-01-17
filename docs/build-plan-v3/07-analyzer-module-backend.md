# Section 7: Analyzer Module Backend

**Time Required**: ~2-3 hours  
**Difficulty**: Medium-High  
**Goal**: Create all backend APIs for the Analyzer module (Priority 1)

---

## Overview

The Analyzer module is your highest priority. It includes:
- **Pane 1**: Key metrics dashboard (revenue, margins, valuation)
- **Pane 2**: Document repository grid
- **Pane 3**: Verdict recording

This section builds the backend APIs. Section 8 builds the frontend.

---

## API Endpoints for Analyzer

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/companies/:id` | GET | Get company details |
| `/api/v1/companies/:id/metrics` | GET | Get computed metrics for Pane 1 |
| `/api/v1/companies/:id/documents` | GET | Get documents for Pane 2 |
| `/api/v1/companies/:id/documents` | POST | Upload document |
| `/api/v1/companies/:id/documents/:docId/download` | GET | Get presigned download URL |
| `/api/v1/companies/:id/verdict` | GET | Get current verdict |
| `/api/v1/companies/:id/verdict` | PUT | Update verdict (with optimistic locking) |
| `/api/v1/companies/:id/verdict/history` | GET | Get verdict history |

---

## Step-by-Step

### Step 7.1: Company Details API

---

#### 📋 PROMPT 7.1.1: Create Company Details Endpoint

```
Create the company details API endpoint.

Create/update `backend/api/src/routes/companies.rs` with:

1. Route: GET /api/v1/companies/:id

2. `CompanyDetailsResponse` struct:
   - id: Uuid
   - symbol: String
   - name: String
   - exchange: String
   - sector: Option<String>
   - market_cap: Option<f64>
   - market_cap_formatted: String (e.g., "$1.2T", "₹50B")
   - currency: String
   - fiscal_year_end_month: i32
   - is_active: bool
   - last_updated: DateTime<Utc>

3. Handler logic:
   - Extract company_id from path
   - Query company from database
   - Return 404 if not found
   - Format market cap with appropriate unit (K, M, B, T)

4. Add route to the router in mod.rs
```

**Verification**: `curl http://localhost:8080/api/v1/companies/{uuid}` returns company data.

---

### Step 7.2: Metrics API (Pane 1)

---

#### 📋 PROMPT 7.2.1: Create Metrics Calculation Service

```
Create the core metrics calculation service for Pane 1.

Reference PRD sections FR-ANL-011 through FR-ANL-015 for required metrics.

Create `backend/core/src/metrics/calculator.rs` with:

1. `MetricsCalculator` struct

2. Calculate these metrics from raw financial data:
   **Section 1 - Growth & Margins:**
   - Revenue (formatted with units)
   - YoY Growth % (vs same period prior year)
   - QoQ Growth % (vs previous quarter)
   - Revenue Growth Acceleration Delta
   - Gross Margin %
   - Operating Margin %
   - Net Margin %
   - Sequential Gross Margin Expansion %
   - Sequential OP Margin Expansion %

   **Section 2 - Cash & Leverage:**
   - OCF % Revenue
   - FCF % Revenue
   - (Revenue - Net Debt) / Revenue %
   - Total Shares Outstanding

   **Section 3 - Valuation:**
   - OHLC % Revenue (Open, High, Low, Close)
   - P/E Ratio (from daily price and EPS)

3. Helper methods:
   - `format_currency_value(value: f64, currency: &str) -> String`
     - Converts to K/M/B/T units
   - `calculate_yoy_change(current: f64, prior: f64) -> Option<f64>`
   - `calculate_margin(numerator: f64, denominator: f64) -> Option<f64>`
   - `calculate_acceleration_delta(growths: &[f64]) -> Vec<Option<f64>>`

4. Each metric result should include:
   - value: Option<f64>
   - formatted_value: String
   - unit: String (%, $, etc.)
   - heat_map_quartile: Option<i32> (1-4 for coloring)
```

**Verification**: Unit tests for each calculation.

---

#### 📋 PROMPT 7.2.2: Create Metrics API Endpoint

```
Create the metrics API endpoint for the Analyzer Pane 1.

Add to `backend/api/src/routes/companies.rs`:

1. Route: GET /api/v1/companies/:id/metrics

2. Query parameters:
   - period_type: "quarterly" | "annual" (default: quarterly)
   - period_count: 4-10 (default: 8)

3. `MetricsResponse` struct:
   - company_id: Uuid
   - period_type: String
   - periods: Vec<PeriodInfo> (period labels with dates)
   - sections: MetricsSections

4. `MetricsSections` struct:
   - growth_and_margins: Vec<MetricRow>
   - cash_and_leverage: Vec<MetricRow>
   - valuation: Vec<MetricRow>

5. `MetricRow` struct:
   - metric_name: String
   - display_name: String (e.g., "Revenue ($B)")
   - values: Vec<MetricValue> (one per period)
   - heat_map_enabled: bool

6. `MetricValue` struct:
   - period: String
   - value: Option<f64>
   - formatted: String
   - heat_map_quartile: Option<i32>

7. Handler logic:
   - Fetch company
   - Get income statements, balance sheets, cash flows for requested periods
   - Use MetricsCalculator to compute all metrics
   - Apply period window generator for consistent period labels
   - Return formatted response

Reference architecture-design-v3.md section 4.5 for period window generation.
```

**Verification**: Endpoint returns metrics for AAPL (seed data).

---

### Step 7.3: Documents API (Pane 2)

---

#### 📋 PROMPT 7.3.1: Create Documents List Endpoint

```
Create the documents API for Analyzer Pane 2.

Add to `backend/api/src/routes/companies.rs`:

1. Route: GET /api/v1/companies/:id/documents

2. Query parameters:
   - document_type: optional filter (earnings_transcript, annual_report, etc.)

3. `DocumentsResponse` struct (per architecture-design-v3.md section 9.4):
   - documents: Vec<Document>
   - freshness: FreshnessMetadata

4. `Document` struct:
   - id: Uuid
   - document_type: String
   - period_end_date: NaiveDate
   - fiscal_year: i32
   - fiscal_quarter: Option<i32>
   - title: String
   - source_url: Option<String>
   - storage_key: Option<String> (null if not yet downloaded)
   - file_size: Option<i64>
   - mime_type: Option<String>
   - available: bool

5. `FreshnessMetadata` struct:
   - last_refreshed_at: Option<DateTime<Utc>>
   - is_stale: bool (true if >24 hours old)
   - refresh_requested: bool

6. Handler logic:
   - Fetch documents from database grouped by type
   - Check freshness (last refresh timestamp)
   - If stale, enqueue background refresh (don't block)
   - Return immediately with cached data + freshness info

7. Structure response for Pane 2 grid:
   - Group by document_type
   - Sort by period_end_date descending
```

**Verification**: Endpoint returns document list (may be empty initially).

---

#### 📋 PROMPT 7.3.2: Create Document Download Endpoint

```
Create the document download endpoint with presigned S3 URLs.

Add to `backend/api/src/routes/companies.rs`:

1. Route: GET /api/v1/companies/:id/documents/:docId/download

2. Handler logic:
   - Fetch document by ID
   - Verify document belongs to company
   - Verify document has storage_key (is downloaded)
   - Generate presigned S3 URL (15 minute expiry)
   - Return redirect or URL

3. `DownloadResponse` struct:
   - download_url: String
   - expires_in: i64 (seconds)
   - filename: String
   - content_type: String

4. Error cases:
   - 404: Document not found
   - 400: Document not yet available (no storage_key)

Create the S3 presigned URL generation in `backend/providers/src/s3/mod.rs`:
- Use aws-sdk-s3 or rusoto for S3 operations
- Configure with MinIO endpoint for local dev
```

**Verification**: Download URL works for uploaded documents.

---

#### 📋 PROMPT 7.3.3: Create Document Upload Endpoint

```
Create the document upload endpoint for user-uploaded analysis reports.

Add to `backend/api/src/routes/companies.rs`:

1. Route: POST /api/v1/companies/:id/documents

2. Accept multipart form upload:
   - file: The uploaded file
   - document_type: Type of document
   - period_end_date: Optional period association

3. Handler logic:
   - Validate file size (max 50MB)
   - Validate file type (PDF, PPT, PPTX, DOC, DOCX)
   - Generate storage key: "documents/{company_id}/{uuid}/{filename}"
   - Upload to S3
   - Create document record in database
   - Return created document

4. Add multipart form handling to Axum:
   - Use axum-extra or tower-multipart

This endpoint is for user-uploaded documents (analysis reports).
System-fetched documents (transcripts, filings) come from background jobs.
```

**Verification**: Can upload a PDF and retrieve it.

---

### Step 7.4: Verdict API (Pane 3)

---

#### 📋 PROMPT 7.4.1: Create Get Verdict Endpoint

```
Create the verdict retrieval endpoint.

Add to `backend/api/src/routes/companies.rs`:

1. Route: GET /api/v1/companies/:id/verdict

2. `VerdictResponse` struct:
   - verdict_id: Option<Uuid> (null if never analyzed)
   - company_id: Uuid
   - final_verdict: Option<String> (INVEST, PASS, WATCHLIST, NO_THESIS)
   - summary_text: Option<String>
   - strengths: Vec<String>
   - weaknesses: Vec<String>
   - guidance_summary: Option<String>
   - lock_version: i32
   - created_at: Option<DateTime<Utc>>
   - updated_at: Option<DateTime<Utc>>
   - linked_reports: Vec<LinkedReport>

3. `LinkedReport` struct:
   - report_id: Uuid
   - filename: String
   - uploaded_at: DateTime<Utc>

4. Handler logic:
   - Fetch current verdict for company
   - If no verdict exists, return empty response with lock_version: 0
   - Include linked analysis reports
```

**Verification**: Returns empty verdict for new companies.

---

#### 📋 PROMPT 7.4.2: Create Update Verdict Endpoint with Optimistic Locking

```
Create the verdict update endpoint with optimistic locking.

Reference architecture-design-v3.md section 6.4 for the full specification.

Add to `backend/api/src/routes/companies.rs`:

1. Route: PUT /api/v1/companies/:id/verdict

2. `VerdictUpdateRequest` struct:
   - lock_version: i32 (required)
   - final_verdict: Option<String>
   - summary_text: Option<String>
   - strengths: Vec<String>
   - weaknesses: Vec<String>
   - guidance_summary: Option<String>
   - linked_report_ids: Vec<Uuid>

3. Handler logic:
   a. Validate final_verdict is one of: INVEST, PASS, WATCHLIST, NO_THESIS
   b. If verdict doesn't exist, create new one (lock_version should be 0)
   c. If exists, update with optimistic lock check:
      - Use VerdictRepository.update_with_lock()
      - If version mismatch, return 409 Conflict with current state

4. Conflict response (409):
   ```json
   {
     "error": {
       "code": "CONFLICT",
       "details": {
         "message": "Resource modified by another request",
         "current_version": 4,
         "current_state": { ... full current verdict ... }
       }
     }
   }
   ```

5. Success response: Updated VerdictResponse

This is critical for multi-tab editing safety.
```

**Verification**: 
- Update succeeds with correct version
- Update returns 409 with wrong version

---

#### 📋 PROMPT 7.4.3: Create Verdict History Endpoint

```
Create the verdict history endpoint.

Add to `backend/api/src/routes/companies.rs`:

1. Route: GET /api/v1/companies/:id/verdict/history

2. `VerdictHistoryResponse` struct:
   - company_id: Uuid
   - history: Vec<VerdictHistoryEntry>

3. `VerdictHistoryEntry` struct:
   - history_id: Uuid
   - version: i32
   - final_verdict: String
   - summary_text: String
   - recorded_at: DateTime<Utc>
   - linked_report: Option<LinkedReport>

4. Handler logic:
   - Fetch all verdict history for company
   - Include linked analysis reports
   - Order by version descending (most recent first)
   - Limit to 50 entries

Reference architecture-design-v3.md section 6.5 for the verdict-history linkage.
```

**Verification**: History shows after verdict updates.

---

### Step 7.5: Integration Tests

---

#### 📋 PROMPT 7.5.1: Create Analyzer API Integration Tests

```
Create integration tests for all Analyzer API endpoints.

Create `tests/integration/analyzer_test.rs` with:

1. Setup:
   - Create test database
   - Run migrations
   - Seed with AAPL data
   - Create test user and get auth token

2. Company tests:
   - `test_get_company_details_returns_data`
   - `test_get_nonexistent_company_returns_404`

3. Metrics tests:
   - `test_get_metrics_returns_quarterly_data`
   - `test_get_metrics_returns_annual_data`
   - `test_get_metrics_with_period_count`
   - `test_metrics_include_all_sections`

4. Documents tests:
   - `test_list_documents_returns_freshness_metadata`
   - `test_upload_document_creates_record`
   - `test_download_document_returns_presigned_url`

5. Verdict tests:
   - `test_get_verdict_for_unanalyzed_company`
   - `test_create_initial_verdict`
   - `test_update_verdict_succeeds_with_correct_version`
   - `test_update_verdict_fails_with_stale_version`
   - `test_conflict_response_includes_current_state`
   - `test_verdict_history_tracks_changes`

All tests should use authentication.
Clean up test data after each test.
```

**Verification**: `cargo test --test analyzer_test` passes.

---

### Step 7.6: Git Checkpoint

```bash
# Restart API to pick up changes
docker compose restart api
docker compose logs -f api

# Run integration tests
docker compose exec api cargo test --test analyzer_test

# Manual test with curl
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "TestPass123!"}' \
  | jq -r '.access_token')

# Get company (use AAPL UUID from seed data)
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/companies/{aapl-uuid}/metrics

# Commit
git add .

git commit -m "feat(api): implement Analyzer module backend APIs

- GET /companies/:id - Company details
- GET /companies/:id/metrics - Pane 1 metrics with calculations
- GET /companies/:id/documents - Pane 2 documents with freshness
- POST /companies/:id/documents - Document upload
- GET /companies/:id/documents/:id/download - Presigned URLs
- GET /companies/:id/verdict - Current verdict
- PUT /companies/:id/verdict - Update with optimistic locking (409 on conflict)
- GET /companies/:id/verdict/history - Audit trail

Includes metrics calculator for all Pane 1 metrics.
Integration tests for all endpoints."

git push origin develop
```

---

## Verification Checklist

- [ ] All endpoints return expected data
- [ ] Metrics calculation produces correct values
- [ ] Optimistic locking works (409 on version mismatch)
- [ ] Document upload/download works via S3 (MinIO)
- [ ] Integration tests pass
- [ ] Commit pushed to GitHub

---

## Next Step

**Proceed to**: [08-analyzer-module-frontend.md](./08-analyzer-module-frontend.md)
