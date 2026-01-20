# Section 9: Screener Module

**Time Required**: ~2-3 hours  
**Difficulty**: Medium  
**Goal**: Build the complete Screener module (Priority 2) - backend and frontend

---

## Overview

The Screener module allows filtering and discovering companies. It has:
- **Pane 1**: Saved screener list
- **Pane 2**: Screener results table

Reference: PRD sections FR-SCR-001 through FR-SCR-010

---

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/screeners` | GET | List saved screeners |
| `/api/v1/screeners` | POST | Create new screener |
| `/api/v1/screeners/:id` | GET | Get screener details |
| `/api/v1/screeners/:id` | PUT | Update screener |
| `/api/v1/screeners/:id` | DELETE | Delete screener |
| `/api/v1/screeners/:id/run` | POST | Execute screener, return results |

---

## Backend Implementation

### Step 9.1: Screener Repository

---

#### 📋 PROMPT 9.1.1: Create Screener Repository

```
Create the screener repository for database operations.

Create `backend/db/src/repositories/screener_repository.rs`:

1. `ScreenerRepository` struct with PgPool

2. Methods:
   - `list_by_user(&self, user_id: Uuid) -> Result<Vec<Screener>>`
   - `find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Screener>>`
   - `create(&self, user_id: Uuid, screener: CreateScreener) -> Result<Screener>`
   - `update(&self, id: Uuid, user_id: Uuid, screener: UpdateScreener) -> Result<Screener>`
   - `delete(&self, id: Uuid, user_id: Uuid) -> Result<bool>`

3. `CreateScreener` struct:
   - title: String
   - description: Option<String>
   - filter_criteria: serde_json::Value
   - sort_config: Option<serde_json::Value>
   - display_columns: Option<serde_json::Value>

4. All methods should filter by user_id for security
```

**Verification**: Repository compiles.

---

### Step 9.2: Screener Execution Service

---

#### 📋 PROMPT 9.2.1: Create Screener Execution Service

```
Create the screener execution service that filters companies.

Create `backend/core/src/services/screener_service.rs`:

1. `ScreenerService` struct

2. `execute(&self, criteria: FilterCriteria) -> Result<Vec<ScreenerResult>>`:
   - Parse filter criteria JSON
   - Build dynamic SQL query
   - Execute and return results

3. `FilterCriteria` struct (matching PRD FR-SCR-003):
   - exchanges: Option<Vec<String>>
   - sectors: Option<Vec<String>>
   - market_cap_min: Option<f64>
   - market_cap_max: Option<f64>
   - momentum_1m_min: Option<f64>
   - momentum_3m_min: Option<f64>
   - momentum_6m_min: Option<f64>
   - has_verdict: Option<bool>
   - verdict_types: Option<Vec<String>>

4. `ScreenerResult` struct (PRD FR-SCR-004):
   - company_id: Uuid
   - symbol: String
   - company_name: String
   - exchange: String
   - sector: Option<String>
   - market_cap: f64
   - market_cap_formatted: String
   - momentum_1m: Option<f64>
   - momentum_3m: Option<f64>
   - momentum_6m: Option<f64>
   - revenue_yoy_growth: Option<f64>
   - operating_margin: Option<f64>
   - verdict: Option<String>
   - last_analyzed: Option<DateTime<Utc>>
   - guidance_summary: Option<String>

5. Build query dynamically with SQLx:
   - Use query_builder pattern
   - Add WHERE clauses based on criteria
```

**Verification**: Service can filter by market cap and exchange.

---

### Step 9.3: Screener API Routes

---

#### 📋 PROMPT 9.3.1: Create Screener API Endpoints

```
Create all Screener API endpoints.

Create `backend/api/src/routes/screeners.rs`:

1. GET /api/v1/screeners
   - List all screeners for authenticated user
   - Return: Vec<ScreenerSummary>

2. POST /api/v1/screeners
   - Create new screener
   - Body: CreateScreenerRequest
   - Return: Screener

3. GET /api/v1/screeners/:id
   - Get screener with full criteria
   - Return: Screener

4. PUT /api/v1/screeners/:id
   - Update screener
   - Body: UpdateScreenerRequest
   - Return: Screener

5. DELETE /api/v1/screeners/:id
   - Delete screener
   - Return: 204 No Content

6. POST /api/v1/screeners/:id/run
   - Execute screener
   - Optional body: { override_criteria: FilterCriteria }
   - Return: ScreenerResultsResponse

7. `ScreenerResultsResponse` struct:
   - screener_id: Uuid
   - executed_at: DateTime<Utc>
   - total_results: i32
   - results: Vec<ScreenerResult>

