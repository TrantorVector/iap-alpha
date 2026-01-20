# Step 8.9 - Visual Verification Guide

**Guide Version**: 1.0  
**Date**: 2026-01-19  
**Purpose**: Manual testing checklist for Step 8.9 integration and polish features

---

## Prerequisites

Before starting visual verification, ensure:
- [ ] Docker is installed and running
- [ ] All code changes are in place
- [ ] Database migrations are up to date

---

## Part 1: Start the Application

### 1.1 Start Docker Services

```bash
# From project root
cd /home/founder/iap-alpha

# Start all services
docker compose up -d

# Check services are running
docker compose ps
```

**Expected Output:**
- ✅ `postgres` - running
- ✅ `backend` - running  
- ✅ `frontend` - running

### 1.2 Wait for Services to Initialize

```bash
# Watch backend logs
docker compose logs -f backend

# Wait for: "Application startup complete" or similar
# Ctrl+C to exit logs
```

### 1.3 Access the Application

1. Open your browser
2. Navigate to: `http://localhost:3000`
3. You should see the IAP Alpha landing page

---

## Part 2: Login and Navigate to Analyzer

### 2.1 Auto-Login to AAPL Analyzer

On the home page:
1. Click **"Open AAPL Analyzer (Auto-Login)"** button
2. Wait for login to complete
3. You should be redirected to: `/analyzer/10000000-0000-0000-0000-000000000001`

**What to Verify:**
- [ ] Login succeeds without errors
- [ ] Browser navigates to analyzer page
- [ ] No console errors (open DevTools: F12)

---

## Part 3: Data Flow Integration Testing

### 3.1 Test Period Type Toggle

**Action:**
1. Locate the **Controls Bar** at the top (Pane 0)
2. Find the **Period Type** toggle (Quarterly / Annual)
3. Click **"Annual"**

**Expected Behavior:**
- [ ] Button visually changes to show "Annual" is selected
- [ ] Metrics Dashboard (Pane 1) columns update from "Q1-2023" format to "2023" format
- [ ] Document Grid (Pane 2) period columns match the new format
- [ ] Loading skeletons briefly appear during data fetch
- [ ] Data loads successfully with annual periods

**Action:**
4. Click **"Quarterly"** to switch back

**Expected Behavior:**
- [ ] Columns revert to quarterly format (Q1-2023, Q2-2023, etc.)
- [ ] All data reloads correctly

### 3.2 Test Period Count Selector

**Action:**
1. In Controls Bar, find the **Period Count** dropdown
2. Current value should be **8**
3. Click the dropdown and select **10**

**Expected Behavior:**
- [ ] Dropdown opens smoothly
- [ ] Selecting 10 triggers a refetch
- [ ] Metrics Dashboard now shows 10 period columns
- [ ] Document Grid now shows 10 period columns
- [ ] All data aligned correctly across both panes

**Action:**
4. Change back to **8** periods

### 3.3 Test Refresh Button

**Action:**
1. Click the **Refresh** button in Controls Bar (circular arrow icon)

**Expected Behavior:**
- [ ] All panes show brief loading skeletons
- [ ] Data reloads from API
- [ ] No errors in console
- [ ] Company data, metrics, documents, and verdict all refresh

---

## Part 4: Loading Coordination Testing

### 4.1 Initial Page Load

**Action:**
1. Hard refresh the page (Ctrl+Shift+R or Cmd+Shift+R)
2. Watch the loading sequence

**Expected Behavior:**
- [ ] Full-page skeleton appears immediately
- [ ] Controls Bar shows company name placeholder
- [ ] Metrics Dashboard shows skeleton rows
- [ ] Document Grid shows loading skeleton
- [ ] Verdict Form shows skeleton fields
- [ ] Skeletons are replaced smoothly with actual data
- [ ] No layout shift or jumping

### 4.2 Section-Specific Loading

This was tested in 3.1-3.3, but verify:
- [ ] Only the relevant section shows loading state during refresh
- [ ] Other sections remain interactive
- [ ] Loading indicators are clear (skeletons or spinners)

---

## Part 5: Error Handling Testing

### 5.1 Simulate Network Error

