# Step 8.5 Visual Verification Guide

## Quick Test Instructions

This guide helps you manually verify the Metrics Dashboard component in your browser.

---

## Prerequisites

Ensure all services are running:
```bash
docker compose ps
```

You should see:
- ✅ `frontend` - Up on port 3000
- ✅ `api` - Up on port 8080
- ✅ `postgres` - Up (healthy)

If not, start them:
```bash
docker compose up -d
```

---

## Step-by-Step Visual Verification

### 1. Get Test Company ID

The AAPL company UUID is:
```
10000000-0000-0000-0000-000000000001
```

Other available companies:
- MSFT: `10000000-0000-0000-0000-000000000002`
- JPM: `10000000-0000-0000-0000-000000000003`
- JNJ: `10000000-0000-0000-0000-000000000004`
- TSLA: `10000000-0000-0000-0000-000000000005`

### 2. Login to Application

1. Open your browser and navigate to:
   ```
   http://localhost:3000/login
   ```

2. You should see a login page

3. Enter credentials:
   - **Username:** `testuser`
   - **Password:** `TestPass123!`

4. Click "Login" button

5. You should be redirected to a dashboard or home page

### 3. Navigate to Analyzer Page

In your browser's address bar, navigate to:
```
http://localhost:3000/analyzer/10000000-0000-0000-0000-000000000001
```

This will load the analyzer for Apple (AAPL).

### 4. Verify Page Layout

You should see a 4-pane layout:

```
┌─────────────────────────────────────────────────────┐
│ Pane 0: Controls Bar (top, sticky)                  │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Pane 1: Key Metrics Dashboard (main content)       │
│         ← YOU ARE VERIFYING THIS                    │
│                                                     │
├─────────────────────────────────────────────────────┤
│ Pane 2: Document Grid (placeholder)                │
├─────────────────────────────────────────────────────┤
│ Pane 3: Verdict Recording (placeholder)            │
└─────────────────────────────────────────────────────┘
```

### 5. Verify Controls Bar (Pane 0)

At the top, you should see:
- [ ] Company name and symbol (e.g., "Apple Inc. (AAPL)")
- [ ] Period type toggle (Quarterly / Annual)
- [ ] Period count dropdown (4-10)
- [ ] Refresh button
- [ ] Close button

### 6. Verify Metrics Dashboard (Pane 1)

#### Header Section

Look for:
- [ ] "Key Metrics Dashboard" heading
- [ ] Subtitle showing period type and count (e.g., "Quarterly metrics for 8 periods")

#### Section 1: Growth & Margins

- [ ] Section header "Growth & Margins" is visible
- [ ] Click the header - section should collapse/expand
- [ ] Chevron icon rotates when toggling

When expanded, verify the table shows:
- [ ] First column: Metric names (sticky when scrolling)
  - Revenue ($B)
  - Revenue Growth %
  - Gross Margin %
  - Operating Margin %
  - Net Margin %
  - EPS ($)
  - EPS Growth %
  
- [ ] Period columns (e.g., Q1 2024, Q2 2024, ...)
- [ ] Numeric values in each cell
- [ ] **Heat map colors visible:**
  - Best (highest) values should have a **green** background
  - Worst (lowest) values should have an **orange** background
  - Middle values should show a **gradient** (yellow-ish)

#### Section 2: Cash & Leverage

- [ ] Section header "Cash & Leverage" is visible
- [ ] Section is collapsible

When expanded, verify metrics:
- [ ] Operating Cash Flow ($B)
- [ ] Free Cash Flow ($B)
- [ ] FCF Margin %
- [ ] Total Debt ($B)
- [ ] Net Debt ($B)
- [ ] Debt-to-Equity
- [ ] Current Ratio
- [ ] Heat map colors applied

#### Section 3: Valuation Metrics

- [ ] Section header "Valuation Metrics" is visible
- [ ] Section is collapsible

When expanded, verify metrics:
- [ ] P/E Ratio
- [ ] Price-to-Sales
- [ ] Price-to-Book
- [ ] EV/EBITDA
- [ ] PEG Ratio
- [ ] Dividend Yield %
- [ ] Return on Equity %
- [ ] Return on Invested Capital %

**IMPORTANT:** Valuation metrics use **INVERTED** colors:
- [ ] Lower P/E ratio should be **GREEN** (better)
- [ ] Higher P/E ratio should be **ORANGE** (worse)
- [ ] Same inversion for P/S, P/B, EV/EBITDA, PEG

### 7. Test Interactivity

#### Collapsible Sections
1. Click "Growth & Margins" header
   - [ ] Section collapses (content hidden)
   - [ ] Chevron icon changes from down to right
2. Click header again
   - [ ] Section expands (content visible)
   - [ ] Chevron icon changes from right to down

Repeat for other sections.

