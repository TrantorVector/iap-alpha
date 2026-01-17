# Section 8: Analyzer Module Frontend

**Time Required**: ~3-4 hours  
**Difficulty**: Medium-High  
**Goal**: Build the complete Analyzer UI with all 4 panes

---

## Overview

The Analyzer is a 4-pane interface:
- **Pane 0**: Controls bar (period toggle, period count, company selector)
- **Pane 1**: Key metrics dashboard (revenue, margins, valuation)
- **Pane 2**: Document repository grid
- **Pane 3**: Verdict recording form

Reference: PRD sections FR-ANL-001 through FR-ANL-035

---

## Step-by-Step

### Step 8.1: Set Up Frontend Foundation

---

#### 📋 PROMPT 8.1.1: Configure React Project with Shadcn/UI

```
Set up the React frontend with Shadcn/UI components.

1. Initialize Shadcn/UI (if not already):
   ```bash
   cd frontend
   npx shadcn-ui@latest init
   ```
   
   Configuration:
   - TypeScript: Yes
   - Style: Default
   - Base color: Slate
   - Global CSS: src/index.css
   - CSS variables: Yes
   - Tailwind.config location: tailwind.config.js
   - Components location: @/components
   - Utils location: @/lib/utils

2. Install required Shadcn components:
   - Button
   - Card
   - Table
   - Select
   - Input
   - Textarea
   - Tabs
   - Dialog
   - Toast
   - Badge
   - Skeleton
   - Tooltip
   - Alert
   - Label

3. Install additional dependencies:
   - @tanstack/react-query (data fetching)
   - @tanstack/react-table (data tables)
   - recharts (charts)
   - react-hook-form (forms)
   - zod (validation)
   - @hookform/resolvers (RHF + Zod)
   - lucide-react (icons)
   - date-fns (date formatting)

4. Set up React Query provider in main.tsx
```

**Verification**: App starts with Shadcn styling applied.

---

### Step 8.2: API Client and Types

---

#### 📋 PROMPT 8.2.1: Create TypeScript API Client

```
Create a typed API client for the frontend.

Create `frontend/src/api/client.ts`:

1. Base configuration:
   - baseUrl: import.meta.env.VITE_API_URL || '/api/v1'
   - Default headers with Content-Type
   - Auth token injection from storage

2. Request interceptor:
   - Add Authorization header if token exists
   - Handle token refresh on 401

3. Error handling:
   - Parse API error responses
   - Create typed error objects

Create `frontend/src/api/types.ts` with all API types:
- LoginRequest, LoginResponse
- CompanyDetails
- MetricsResponse, MetricRow, MetricValue
- DocumentsResponse, Document, FreshnessMetadata
- VerdictResponse, VerdictUpdateRequest
- VerdictHistoryResponse

Create `frontend/src/api/endpoints.ts`:
- auth.login(credentials)
- auth.refresh(refreshToken)
- auth.logout()
- companies.getDetails(companyId)
- companies.getMetrics(companyId, options)
- companies.getDocuments(companyId)
- companies.uploadDocument(companyId, file, metadata)
- companies.getDownloadUrl(companyId, docId)
- verdicts.get(companyId)
- verdicts.update(companyId, update)
- verdicts.getHistory(companyId)

All functions should return typed Promise responses.
```

**Verification**: Types compile without errors.

---

### Step 8.3: Analyzer Layout

---

#### 📋 PROMPT 8.3.1: Create Analyzer Page Layout