**Action:**
1. Open DevTools (F12)
2. Go to **Network** tab
3. Find the throttling dropdown (usually says "No throttling")
4. Select **"Offline"**
5. Click the Refresh button in Controls Bar

**Expected Behavior:**
- [ ] Error alert appears in center of page
- [ ] Alert shows "Error Loading Data"
- [ ] Specific errors listed (Metrics, Documents, Verdict, Company)
- [ ] Individual **Retry** buttons appear for each failed source
- [ ] **Retry All** button is present
- [ ] Error has red destructive styling
- [ ] AlertCircle icon is visible

### 5.2 Test Individual Retry Buttons

**Action (while still offline):**
1. Click **"Retry Metrics"** button

**Expected Behavior:**
- [ ] Only metrics retry attempt happens
- [ ] Button shows loading state briefly
- [ ] Error persists (still offline)

**Action:**
2. Switch Network tab back to **"No throttling"**
3. Click **"Retry Metrics"** again

**Expected Behavior:**
- [ ] Metrics load successfully
- [ ] Metrics error disappears from error list
- [ ] Other errors still shown
- [ ] Can retry each source individually

**Action:**
4. Click **"Retry All"**

**Expected Behavior:**
- [ ] All remaining data sources retry
- [ ] Error alert disappears once all data loads
- [ ] Page returns to normal state

---

## Part 6: Responsive Design Testing

### 6.1 Pane Resizing

**Action:**
1. Scroll to find the **resize handle** between Pane 1 (Metrics) and Pane 2 (Documents)
2. Look for a horizontal grip icon with hover effect
3. Hover over the resize handle

**Expected Behavior:**
- [ ] Handle becomes visible on hover (semi-transparent)
- [ ] Cursor changes to `row-resize` cursor
- [ ] Handle shows grip icon (GripHorizontal)

**Action:**
4. Click and drag the handle **upward**

**Expected Behavior:**
- [ ] Metrics Dashboard height decreases smoothly
- [ ] Document Grid section expands to fill space
- [ ] No jumping or glitches during drag
- [ ] Minimum height enforced (300px)
- [ ] Cursor remains `row-resize` during drag

**Action:**
5. Drag the handle **downward** (try to make Metrics very tall)

**Expected Behavior:**
- [ ] Metrics Dashboard height increases
- [ ] Document Grid shrinks
- [ ] Maximum height enforced (1000px)
- [ ] Transition is smooth

**Action:**
6. Release the mouse button

**Expected Behavior:**
- [ ] Cursor returns to default
- [ ] Pane heights remain at new size
- [ ] Both panes still scrollable if content overflows

### 6.2 Desktop-First Design

**Action:**
1. Open DevTools responsive mode (Ctrl+Shift+M)
2. Set viewport to **1280px width** (minimum)
3. Try **1920px width** (standard desktop)

**Expected Behavior:**
- [ ] Layout works well at 1280px minimum
- [ ] No horizontal scrollbar at 1280px
- [ ] Layout scales nicely to 1920px
- [ ] All controls accessible at both sizes
- [ ] Text readable at both sizes

---

## Part 7: Keyboard Shortcuts Testing

### 7.1 Test Ctrl+S (Save Verdict)

**Action:**
1. Scroll to Pane 3 (Verdict Recording)
2. Make a change in the verdict form (e.g., select "INVEST")
3. Press **Ctrl+S** (Windows/Linux) or **Cmd+S** (Mac)

**Expected Behavior:**
- [ ] Browser's native save dialog does NOT appear (prevented)
- [ ] Verdict form submits
- [ ] Toast notification appears: "Verdict saved successfully" (or similar)
- [ ] Console shows no errors
- [ ] Verdict updates in database

### 7.2 Test Escape (Close with Warning)

**Action:**
1. Ensure verdict form has unsaved changes OR no verdict recorded
2. Press **Escape** key

**Expected Behavior:**
- [ ] Navigation is blocked
- [ ] **ConfirmCloseDialog** appears
- [ ] Dialog says "No analysis recorded. Close anyway?"
- [ ] Two buttons: "Cancel" and "Close Without Saving"

**Action:**
3. Click **"Cancel"**

**Expected Behavior:**
- [ ] Dialog closes
- [ ] Stays on analyzer page
- [ ] Data preserved

**Action:**
4. Press **Escape** again
5. Click **"Close Without Saving"**