#### Period Toggle
1. Click "Annual" toggle in controls bar (if currently on Quarterly)
   - [ ] Metrics should reload
   - [ ] Period columns should update (e.g., to "2020", "2021", etc.)
   - [ ] Heat map colors recalculate

#### Period Count
1. Change period count dropdown (e.g., from 8 to 5)
   - [ ] Number of columns should update
   - [ ] Metrics should reload
   - [ ] Heat map colors recalculate

### 8. Visual Quality Checks

- [ ] Tables are aligned properly
- [ ] Text is readable (good contrast)
- [ ] Colors are distinct enough to differentiate
- [ ] No layout shifts or jumping
- [ ] Scrolling is smooth
- [ ] First column (metric names) stays visible when scrolling horizontally

### 9. Edge Case Checks

Look for cells with "N/A":
- [ ] N/A cells should be **gray** (not colored)
- [ ] N/A cells should show "N/A" text

Hover over value cells:
- [ ] Tooltip appears showing metric name and value
- [ ] Tooltip is readable

### 10. Loading State (Optional)

To see loading skeletons:
1. Open browser DevTools (F12)
2. Go to Network tab
3. Set throttling to "Slow 3G"
4. Refresh the page
5. You should briefly see:
   - [ ] Gray skeleton rows in each section
   - [ ] Skeleton rows match expected metric count

---

## Expected Visual Result

### Color Examples

For the **Growth & Margins** section, Revenue Growth % row:

If the values are: `[5.2%, 8.1%, 3.4%, 7.5%, 6.0%, 9.2%, 4.8%, 7.1%]`

Expected colors (approximate):
- **9.2%** (highest) → Deep **GREEN**
- **8.1%, 7.5%, 7.1%** (high) → Light **GREEN**
- **6.0%, 5.2%** (medium) → **YELLOW**-ish
- **4.8%** (low) → Light **ORANGE**
- **3.4%** (lowest) → Deep **ORANGE**

### Screenshot Reference

Take a screenshot of the full Metrics Dashboard for reference. It should look similar to this description:

```
┌──────────────────────────────────────────────────────────┐
│ Key Metrics Dashboard                                    │
│ Quarterly metrics for 8 periods                          │
├──────────────────────────────────────────────────────────┤
│ ▼ Growth & Margins                                       │
├─────────────┬────────┬────────┬────────┬────────┬────────┤
│ Metric      │ Q1 '24 │ Q2 '24 │ Q3 '24 │ Q4 '24 │ ...    │
├─────────────┼────────┼────────┼────────┼────────┼────────┤
│ Revenue ($B)│  🟢95.3│  🟢97.2│  🟡94.1│  🟢98.5│ ...    │
│ Rev Growth %│  🟢8.1 │  🟢7.5 │  🟠3.4 │  🟢9.2 │ ...    │
│ ...         │        │        │        │        │        │
└─────────────┴────────┴────────┴────────┴────────┴────────┘
```

(🟢 = green background, 🟡 = yellow background, 🟠 = orange background)

---

## Troubleshooting

### Issue: Page shows "Error Loading Data"
**Solution:**
1. Check API is running: `docker compose ps api`
2. Check API logs: `docker compose logs api --tail=50`
3. Ensure company ID is valid
4. Verify you're logged in

### Issue: Metrics show all N/A
**Solution:**
1. Check if AAPL has financial data in database
2. Try a different company
3. Check API response: `curl http://localhost:8080/api/v1/companies/{id}/metrics?period_type=quarterly&period_count=8`

### Issue: No colors visible
**Solution:**
1. Check browser console (F12) for errors
2. Verify heat map CSS is applying (check element styles in DevTools)
3. Try a different metric that has varied values

### Issue: Sections won't collapse
**Solution:**
1. Check browser console for JavaScript errors
2. Verify React is running properly
3. Try refreshing the page

---

## Success Criteria

✅ All three sections visible and collapsible  
✅ Heat map colors clearly visible (green, yellow, orange gradient)  
✅ Valuation metrics use inverted colors  
✅ Period toggle updates data  
✅ All metrics display correctly  
✅ No console errors  
✅ Smooth performance  

---

## Next Steps After Verification

Once you've confirmed all checks pass:

1. Document any issues found
2. Take screenshots for documentation
3. Proceed to Step 8.6 (Document Grid)

---

## Quick Command Reference

```bash
# Get company ID
curl -s http://localhost:8080/api/v1/companies | jq -r '.[] | select(.symbol=="AAPL") | .id'

# Test metrics endpoint directly
curl -s "http://localhost:8080/api/v1/companies/{ID}/metrics?period_type=quarterly&period_count=8" | jq '.'

# Check frontend logs
docker compose logs frontend --tail=50 -f

# Check API logs
docker compose logs api --tail=50 -f

# Restart frontend
docker compose restart frontend

# Rebuild frontend
cd frontend && npm run build
```