```
Create the main Analyzer page with 4-pane layout.

Create `frontend/src/pages/AnalyzerPage.tsx`:

1. Layout structure:
   ```
   ┌─────────────────────────────────────────────────────┐
   │ Pane 0: Controls Bar (sticky)                       │
   ├─────────────────────────────────────────────────────┤
   │                                                     │
   │ Pane 1: Key Metrics Dashboard                       │
   │ (scrollable, ~60% height)                           │
   │                                                     │
   ├─────────────────────────────────────────────────────┤
   │                                                     │
   │ Pane 2: Document Grid                               │
   │ (scrollable, collapsible)                           │
   │                                                     │
   ├─────────────────────────────────────────────────────┤
   │                                                     │
   │ Pane 3: Verdict Recording                           │
   │ (sticky bottom or expandable)                       │
   │                                                     │
   └─────────────────────────────────────────────────────┘
   ```

2. Props:
   - companyId from route params

3. State:
   - periodType: 'quarterly' | 'annual'
   - periodCount: number (4-10)
   - Fetch company details, metrics, documents, verdict

4. Use React Query for data fetching:
   - useQuery for getting data
   - useMutation for updates

5. Handle loading states with Skeleton components

6. Handle error states with Alert components

Add route in router: /analyzer/:companyId
```

**Verification**: Page loads with skeleton states for company UUID.

---

### Step 8.4: Pane 0 - Controls Bar

---

#### 📋 PROMPT 8.4.1: Create Controls Bar Component

```
Create the Pane 0 Controls Bar component.

Create `frontend/src/components/analyzer/ControlsBar.tsx`:

1. Layout: Horizontal bar with:
   - Company name and symbol (left)
   - Period Type toggle: Quarterly / Annual (center)
   - Period Count dropdown: 4, 5, 6, 7, 8, 9, 10 (center)
   - Refresh button (right)
   - Close/back button (right)

2. Props:
   - company: CompanyDetails
   - periodType: string
   - periodCount: number
   - onPeriodTypeChange: (type: string) => void
   - onPeriodCountChange: (count: number) => void
   - onRefresh: () => void
   - onClose: () => void

3. Styling:
   - Sticky position at top
   - Semi-transparent background with blur
   - Compact height

4. Behavior per PRD FR-ANL-002:
   - Hover-visible by default
   - User can pin it to always show (store in preferences)
```

**Verification**: Controls bar renders and toggles work.

---

### Step 8.5: Pane 1 - Key Metrics Dashboard

---

#### 📋 PROMPT 8.5.1: Create Metrics Table Component

```
Create the Pane 1 Key Metrics Dashboard component.

Create `frontend/src/components/analyzer/MetricsDashboard.tsx`:

1. Structure: Three sections as per PRD FR-ANL-012, 013, 014:
   - Section 1: Growth & Margins
   - Section 2: Cash & Leverage  
   - Section 3: Valuation Metrics

2. Each section:
   - Collapsible header with section title
   - Table with rows for each metric
   - Columns for each period

3. Table structure:
   - First column: Metric name (e.g., "Revenue ($B)")
   - Period columns: Values with formatting

4. Heat map coloring (FR-ANL-016):
   - Deep green: Highest value in row
   - Deep orange: Lowest value in row
   - Gradient for intermediate values
   - INVERT for valuation metrics (lower = better)

5. Use @tanstack/react-table for:
   - Column definitions
   - Sorting (optional)
   - Resize support

6. Loading state: Skeleton rows matching expected data

Create `frontend/src/components/analyzer/MetricRow.tsx`:
   - Single metric row component
   - Heat map color calculation
   - Number formatting

Create `frontend/src/lib/heatmap.ts`:
   - calculateHeatMapColor(value, min, max, invert)
   - Returns Tailwind color class or RGB value
```

**Verification**: Metrics table displays AAPL data with colors.

---

### Step 8.6: Pane 2 - Document Grid

---

#### 📋 PROMPT 8.6.1: Create Document Grid Component