**Expected Behavior:**
- [ ] Navigation proceeds
- [ ] Returns to home page
- [ ] No errors

### 7.3 Test Escape (Close without Warning)

**Action:**
1. Navigate back to analyzer
2. Ensure verdict is already saved and no changes made
3. Press **Escape**

**Expected Behavior:**
- [ ] No dialog appears
- [ ] Immediately navigates to home page
- [ ] Clean exit

---

## Part 8: Accessibility Testing

### 8.1 ARIA Labels Visual Verification

**Action:**
1. Open DevTools Elements/Inspector tab
2. Inspect the main analyzer container (`<div>` with analyzer content)

**Verify These Attributes:**
- [ ] `role="main"`
- [ ] `aria-label="Analyzer Module"`

**Action:**
3. Inspect the error alert (trigger an error first if needed)

**Verify:**
- [ ] `role="alert"`
- [ ] `aria-live="assertive"`

**Action:**
4. Inspect the retry buttons

**Verify:**
- [ ] Each has `aria-label="Retry loading [source]"`

### 8.2 Table Semantics

**Action:**
1. Inspect the Metrics Dashboard table
2. Look at the `<table>` element

**Verify:**
- [ ] `role="table"`
- [ ] `aria-label` describing the table

**Action:**
3. Inspect table headers (`<th>`)

**Verify:**
- [ ] All headers have `scope="col"`
- [ ] Proper semantic structure

**Action:**
4. Inspect Document Grid table

**Verify:**
- [ ] Same semantic structure
- [ ] Proper ARIA labels

### 8.3 Keyboard Navigation

**Action:**
1. Use **Tab** key to navigate through the page
2. Observe focus indicators

**Expected Behavior:**
- [ ] Tab order is logical (top to bottom, left to right)
- [ ] Focus indicators visible on all interactive elements
- [ ] Can tab through all buttons, inputs, dropdowns
- [ ] No keyboard traps
- [ ] Can navigate collapsible sections

### 8.4 Collapsible Sections Accessibility

**Action:**
1. Find a collapsible section in Metrics Dashboard
2. Inspect the expand/collapse button

**Verify:**
- [ ] `aria-expanded="true"` when expanded
- [ ] `aria-expanded="false"` when collapsed
- [ ] `aria-label` describes the action

**Action:**
3. Click to collapse the section

**Verify:**
- [ ] `aria-expanded` updates to "false"
- [ ] Screen reader would announce state change

### 8.5 Icon Accessibility

**Action:**
1. Inspect any decorative icons (chevrons, grip handles, etc.)

**Verify:**
- [ ] All decorative icons have `aria-hidden="true"`
- [ ] Icons are not in tab order
- [ ] Associated text provides context

---

## Part 9: Performance Verification

### 9.1 React DevTools Profiler

**Action (requires React DevTools extension):**
1. Install React DevTools if not already installed
2. Open React DevTools
3. Go to **Profiler** tab
4. Click **Record**
5. Toggle period type from Quarterly to Annual
6. Stop recording

**Expected Behavior:**
- [ ] Minimal re-renders outside of changed components
- [ ] Memoized components (MetricsDashboard, DocumentGrid) don't re-render unless props change
- [ ] Handler functions don't cause parent re-renders
- [ ] Flamegraph shows efficient render tree

### 9.2 Visual Performance

**Action:**
1. Toggle period type multiple times quickly
2. Change period count several times
3. Resize panes back and forth

**Expected Behavior:**
- [ ] No visible lag or stuttering
- [ ] Animations smooth (60fps feel)
- [ ] No layout thrashing
- [ ] Responsive to input
- [ ] No memory leaks (check DevTools Memory tab if concerned)

---

## Part 10: Polish Verification

### 10.1 Visual Consistency

**Check Throughout:**
- [ ] Consistent spacing and padding
- [ ] Aligned elements
- [ ] Proper font weights and sizes
- [ ] Color scheme consistent
- [ ] Icons properly sized and styled
- [ ] Shadows and borders consistent

### 10.2 Hover States

**Test Hover:**
- [ ] All buttons have hover states
- [ ] Resize handle highlights on hover
- [ ] Table rows highlight on hover
- [ ] Drag handles appear on row hover
- [ ] Tooltips appear where appropriate