All endpoints require authentication.
```

**Verification**: All CRUD operations work.

---

## Frontend Implementation

### Step 9.4: Screener List Component

---

#### 📋 PROMPT 9.4.1: Create Screener List View

```
Create the Screener page with list and results panes.

Create `frontend/src/pages/ScreenerPage.tsx`:

1. Two-pane layout:
   - Left pane (30%): Saved screener list
   - Right pane (70%): Results table

2. Left pane contents:
   - "Create New" button at top
   - List of saved screeners with:
     - Title
     - Description
     - Last run date
     - Actions: Edit, Delete, Run

3. State:
   - selectedScreener: Screener | null
   - results: ScreenerResult[]
   - isRunning: boolean

Create `frontend/src/components/screener/ScreenerList.tsx`:
   - Map over screeners
   - Highlight selected
   - Action buttons

Create `frontend/src/components/screener/ScreenerCard.tsx`:
   - Individual screener card
   - Edit/Delete/Run buttons
```

**Verification**: Screener list displays.

---

### Step 9.5: Screener Editor

---

#### 📋 PROMPT 9.5.1: Create Screener Editor Dialog

```
Create the screener creation/editing dialog.

Create `frontend/src/components/screener/ScreenerEditor.tsx`:

1. Dialog/Modal with form:
   - Title input (required)
   - Description textarea

2. Filter sections per PRD FR-SCR-003:
   **Exchange filter:**
   - Multi-select: NASDAQ, NYSE, BSE, NSE

   **Sector filter:**
   - Multi-select from available sectors

   **Market Cap filter:**
   - Min/Max range inputs
   - Preset buttons: Small Cap, Mid Cap, Large Cap, Mega Cap

   **Momentum filters:**
   - 1M Min %
   - 3M Min %
   - 6M Min %

   **Analysis Status:**
   - Filter by verdict type
   - Filter by "needs analysis" (no verdict or outdated)

3. Save/Cancel buttons

4. Props:
   - mode: 'create' | 'edit'
   - initialData: Screener | null
   - onSave: (screener: CreateScreener) => void
   - onClose: () => void

Use react-hook-form for form management.
```

**Verification**: Can create and edit screeners.

---

### Step 9.6: Results Table [COMPLETED]

---

#### 📋 PROMPT 9.6.1: Create Screener Results Table

```
Create the screener results table component.

Create `frontend/src/components/screener/ResultsTable.tsx`:

1. Use @tanstack/react-table

2. Columns per PRD FR-SCR-004, FR-SCR-005:
   - Symbol (clickable → opens Analyzer)
   - Company Name
   - Exchange
   - Market Cap (formatted: $1.2B)
   - Sector
   - Momentum 1M % (color coded)
   - Momentum 3M % (color coded)
   - Momentum 6M % (color coded)
   - YoY Revenue Growth %
   - Operating Margin %
   - Verdict (badge: INVEST, PASS, etc.)
   - Last Analyzed (relative date)
   - Guidance Summary (truncated)

3. Features:
   - Sortable columns
   - Click row to open Analyzer
   - Column visibility toggle
   - Export to CSV

4. Empty state when no results

5. Loading state with skeleton rows

Create `frontend/src/components/screener/ResultRow.tsx`:
   - Individual result row
   - Color coding for momentum
```

**Verification**: Results table shows data with sorting. [PASSED]

---

### Step 9.7: Git Checkpoint

```bash
# Restart services
docker compose restart api frontend

# Test API
TOKEN=$(curl -s ...)  # Get token

# Create screener
curl -X POST http://localhost:8080/api/v1/screeners \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "Tech Large Cap", "filter_criteria": {"exchanges": ["NASDAQ"], "market_cap_min": 100000000000}}'

# Run screener
curl -X POST http://localhost:8080/api/v1/screeners/{id}/run \
  -H "Authorization: Bearer $TOKEN"

# Visual test in browser
# http://localhost:3000/screener

# Commit
git add .

git commit -m "feat: implement Screener module

Backend:
- CRUD endpoints for screeners
- Screener execution service with dynamic filtering
- Market cap, exchange, sector, momentum filters
- Results include derived metrics

Frontend:
- Two-pane layout (list + results)
- Screener editor dialog
- Results table with sorting
- Click-through to Analyzer
- Column customization"

git push origin develop
```

---

## Verification Checklist

- [ ] Can create, edit, delete screeners
- [ ] Screener execution returns filtered results
- [ ] Results table displays with correct columns
- [ ] Clicking company opens Analyzer
- [ ] Sorting works on results
- [ ] Commit pushed to GitHub

---

## Next Step

**Proceed to**: [10-background-jobs.md](./10-background-jobs.md)