```
Create the Pane 2 Document Repository Grid component.

Reference PRD FR-ANL-018 through FR-ANL-025.

Create `frontend/src/components/analyzer/DocumentGrid.tsx`:

1. Grid structure:
   - Rows: Document types (Earnings Transcript, Annual Report, etc.)
   - Columns: Periods (matching Pane 1 periods)
   - Cells: Document status/actions

2. Document types (rows):
   - Earnings Call Transcript
   - Quarterly Report (10-Q)
   - Annual Report (10-K)
   - Investor Presentation
   - Analyst Report (user-uploaded)

3. Cell states:
   - Available: Download icon, clickable
   - Unavailable: Dash or empty
   - Loading: Spinner
   - Error: Error icon with tooltip

4. Row reordering:
   - Drag handle on left of each row
   - Use dnd-kit or similar for drag-drop
   - Save order to user preferences

5. Actions:
   - Click available doc → Open download dialog
   - Upload button for Analyst Reports row

6. Freshness indicator:
   - Show if data is stale
   - "Refreshing..." badge when background refresh in progress

Create `frontend/src/components/analyzer/DocumentCell.tsx`:
   - Individual cell component
   - Click handler
   - Status styling
```

**Verification**: Grid renders with document data.

---

### Step 8.7: Pane 3 - Verdict Recording

---

#### 📋 PROMPT 8.7.1: Create Verdict Form Component

```
Create the Pane 3 Verdict Recording component.

Reference PRD FR-ANL-027 through FR-ANL-031.

Create `frontend/src/components/analyzer/VerdictForm.tsx`:

1. Form fields:
   - Final Verdict: Radio buttons (INVEST, PASS, WATCHLIST, NO_THESIS)
   - Summary Text: Textarea (50-100 words suggested)
   - Strengths: List input (add/remove items)
   - Weaknesses: List input (add/remove items)  
   - Guidance Summary: Textarea (optional)

2. Props:
   - companyId: string
   - initialData: VerdictResponse | null
   - onSaved: () => void

3. Use react-hook-form with zod validation:
   - final_verdict required
   - summary_text max 500 chars
   - strengths/weaknesses as string arrays

4. Submit logic:
   - Include lock_version from initial data
   - Handle 409 Conflict with conflict dialog

5. Conflict handling (architecture-design-v3.md section 6.4):
   - Show dialog with "Your version" vs "Server version"
   - Options: "Keep Mine", "Use Server's", "Merge"
   - On resolve, retry with new lock_version

6. Linked reports:
   - Show uploaded analysis reports
   - File upload zone for new reports

Create `frontend/src/components/analyzer/ConflictDialog.tsx`:
   - Modal showing version comparison
   - Resolution buttons
```

**Verification**: Can save verdict and see it updated.

---

### Step 8.8: Close Window Behavior

---

#### 📋 PROMPT 8.8.1: Implement Close Window Warning

```
Implement the close window behavior per PRD FR-ANL-004.

Update `frontend/src/pages/AnalyzerPage.tsx`:

1. Track if verdict has been recorded:
   - Compare initial verdict with current form state
   - Set "hasChanges" flag

2. Before navigate away:
   - If NO verdict recorded (verdict was null/empty):
     - Show dialog: "No analysis recorded. Close anyway?"
   - If verdict HAS been recorded:
     - Allow close without prompt

3. browser beforeunload event:
   - Prevent accidental tab close with unsaved changes

4. Create ConfirmCloseDialog component:
   - "No analysis recorded. Close anyway?"
   - Buttons: "Cancel", "Close Without Saving"

Use react-router's useBlocker or custom navigation guard.
```

**Verification**: Warning appears when closing without verdict.

---

### Step 8.9: Integration and Polish

---

#### 📋 PROMPT 8.9.1: Connect All Panes and Add Polish

```
Integrate all Analyzer panes and add polish.

1. Data flow:
   - ControlsBar changes update metrics query params
   - Period changes re-fetch metrics AND update document grid columns
   - Document grid rows match user preferences order

2. Loading coordination:
   - Show full-page skeleton initially
   - Show section skeletons for individual refreshes
   - Optimistic updates for verdict saving

3. Error handling:
   - Toast notifications for errors
   - Inline error states where appropriate
   - Retry buttons for failed fetches

4. Responsive design considerations:
   - Desktop-first (per PRD, browser minimum 1280px)
   - Pane resizing: Allow Pane 1 and Pane 2 height adjustment

5. Keyboard shortcuts:
   - Escape to close (with warning if needed)
   - Ctrl+S to save verdict

6. Accessibility:
   - ARIA labels on interactive elements
   - Focus management in dialogs
   - Color contrast for heat map

7. Performance:
   - Memoize heavy components
   - Virtual scrolling if many documents
```