### 10.3 Loading States

**Verify:**
- [ ] Skeleton loaders match content shape
- [ ] Loading spinners visible and animated
- [ ] No flash of unstyled content
- [ ] Smooth transitions from loading to loaded
- [ ] "Refreshing..." badge visible when appropriate

### 10.4 Status Indicators

**Check Document Grid:**
- [ ] "Up to date" shown when fresh
- [ ] "Refreshing..." badge with spinner when updating
- [ ] "Updates Available" badge when stale
- [ ] Badges properly styled and positioned

---

## Part 11: End-to-End Workflow Test

### Complete User Journey

**Scenario:** Analyst opens AAPL, reviews metrics, records verdict

1. **Start**: Click "Open AAPL Analyzer"
   - [ ] Loads successfully

2. **Review Controls**: Check company info
   - [ ] Company name visible (Apple Inc.)
   - [ ] Period controls functional

3. **Explore Metrics**: 
   - [ ] Expand/collapse sections
   - [ ] Observe heat map colors
   - [ ] Scroll through metrics
   - [ ] Values make sense

4. **Check Documents**:
   - [ ] Document grid shows availability
   - [ ] Try downloading a document (if available)
   - [ ] Reorder rows with drag-and-drop
   - [ ] Order persists on refresh

5. **Record Verdict**:
   - [ ] Select verdict (INVEST)
   - [ ] Enter summary text
   - [ ] Add strengths and weaknesses
   - [ ] Press Ctrl+S to save
   - [ ] Toast confirms save

6. **Test Close Behavior**:
   - [ ] Make a change without saving
   - [ ] Press Escape
   - [ ] Warning dialog appears
   - [ ] Cancel and verify can continue working

7. **Final Save and Close**:
   - [ ] Save changes
   - [ ] Press Escape
   - [ ] Should close without warning
   - [ ] Returns to home

---

## Checklist Summary

### Critical Items (Must Pass)
- [ ] All data loads without errors
- [ ] Period toggles work correctly
- [ ] Error handling shows retry buttons
- [ ] Pane resizing is smooth
- [ ] Keyboard shortcuts (Ctrl+S, Escape) work
- [ ] Close warning appears when appropriate
- [ ] ARIA labels present throughout

### Important Items (Should Pass)
- [ ] Performance is smooth (no lag)
- [ ] Memoization prevents unnecessary re-renders
- [ ] Loading states are clear
- [ ] Tables have proper semantics
- [ ] Keyboard navigation works well

### Nice-to-Have Items
- [ ] Animations are polished
- [ ] Visual consistency throughout
- [ ] Hover states on all interactive elements
- [ ] Status indicators clear and helpful

---

## Troubleshooting

### Issue: Services won't start
**Solution:**
```bash
docker compose down
docker compose up -d --build
```

### Issue: Frontend shows blank page
**Solution:**
```bash
# Check frontend logs
docker compose logs frontend

# Rebuild frontend
docker compose up -d --build frontend
```

### Issue: API calls fail
**Solution:**
```bash
# Check backend is running
docker compose ps backend

# Check backend logs
docker compose logs backend

# Verify database is up
docker compose ps postgres
```

### Issue: Data doesn't load
**Solution:**
- Check browser console for errors
- Verify backend API is accessible at `/api/v1`
- Ensure test data is seeded in database
- Check network tab for failed requests

---

## Success Criteria

**Step 8.9 is visually confirmed when:**

✅ **All 11 parts completed without critical failures**  
✅ **Data flows correctly between panes**  
✅ **Error handling works with retry buttons**  
✅ **Keyboard shortcuts functional**  
✅ **Accessibility features present**  
✅ **Performance is smooth**  
✅ **Polish and UX are professional**

---

## After Verification

Once visual verification passes:

1. **Document any issues found** in a separate file
2. **Fix critical issues** before proceeding
3. **Proceed to Step 8.10** (E2E Tests) if required
4. **Commit all changes** to git
5. **Consider this step FULLY COMPLETE**

---

**Time Estimate**: 30-45 minutes for complete verification  
**Recommended**: Do this verification with a fresh browser session  
**Tip**: Use browser's DevTools Console and Network tabs throughout