**Verification**: Full workflow works end-to-end.

---

### Step 8.10: E2E Tests

---

#### 📋 PROMPT 8.10.1: Create Analyzer E2E Tests

```
Create Playwright E2E tests for the Analyzer module.

Create `tests/e2e/analyzer.spec.ts`:

1. Setup:
   - Login as test user
   - Navigate to Analyzer with AAPL company

2. Test cases:
   - `test('loads company metrics')`
     - Verify metrics table is visible
     - Check revenue row has values
     
   - `test('toggle period type updates data')`
     - Click Annual toggle
     - Verify columns update
     
   - `test('document grid shows availability')`
     - Verify document grid renders
     - Check cell states
     
   - `test('save new verdict')`
     - Select INVEST
     - Enter summary
     - Click Save
     - Verify success toast
     
   - `test('optimistic lock conflict shows dialog')`
     - Load page in two tabs
     - Save verdict in tab 1
     - Try to save in tab 2
     - Verify conflict dialog appears
     
   - `test('close without verdict shows warning')`
     - Open analyzer for company without verdict
     - Click close
     - Verify warning dialog
     - Click "Close Without Saving"
     - Verify navigated away

Use @playwright/test with page objects for reusability.
```

**Verification**: E2E tests pass with `npx playwright test`.

---

### Step 8.11: Git Checkpoint

```bash
# Restart frontend container
docker compose restart frontend

# Watch logs
docker compose logs -f frontend

# Open browser and test
# http://localhost:3000/analyzer/{aapl-uuid}

# Run E2E tests (if configured)
npx playwright test tests/e2e/analyzer.spec.ts

# Commit
git add .

git commit -m "feat(ui): implement Analyzer module frontend

Pane 0 - Controls Bar:
- Period type toggle (Quarterly/Annual)
- Period count selector (4-10)
- Company info display

Pane 1 - Key Metrics Dashboard:
- Three sections: Growth/Margins, Cash/Leverage, Valuation
- Heat map coloring with value gradient
- All PRD-specified metrics

Pane 2 - Document Grid:
- Document type rows, period columns
- Download/upload functionality
- Freshness indicators
- Row reordering with drag-drop

Pane 3 - Verdict Recording:
- Verdict selection (INVEST/PASS/WATCHLIST/NO_THESIS)
- Strengths/weaknesses lists
- Optimistic locking with conflict dialog
- Analysis report upload

Includes E2E tests for critical flows."

git push origin develop
```

---

## Visual Verification

After completing, manually test:

1. **Open Analyzer**: Navigate to `/analyzer/{company-id}`
2. **Pane 0**: Toggle between Quarterly/Annual, change period count
3. **Pane 1**: Verify metrics load, heat map colors visible
4. **Pane 2**: Click a document cell, verify download dialog
5. **Pane 3**: Record a verdict, verify it saves
6. **Conflict test**: Open in two tabs, edit both, verify conflict handling
7. **Close warning**: Open for new company, try to close without verdict

---

## Verification Checklist

- [ ] App starts and Analyzer page loads
- [ ] Metrics display with heat map colors
- [ ] Period toggle updates data correctly
- [ ] Document grid shows availability status
- [ ] Verdict form saves successfully
- [ ] Conflict dialog appears on version mismatch
- [ ] Close warning appears when no verdict recorded
- [ ] E2E tests pass
- [ ] Commit pushed to GitHub

---

## Next Step

**Proceed to**: [09-screener-module.md](./09-screener-module.md)
