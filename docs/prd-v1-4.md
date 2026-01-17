# **Investment Research Platform - Product Requirements Document (PRD)**

## **Document Information**

- **Version**: 1.4
- **Date**: January 16, 2026
- **Author**: Product Requirements
- **Status**: Draft for Review
- **Related Documents**: Investment-Research-Platform-PRFAQ.md
- **Changes from v1.3**: 
  - **Architecture Redesign**: Changed from desktop application to cloud-hosted web application
  - **Database Strategy**: Updated from "local database" to "Centralized Cloud Database" (PostgreSQL/MySQL)
  - **Authentication**: Added JWT-based API authentication requirements
  - **Background Services**: Server-side daemon for nightly polling (2:00 AM IST)
  - **Security**: Added comprehensive security requirements (encryption at rest, secrets management, API authentication)
  - **Deployment Architecture**: New section detailing production and development environments
  - **Terminology Clarification**: "Local storage" now means "cached data in cloud database" (not device-local)
  - **Scalability**: Updated database sizing from <1GB to ~50GB (cloud-hosted documents + metadata)

***

## **1. Executive Summary**

The Investment Research Platform is an institutional-grade equity analysis tool designed to enable systematic, process-driven investment research across US and Indian markets. The platform addresses critical workflow inefficiencies that currently consume 95% of research time on infrastructure tasks, limiting analytical throughput to 20-30 companies weekly.

**Primary Objective**: Enable analysis of 50-75 companies weekly (2.5x throughput increase) by collapsing multi-tool workflows into unified, single-screen experiences with intelligent automation of data retrieval, document processing, and tracking logic.

**Phase One Scope**: Single-user web application (cloud-hosted backend + browser-based frontend) with three core modules—Screener, Analyzer, Results Tracker—plus authentication and landing page. Supports US and Indian equity markets.

**Key Design Shift (v1.3)**: Analyzer workflow now emphasizes document access and download over automated analysis, enabling user-driven analysis via external tools (Perplexity, custom LLM) for maximum flexibility and cost efficiency.

***

## **2. Goals & Success Metrics**

### **Primary Success Metric**

- **Analytical Throughput**: Increase from 20-30 companies/week baseline to 50-75 companies/week target
- **Measurement**: Count of companies with completed verdict recordings per week

### **Secondary Metrics**

- **Time Efficiency**: Reduce per-company analysis time from 20 minutes to 5 minutes (75% reduction)
- **Decision Quality**: Track percentage of "Invest" verdicts held 6+ months (proxy for process discipline)
- **Cost Efficiency**: Total platform costs (data APIs + LLM + hosting) vs. current $12,000/year Capital IQ + tools

### **Operational Metrics**

- **Data Freshness**: 100% of companies analyzed with data <7 days from latest earnings
- **Automation Rate**: 95%+ of documents retrieved automatically (vs. manual upload)
- **System Reliability**: 99%+ uptime
- **Document Availability**: 90%+ of available company documents surfaced in Document Repository Grid

***

## **3. User Personas & Use Cases**

### **Primary User: Quantitative Fund Manager**

- **Profile**: Solo institutional investor managing long-only equity strategy focused on revenue acceleration and business model inflection points
- **Technical Proficiency**: Advanced (comfortable with data-dense interfaces, power-user patterns)
- **Workflow Pattern**: Systematic screening → rapid metrics review → download documents → external analysis (Perplexity/custom LLM) → structured verdict recording
- **Time Constraints**: Must evaluate 50-75 companies weekly within fixed research time budget
- **Tool Stack**: Integrates Investment Platform with external LLM tools for qualitative analysis

### **Primary Use Cases**

**UC1: Weekly Screening & Analysis**

- Run saved screeners with configurable filters (market cap, sector, momentum)
- Identify companies needing fresh analysis (never reviewed OR new quarterly results)
- Rapidly analyze 50-75 companies: review metrics (80% reject) → download documents for promising candidates → analyze offline → record verdicts

**UC2: Results-Driven Monitoring**

- Track earnings announcement dates across all companies in historical screener universe
- Analyze companies with newly announced results to catch inflection points early
- Download documents immediately upon announcement → run through Perplexity → record assessment
- Avoid missing "needle in haystack" opportunities outside standard screening criteria

**UC3: On-Demand Research**

- Directly search and analyze any company across US/Indian markets
- Download documents → external analysis → review historical assessment
- Update assessments when new information emerges or investment thesis evolves

***

## **4. Functional Requirements**

### **4.1 Authentication & Landing Page**

#### **FR-AUTH-001: User Authentication**

- System shall provide login functionality requiring username/password
- System shall implement JWT-based API authentication
  - Access tokens expire after 24 hours
  - Refresh tokens valid for 30 days
  - All API endpoints (except /login and public landing page) require valid JWT Bearer token
- System shall maintain proper user account database schema supporting future multi-user expansion
- Phase One shall have single active user account (user registration disabled)
- System shall maintain session state across platform navigation
- Frontend shall include Authorization header: `Bearer <token>` for all authenticated API requests

#### **FR-AUTH-002: Public Landing Page**

- System shall display public landing page at root URL without authentication
- Landing page shall present static educational content on growth cycle investing philosophy
- Landing page shall NOT display navigation links to Screener/Analyzer/Results Tracker when unauthenticated

#### **FR-AUTH-003: Authenticated Navigation**

- Upon successful login, landing page shall display left navigation pane with links to:
    - Screener
    - Analyzer
    - Results Tracker
- System shall NOT auto-redirect after login; user remains on landing page with navigation visible
- Left navigation pane shall be collapsible

***

### **4.2 Screener Module**

#### **FR-SCR-001: Screener Management Interface**

System shall provide three-pane layout:

- **Vertical Pane (Left)**: Collapsible navigation with module links
- **Horizontal Pane 1 (Top)**: List of saved screeners with management controls
- **Horizontal Pane 2 (Bottom)**: Screener results display

#### **FR-SCR-002: Saved Screener List (Pane 1)**

System shall display list of saved screeners showing:

- Screener title
- Description of filter criteria
- Actions: Edit, Delete, Run

System shall provide "Create New Screener" action

#### **FR-SCR-003: Screener Creation & Editing**

System shall provide filter configuration interface with following options:

**Required Filters (must select at least one + exchange):**

1. **Exchange/Country Selection**
    - Initial support: USA, India
    - User selects from country list → displays associated exchanges
2. **Market Capitalization Range**
    - Lower bound (optional)
    - Upper bound (optional)
    - At least one bound must be specified if using this filter
3. **Sector Classification** (optional multi-select)
4. **Price Momentum** (optional)
    - Weekly % change
    - Monthly % change
    - Quarterly % change

**Validation Rules:**

- Must select at least one exchange/country
- Must select at least one additional filter beyond exchange
- Cannot create screener with ONLY exchange selection

#### **FR-SCR-004: Screener Execution Logic**

When user runs a saved screener:

- System shall execute filter criteria against CURRENT data (dynamic execution)
- System shall apply data freshness heuristics:
    - Market cap: Refresh if >1 trading day old
    - Fundamental data: Refresh if new quarterly results announced since last fetch
    - If same trading day + no new results: Use cached data
- System shall compute all derived metrics (revenue acceleration, margin expansion, etc.) from persisted derived metrics store
- System shall retrieve analysis state from database for all companies in results

#### **FR-SCR-005: Screener Results Display (Pane 2)**

System shall display results table with following columns:

1. **Company Name** (clickable → opens Analyzer)
2. **Country/Exchange**
3. **Market Cap**
    - Display in USD
    - Units: M (million) or B (billion)
    - Auto-select unit to keep ≤4 digits before decimal
4. **Analysis Status**
    - If never analyzed: Display "Not Analyzed" with visual indicator (e.g., orange badge)
    - If analyzed with latest results: Display analysis date (YYYY-MM-DD format) with visual indicator (e.g., green badge)
    - If analyzed but new results available: Display last analysis date + "New Results Available" tag with visual indicator (e.g., yellow badge)
5. **Sequential YoY Growth Acceleration — Delta (pp)** *(primary)*
    - Formula: YoY Growth[n] - YoY Growth[n-1] (percentage points)
    - Display with 1 decimal place
6. **Sequential YoY Growth Acceleration — Ratio (%)** *(optional column; off by default)*
    - Formula: (YoY Growth[n] - YoY Growth[n-1]) / |YoY Growth[n-1]|
    - Display with 1 decimal place
7. **Sequential OP Margin Expansion %**
    - Formula: (OPM Q[n] - OPM Q[n-1]) / |OPM Q[n-1]|
8. **OCF % Revenue** (latest Q)
9. **FCF % Revenue** (latest Q)
10. **(Revenue - Net Debt) / Revenue %** (latest Q)
11. **Forward Looking Guidance & Optionality**
    - If company analyzed: Display cached 50-100 word summary from Analyzer verdict
    - If new results available: Display previous summary in grey text + "New Results Available" tag
    - If never analyzed: Display "Not Analyzed Yet"

#### **FR-SCR-006: Results Sorting & Prioritization**

System shall automatically sort screener results:

- **Top Section**: Companies needing current analysis (never analyzed OR new results available)
- **Bottom Section**: Companies already analyzed with no new results
- Visual differentiation: Color coding or greyed-out presentation
- All companies remain clickable (no inactive rows)

#### **FR-SCR-007: Results Interaction**

System shall support:

- Column resizing
- Column sorting (ascending/descending)
- Column filtering (similar to Excel filtering)
- Clicking company name/ticker → Opens Analyzer window for that company
- Pane resizing and pop-out to full screen

#### **FR-SCR-008: Screener Persistence**

- All screener configurations shall be saved to database with user-provided title and auto-generated description
- Saved screeners shall be editable (modify filter values)
- Saved screeners shall be deletable with confirmation prompt
- No limit on number of saved screeners

***

### **4.3 Analyzer Module**

#### **FR-ANL-001: Analyzer Interface Layout**

System shall provide four-pane layout:

- **Vertical Pane (Left)**: Collapsible navigation
- **Horizontal Pane 0 (Top Strip)**: Controls bar (hover-visible, user-pinnable)
- **Horizontal Pane 1**: Key Metrics Dashboard
- **Horizontal Pane 2**: Document Repository Grid
- **Horizontal Pane 3 (Bottom)**: Verdict & Assessment Recording

#### **FR-ANL-002: Analyzer Entry Points**

Analyzer window can be opened via:

1. Clicking company name in Screener results
2. Clicking company name in Results Tracker
3. Direct search via global search bar in Pane 0
4. All methods open company in new window

#### **FR-ANL-003: Multi-Window Management**

- System shall support multiple browser tabs/windows analyzing different companies simultaneously
- Each tab/window maintains independent client-side state
- Backend enforces optimistic locking with version timestamps to prevent concurrent edit conflicts
  - If two tabs attempt to save verdicts for same company simultaneously, second save fails with conflict error
  - User prompted to refresh and merge changes manually
- If company already has open tab, clicking again shall bring existing tab to foreground (browser behavior)
- No data auto-refresh within open tabs (refresh only on close/reopen)

#### **FR-ANL-004: Close Window Behavior**

When user attempts to close Analyzer window:

- If NO verdict parameters recorded in Pane 3: Show prompt "No analysis recorded. Close anyway?"
    - If user confirms: Close without saving
    - If user cancels: Return to Analyzer window
- If ANY verdict parameters recorded: Save all entered data to database and close

#### **FR-ANL-005: Controls Bar (Pane 0)**

System shall provide top strip with following controls:

- **Global Company Search Bar**
    - Searches across all available tickers in US + Indian markets
    - Allows analysis of companies not in any saved screener
    - Autocomplete/typeahead functionality
- **Period Toggle**: Quarterly / Annual (affects Pane 1 display)
- **Period Range Dropdown**: 4, 5, 6, 7, 8, 9, 10 periods (affects Pane 1 and Pane 2 display)
- **Currency Toggle**: USD / Local Currency (affects Pane 1 display)

Default state: Hover-visible (disappears when mouse moves away)
User can pin to always-visible mode

***

### **4.4 Analyzer - Pane 1: Key Metrics Dashboard**

#### **FR-ANL-010: Page Title & Company Header**

- Title: "Key Metrics of [Company Name]"
- Mouse hover on company name anywhere: Display tooltip with short company description

#### **FR-ANL-011: Data Display Period**

- Display format determined by Period Toggle (Quarterly or Annual) and Period Range (4-10)
- All metrics displayed in tabular format with periods as columns

#### **FR-ANL-012: Section 1 - Growth & Margins**

Display following metrics as rows:

1. **Revenue**
    - Auto-convert to nearest sub-3-digit unit (K, M, B, T)
    - Row header shows unit: e.g., "Revenue ($B)" or "Revenue (₹M)"
    - Display with 1 decimal place
2. **YoY Growth %**
    - Growth vs. same period prior year
    - Display with 1 decimal place, italics
    - Heat map: Deep green (highest) to deep orange (lowest)
3. **Sequential YoY Growth Acceleration — Delta (pp)** *(primary)*
    - Formula: YoY Growth[n] - YoY Growth[n-1]
    - Display with 1 decimal place, italics
    - Heat map coloring
4. **Sequential YoY Growth Acceleration — Ratio (%)** *(secondary)*
    - Formula: (YoY Growth[n] - YoY Growth[n-1]) / |YoY Growth[n-1]|
    - Display with 1 decimal place, italics
5. **Gross Margin %**
    - Display with 1 decimal place, italics
6. **Operating Margin %**
    - Display with 1 decimal place, italics
7. **Sequential OP Margin Expansion %**
    - Formula: (OPM[n] - OPM[n-1]) / |OPM[n-1]|
    - Display with 1 decimal place, italics
    - Heat map coloring
8. **PBT %**
    - Display with 1 decimal place, italics

#### **FR-ANL-013: Section 2 - Financial Architecture**

Display following metrics as rows:

1. **OCF % Revenue**
    - Display with 1 decimal place, italics
    - Heat map coloring
2. **FCF % Revenue**
    - Display with 1 decimal place, italics
3. **(Revenue - Net Debt) / Revenue %**
    - Display with 1 decimal place, italics
    - Heat map coloring
4. **Total Shares Outstanding**
    - Auto-convert to appropriate unit (M, B)
    - Row header shows unit
    - Display with 1 decimal place

#### **FR-ANL-014: Section 3 - Valuation Metrics**

Display OHLC (Open, High, Low, Close) for each period:

1. **P/S or EV/Revenue**
    - Heat map coloring on Close values
2. **P/E**
3. **EV/FCFF or MCap/FCFE**

Format: Each metric row shows 4 sub-rows (O, H, L, C) per period column

#### **FR-ANL-015: Currency Conversion Logic (store local immutable; USD recomputed)**

When Currency Toggle = Local Currency:

- Display original reporting currency values as-reported and never modify stored local values after ingestion

When Currency Toggle = USD:

- For non-US stocks, display USD values computed from local values using the latest available daily FX rate; these USD values are recomputed when a new FX rate is ingested
- For US stocks, local currency equals USD; no FX conversion is applied
- Percentage metrics remain unconverted (currency-agnostic)
- Row headers shall update to show "(USD)" or "($)"

#### **FR-ANL-016: Heat Map Visualization**

For designated rows (YoY growth, acceleration, OP margin expansion, OCF %, net debt ratio, valuation):

- Apply color gradient across row values
- Deep green = highest value in row
- Deep orange = lowest value in row
- Gradient interpolation for intermediate values
- For valuation row only, invert the heat map colors; because for valuation, lower the number better it is for the investor

#### **FR-ANL-017: Pane Interaction**

- Individual metric panels resizable
- Individual panels pop-out to detailed window for focused examination
- Preserves single-screen efficiency while enabling deep-dive when needed

***

### **4.5 Analyzer - Pane 2: Document Repository Grid**

#### **FR-ANL-020: Pane Title & Layout**

- Title: "Document Repository for [Company Name]"
- Layout: Single document grid interface displaying available company documents organized by time period and document type
- Purpose: Centralized document access hub enabling rapid download of all qualitative materials for offline analysis

#### **FR-ANL-021: Document Grid Structure**

System shall display a 2D grid with the following structure:

**Columns (Time Periods):**

- Display time periods aligned with Pane 1 Period Range selector (4-10 periods)
- Column headers shall show dual notation:
  - **Native Period**: YYYY-MM (e.g., 2025-03, 2024-12)
  - **Derived Period**: Q-notation for interim periods, FY-notation for fiscal year-end periods
    - Example: "2025-03 (Q1 2025)" or "2024-12 (FY24)"
    - Note: Q4 notation shall not be used; fiscal year-end quarters automatically display as "FY" regardless of calendar quarter
- Period range automatically updates when user changes Pane 1 Period Range dropdown
- Columns span both quarterly and annual periods in chronological order (most recent leftmost)

**Rows (Document Types):**

System shall display following document type rows in fixed default order:

1. **Investor Presentation**
2. **Earnings Call Transcript**
3. **Earnings Release**
4. **Quarterly Report (10-Q for US / Quarterly Results for India)**
5. **Annual Report (10-K for US / Annual Report for India)**
6. **Other Documents** (expandable row showing additional miscellaneous filings)

**Row Reordering:**

- User can drag-and-drop rows to customize display order
- Reordered preferences persist across sessions per user profile
- "Reset to Default Order" action available

#### **FR-ANL-022: Cell Content & File Display**

Each grid cell represents the intersection of document type (row) and time period (column).

**Cell Display Rules:**

**When documents are available:**

- Display file format icon(s) as clickable download buttons
- Supported formats (MVP): PDF, PowerPoint (.ppt/.pptx)
- Supported formats (Phase 1.5+): Markdown (.md), Word (.doc/.docx), plain text (.txt)
- Icon design: Small recognizable file type icons (e.g., red PDF icon, orange PPT badge)
- All file icons shown must be downloadable; documents persisted in centralized cloud database

**Multiple files in same cell:**

- **Same document, different formats** (e.g., transcript as PDF + future Markdown):
  - Display icons horizontally on same line within cell
  - Example: [PDF icon] [MD icon]
- **Different documents, same type/period** (e.g., two investor presentations in Q3 2025):
  - Display icons vertically on separate lines within cell
  - Each line can have multiple format icons horizontally
  - Example:
    ```
    Line 1: [PDF icon] [PPT icon]  ← Investor Deck v1
    Line 2: [PDF icon]              ← Investor Deck v2
    ```

**When no documents available:**

- Display manual upload button: "+Upload" or cloud upload icon
- On click, opens file picker dialog
- Supports PDF, PPT formats (MVP); future phases support additional formats
- Uploaded document stored in database with period and type metadata

**File Metadata Tooltip:**

- On hover over any file icon, display tooltip showing:
  - Document title/name
  - File size
  - Source (e.g., "SEC Edgar", "NSE Filing", "Company IR Website", "Manual Upload")
  - Upload/retrieval date
  - File format

#### **FR-ANL-023: Document Retrieval & Source Priority**

**Automated Document Retrieval:**

System shall automatically retrieve documents from data providers when Analyzer opens:

- Poll APIs and direct sources (SEC Edgar, NSE/BSE portals, company IR websites)
- Store retrieved documents in centralized cloud database with metadata
- Retrieval timestamp, source, period identifier (YYYY-MM), document type, file format

**Source Priority Logic:**

When multiple sources provide the same document (by type + period):

1. **Primary Source (Always Include)**:
   - Direct company filings: Exchange regulator filings (SEC Edgar, NSE, BSE)
   - Company investor relations website
   - Show all formats available from primary source

2. **Secondary Source (Conditional Include)**:
   - API providers (Quartr, Financial Modeling Prep, etc.)
   - **If same format as primary source**: Do NOT display (deduplicate by source + format combination)
   - **If different format** (e.g., primary has PDF, secondary has future Markdown): Display both formats with source differentiation in tooltip
   - Show source in tooltip to distinguish

3. **Manual Upload (Always Include)**:
   - User-uploaded documents always displayed
   - Clearly labeled as "Manual Upload" in tooltip

**Document Storage & Persistence:**

- All retrieved documents persisted in centralized cloud database with full metadata
- Documents stored as BLOBs or in cloud object storage (AWS S3/GCS) with database references
- Do not re-fetch documents unless user explicitly requests refresh
- Automatic refresh check: Server-side daemon polls APIs nightly for new documents
- Client fetches documents via authenticated API endpoints on-demand

**MVP Phasing (Document Sources):**

- **MVP Launch**: SEC Edgar (US), NSE/BSE portals (India)
- **Phase 1.5**: Additional API providers, company IR website scraping, secondary sources

#### **FR-ANL-024: Download Workflow & File Management**

**Download Interface:**

When user clicks on file icon:

- Display browser-like "Save As" dialog
- Pre-fill filename with database-stored document title
- Point to default download location (user configurable)
- Dialog workflow similar to standard browser file downloads
- Upon completion, file saved to user's local system

**Bulk Download Feature:**

- Provide selection interface with checkboxes for document types (rows) and periods (columns)
- **Selection UI**: Radio buttons or checkboxes to select:
  - One or all rows (document types)
  - One or all columns (periods)
  - Individual cells
- "Download All" button initiates bulk download
- System packages selected documents and initiates batch download
- Downloaded files maintain original naming and structure

**No Download Tracking:**

- System does NOT track which documents have been downloaded by user
- No visual indicators (checkmarks, dimming) for downloaded status
- Focus on document access, not usage analytics (Phase 1+ enhancement)

***

### **4.6 Analyzer - Pane 3: Verdict & Assessment Recording**

#### **FR-ANL-030: Pane Layout & Purpose**

- Title: "Investment Assessment"
- Located at bottom of Analyzer page (user scrolls after reviewing Panes 1-2)
- Captures structured assessment across investment philosophy dimensions
- Workflow: User downloads documents from Pane 2 → analyzes offline via external LLM tool (Perplexity/custom) → returns to Pane 3 to record verdict

#### **FR-ANL-031: Assessment Parameters**

System shall provide input fields for following enumerated assessments:

1. **Growth Acceleration without Margin Compression**
    - Options: Yes / No
    - Default: Unselected
2. **Strong Operating Cash Flow Quality**
    - Options: Yes / No
    - Default: Unselected
3. **Manageable Net Debt Levels**
    - Options: Yes / No
    - Default: Unselected
4. **Future Growth Potential (Qualitative)**
    - Options: Strong Acceleration / Slight Acceleration / Flat / Downward / Unclear
    - Default: Unselected
5. **Valuation Setup vs. Revenue Acceleration**
    - Options: Attractive / Fair / Expensive / Unclear
    - Default: Unselected
6. **Final Verdict**
    - Options: Invest / Pass / Watchlist
    - Default: Unselected

#### **FR-ANL-032: Free-Text Summary Field**

- Input: Multi-line text box (200-500 character limit)
- **Blank by default** (no auto-population)
- User fills manually after offline analysis based on documents downloaded
- Purpose: Captures 50-100 word summary of company business, recent performance, growth potential, and analyst assessment
- User can edit/override text on subsequent visits

#### **FR-ANL-033: Offline Analysis Report Upload (Optional)**

- Optional "Upload Analysis Report" button
- Allows user to attach analysis documents generated offline (Perplexity output, custom reports, etc.)
- Supported formats: PDF, DOCX, TXT, Markdown
- Uploaded reports stored in database associated with company and verdict version
- Maintains audit trail of analysis methodology
- Not mandatory; supports optional documentation of analysis process

#### **FR-ANL-034: Auto-Populated Metadata**

System shall automatically capture and display (read-only):

1. **Review Date**
    - Auto-populated with current date (YYYY-MM-DD)
    - Updates each time verdict is modified
2. **Latest Results Quarter**
    - Pulled from API metadata. Format of the quarter designator should be YYYY-MM format. Do not use 'Q42025' format because quarter ends of each company can be different but with same name
    - Indicates which quarter's data the analysis covers
3. **Analysis Version**
    - Incremented each time verdict parameters are edited
    - Format: "Version X edited on YYYY-MM-DD HH:MM"

#### **FR-ANL-035: Partial Save & Staged Analysis**

System shall support incremental verdict recording:

- User can fill ANY subset of parameters (1 out of 6, 3 out of 6, etc.)
- Clicking "Save" or closing window (with confirmation) stores all entered fields
- Fields left blank stored as NULL
- User can reopen company later and complete remaining fields
- All subsequent saves treated as edits (version incremented)

#### **FR-ANL-036: Verdict Editing & History**

When reopening previously analyzed company:

- Pane 3 displays last saved verdict with all parameters filled as previously recorded
- All fields remain editable
- Changes trigger version increment and timestamp update
- System maintains full edit history in database:
    - All parameter values per version
    - Timestamp of each edit
    - User ID (for future multi-user support)

**Edit History Display:**

- "View History" button in Pane 3
- Opens modal showing chronological list of all versions:
    - Version number
    - Edit timestamp
    - Changed parameters (diff view)
    - Ability to view full verdict snapshot from any version
    - References to uploaded analysis reports per version

#### **FR-ANL-037: Verdict Validation**

System shall NOT enforce mandatory field completion:

- User can save with 0 fields filled (analysis viewed but no judgment recorded)
- User can save with only some fields filled (partial analysis)
- No validation errors for incomplete verdicts
- Warning prompt on close if all fields blank: "No assessment recorded. Close anyway?"

#### **FR-ANL-038: Database Persistence**

All verdict data shall be stored with following attributes:

- Company identifier (ticker + exchange)
- Latest results quarter analyzed (YYYY-MM format)
- Each assessment parameter value (nullable)
- Free-text summary (nullable)
- Review date (timestamp)
- Version number
- Full edit history (separate historical records table)
- References to uploaded analysis reports (optional)

***

### **4.7 Results Tracker Module**

#### **FR-RTK-001: Purpose & Scope**

Results Tracker module provides universal earnings monitoring across all companies in user's research universe, serving as "brute force" supplement to selective Screener-based analysis.

#### **FR-RTK-002: Company Universe Definition (with lifecycle)**

System shall track earnings announcement dates for:

- All companies that have appeared in ANY saved screener historically (running any screener adds companies to tracking universe)
- Each tracked company has a lifecycle state: `Active` or `Archived`
- Default archival rule: if a company has **not appeared in any screener run for 12 months**, set state to `Archived` (reversible by user)
- Nightly polling runs only for `Active` companies

#### **FR-RTK-003: Results Tracker Display**

System shall display table with following columns:

1. **Company Name** (clickable → opens Analyzer)
2. **Exchange/Country**
3. **Lifecycle Status** (Active / Archived)
    - User can toggle archived companies to active
4. **Earnings Announcement Date**
5. **Status**
    - "Upcoming" (date in future)
    - "Announced - Data Pending" (date passed, financial data not yet available in APIs)
    - "Announced - Data Available" (date passed, data available)
6. **Last Analysis Date**
    - If never analyzed: "Not Analyzed"
    - If analyzed: Display date (YYYY-MM-DD)
7. **Latest Results Quarter**
    - Pulled from API metadata. Format of the quarter designator should be YYYY-MM format. Do not use 'Q42025' format because quarter ends of each company can be different but with same name

#### **FR-RTK-004: Sorting & Filtering**

Default sort:

- Announced + Not Analyzed (top priority)
- Announced + Analyzed (middle)
- Upcoming (bottom)

Within each group: Sort by earnings date (most recent first)

User can apply filters:

- Date range (show only results within next 7/14/30 days)
- Exchange/Country
- Analysis status (Analyzed / Not Analyzed)
- Sector
- Lifecycle status (Active / Archived / All)

#### **FR-RTK-005: Results Tracker Interaction**

- Clicking company name → Opens Analyzer window for that company
- Upon saving verdict in Analyzer, Results Tracker display updates:
    - If verdict covers latest announced quarter: Company removed from default view (user can re-enable via filter)
    - If verdict covers old quarter: Company remains visible with "New Results Available" indicator

#### **FR-RTK-006: Archival Management**

- Automatic archival: System evaluates all tracked companies nightly; companies not in any screener run for 12 months automatically archived
- Manual archival: User can right-click → "Archive" on any company (immediate)
- Restoration: User can restore archived companies to active via UI
- Restored companies begin receiving nightly earnings date polling again

#### **FR-RTK-007: Earnings Announcement Polling**

System shall execute nightly polling (time TBD based on market) for all Active companies:

- Query data providers (CapitalIQ, Quartr, Financial Modeling Prep, etc.) for latest earnings announcement dates
- Update database with most recent announced date
- Flag companies with announced results but missing financial data (wait 1-3 business days post-announcement)
- Once financial data available in primary API: Mark as "Data Available" and surface in Analyzer for analysis
- Store polling timestamp and source per company

#### **FR-RTK-008: Results Tracker Persistence**

- All company lifecycle states (Active/Archived) persisted in database
- User archival preferences maintained
- Earnings announcement dates updated automatically via polling
- Analysis status synced with Verdict Recording system (Pane 3 saves)

***

### **4.8 Data Infrastructure**



#### **FR-DATA-000: System Architecture Overview**

**Architecture Type**: Client-Server Web Application

**Client (Frontend):**
- Browser-based single-page application (React/Vue.js)
- Desktop browsers only (Chrome/Edge/Firefox/Safari 90+)
- No data storage beyond browser session state
- All data fetched via authenticated REST API calls

**Backend API:**
- RESTful API with JWT authentication
- Handles all business logic and data processing
- Python (FastAPI/Django REST) or Node.js (Express)
- Deployed as containerized application

**Database:**
- Centralized private cloud database (PostgreSQL or MySQL)
- Hosted on managed cloud service (AWS RDS/GCP Cloud SQL/Azure Database)
- Stores all financial metrics, documents, user verdicts, screener configurations
- Encryption at rest enabled

**Background Services:**
- Server-side daemon for nightly polling (earnings dates, document refresh)
- Scheduled via cloud cron service (AWS EventBridge/GCP Cloud Scheduler)
- Independent of user sessions; runs at 2:00 AM IST daily

**Data Storage Philosophy:**
- **"Cached data"**: Data persisted in centralized cloud database (not fetched from external APIs)
- **"Live data"**: Real-time API calls to external data providers
- **No device-local storage**: Client never stores data locally except browser session tokens
- **Cloud-first**: All documents, metrics, verdicts stored in cloud for resilience, security, device-agnostic access

#### **FR-DATA-001: Data Providers & API Integration**

**MVP Scope (Phase One):**

- **Market Data APIs**: Fundamentals, valuation, FX rates
  - Provider: [TBD during Phase Zero]
  - Refresh: Daily EOD for equities, quarterly for fundamentals
  - Fallback: Yahoo Finance, Alpha Vantage (for initial development)
  
- **US Company Documents**:
  - SEC Edgar API (free, direct source)
  - Earnings releases, 10-Q, 10-K filings
  
- **Indian Company Documents**:
  - NSE/BSE XBRL filings (via direct portal scraping or API if available)
  - Earnings releases and investor presentations
  - Fallback: Screener.in API for document metadata

**Phase 1.5+ Scope (Future):**

- Additional API providers for document availability
- Company investor relations website scraping
- Markdown conversion services for PDFs/PPTs
- Transcription services for audio (earnings calls)

#### **FR-DATA-002: Earnings Announcement Polling**

Server-side daemon shall execute automated nightly polling for Results Tracker:

- **Execution Time**: 2:00 AM IST (post-US market close, pre-Indian market open)
- **Architecture**: Independent background service (not triggered by user action)
- **Scope**: All Active companies in Results Tracker universe
- **Query**: Latest earnings announcement date, expected result release dates
- **Update**: Centralized cloud database with new dates, flag companies with announced results
- **Benefit**: Fresh data available when user logs in (no wait time)
- **Log**: Polling timestamp, source, success/failure status, errors

#### **FR-DATA-003: FX Conversion & Currency Management**

- Store all financial data in original reporting currency (immutable)
- Fetch daily FX rates from financial data API
- On Currency Toggle to USD: Compute USD values from local amounts + FX rate
- Percentage metrics (growth %, margins, ratios) remain unconverted (currency-agnostic)
- Store FX rate timestamp to enable historical rate retrieval

#### **FR-DATA-004: Document Storage & Retrieval**

- All retrieved documents stored in centralized cloud database (not streamed on-demand)
- Storage architecture:
  - **Option A**: Documents stored as BLOBs in PostgreSQL/MySQL database
  - **Option B**: Documents stored in cloud object storage (AWS S3/GCS) with database references (preferred for scalability)
- Metadata indexed in database: Source, format, period, document type, retrieval timestamp, file hash (for deduplication)
- Retrieval: Client fetches documents via authenticated API endpoints (backend reads from database/S3)
- Refresh: User-initiated "Refresh Documents" button or automatic nightly refresh by background service
- Scalability: Designed for 1000+ companies with 50+ documents each (~50GB cloud storage)

#### **FR-DATA-005: Data Staleness & Refresh Thresholds**

- **Market Cap, Valuation**: Refresh if >1 trading day old
- **Fundamentals (Revenue, Margins, Cash Flow)**: Refresh if new quarterly results announced
- **FX Rates**: Refresh daily (or on toggle if missing today's rate)
- **Earnings Dates**: Refresh nightly (Results Tracker polling)
- **Document List**: Check nightly; refresh immediately if new results announced

***

### **4.9 Error Handling & User Feedback**

#### **FR-ERROR-001: Missing Data Graceful Degradation**

When documents fail to retrieve:

- Analyzer displays message: "Some documents unavailable. Data retrieved from [list of sources]. Missing: [list]. Use upload feature to add manually."
- Grid shows empty cells with "+Upload" button
- Analysis continues with available documents

When market data API fails:

- Pane 1 displays cached data with timestamp: "Data as of [date]. Refresh failed. Click to retry."
- User can manually trigger refresh
- Analysis continues with available data

#### **FR-ERROR-002: User Notifications**

- Non-blocking toast notifications for document retrieval status
- Dialog prompts for data stale >7 days
- Sidebar alert badge for companies with new results available
- Email notifications (future enhancement, Phase 1.5+) for earnings announcements

***

### **4.10 Future Enhancements (Phase 1.5+)**

The following features are explicitly deferred from MVP but planned for Phase 1.5 release:

1. **LLM-Powered Qualitative Analysis**
   - Bring back Pane 2 LLM-powered 8-section analysis (FR-ANL-022 from v1.1)
   - Structured prompt with document inputs
   - Auto-fill of Pane 3 summary field
   
2. **Markdown Document Conversion**
   - Auto-convert PDF/PPT to Markdown for easier LLM processing
   - Display both original and converted formats in grid
   
3. **Interactive Chat Interface**
   - Expand Pane 2 to include historical chat sessions
   - LLM model selection dropdown
   - Context compaction for multi-year analysis
   
4. **Email Notifications**
   - Earnings announcement alerts
   - New results available notifications
   - Watchlist reminder emails
   
5. **Advanced Bulk Operations**
   - Batch verdict recording for multiple companies
   - Export screening results + verdicts to Excel
   - Comparison analysis (2-3 companies side-by-side)
   
6. **Extended Document Sources**
   - Additional API providers for document retrieval
   - International exchange filings (beyond US/India)
   - Third-party research reports

***

## **5. Non-Functional Requirements**

### **Performance**

- Analyzer shall load metrics within 2 seconds of company selection
- Screener results shall display within 3 seconds of filter execution
- Document grid shall display within 1 second of Analyzer load
- Search autocomplete shall respond within 500ms

### **Scalability**

- Support 1000+ tracked companies in Results Tracker
- Handle 50+ documents per company in storage
- Database sized for ~50GB storage (documents + metadata for single user MVP)
- Backend API scaled to handle 10 concurrent requests (sufficient for single user)
- Designed for future multi-user expansion:
  - Horizontal scaling: Load balancer + multiple API server instances
  - Database: Read replicas for query scaling
  - User-scoped data isolation via user_id foreign keys
  - Session isolation via JWT authentication

### **Reliability**

- 99%+ uptime target
- Automatic retry logic for API failures (exponential backoff)
- Database backup: Daily backup with 30-day retention
- Graceful degradation when data sources unavailable

### **Security & Privacy**

**Authentication & Authorization:**
- JWT-based API authentication with RSA signing (asymmetric keys)
- Password hashing using bcrypt (cost factor 12+)
- Session management with token rotation
- Rate limiting on authentication endpoints (5 login attempts per 15 minutes)
- All API endpoints (except /login and landing page) require valid JWT Bearer token

**Data Protection:**
- All API communication over HTTPS/TLS 1.3
- Database encryption at rest (provider-managed keys)
- API keys for external data providers stored in secrets manager (AWS Secrets Manager/HashiCorp Vault)
- No sensitive data in client-side JavaScript
- SQL injection prevention through parameterized queries/ORM
- Input validation on all user-submitted data

**Infrastructure Security:**
- Cloud-hosted database with automated backups (daily + 30-day retention)
- Database access restricted to backend API server IPs only (no direct internet exposure)
- DDoS protection via cloud provider (CloudFlare/AWS WAF)
- Secrets rotation policy: Database credentials rotated quarterly
- JWT signing keys stored in secrets manager (never in code repository)

### **Browser Compatibility**

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Desktop browsers only (tablet/mobile not supported in MVP)

### **Accessibility**

- WCAG 2.1 AA compliance target
- Keyboard navigation for all controls
- Screen reader compatibility for core workflows
- Color-not-only differentiation for status indicators

***

## **6. Constraints & Assumptions**

### **Technical Constraints**

- Single-user web application (multi-user architecture deferred to Phase 2)
- Cloud-hosted infrastructure (AWS/GCP/Azure)
- **Frontend**: React/Vue.js single-page application (browser-based)
- **Backend API**: Python (FastAPI/Django REST) or Node.js (Express)
- **Database**: PostgreSQL or MySQL (cloud-hosted: AWS RDS/GCP Cloud SQL/Azure Database)
- **Background Jobs**: Celery (Python) or Bull (Node.js) with Redis queue
- **Hosting**: Cloud provider managed services (no on-premise deployment)
- No mobile or tablet support (desktop browsers only)
- Single-user MVP: No multi-tenancy logic; user isolation via API authentication

### **Data Constraints**

- US and Indian equity markets only
- Focus on companies with >$50M market cap
- Free/low-cost data sources prioritized (cost targets: <$200/month)
- No proprietary research synthesis in MVP

### **Business Assumptions**

- User has external LLM access (Perplexity Pro, ChatGPT, Claude) for qualitative analysis
- User manually reviews Pane 2 documents and records verdicts based on own analysis
- Platform designed to complement, not replace, user judgment
- Success measured by analysis throughput (50-75 companies/week), not verdict accuracy

### **Timeline Assumptions**

- Phase Zero (Architecture/Tech Stack/Infrastructure Setup): 3-4 weeks
  - Cloud provider selection (AWS/GCP/Azure)
  - Database schema design
  - API contract definition (OpenAPI specification)
  - Authentication flow implementation
  - CI/CD pipeline setup
  - Infrastructure provisioning
- Phase One (MVP Development): 8-10 weeks
- Phase One Launch: March 2026 (target)
- Phase 1.5 (LLM Qualitative, Markdown, Chat): Q2 2026
- Phase Two (Multi-user, Horizontal Scaling): Q3 2026+

***

***

## **8. Deployment Architecture**

### **Production Environment**

**Frontend (Client):**
- Static React/Vue.js build deployed to CDN (AWS CloudFront/GCP Cloud CDN/Azure CDN)
- Environment variables for API endpoint configuration (injected at build time)
- Asset optimization: Minified JavaScript/CSS, gzip compression
- Domain: TBD (custom domain with SSL certificate)

**Backend API:**
- Containerized application (Docker) deployed to managed container service
  - **Options**: AWS ECS/Fargate, GCP Cloud Run, Azure Container Instances
- Auto-scaling based on CPU utilization (single instance for MVP; scale-out for Phase 2)
- Health check endpoint: `/health` (returns 200 OK with system status)
- Environment variables for database connection, API keys, JWT secrets (injected from secrets manager)
- Logging: Structured logs to cloud logging service (CloudWatch/Cloud Logging/Azure Monitor)

**Database:**
- Managed cloud database (AWS RDS PostgreSQL/GCP Cloud SQL/Azure Database for PostgreSQL)
- Instance size: db.t3.medium (2 vCPU, 4GB RAM) sufficient for MVP
- Storage: 100GB SSD with automatic scaling up to 500GB
- Automated daily backups at 3:00 AM IST with 30-day retention
- Single primary instance (no read replicas in MVP)
- Connection pooling: PgBouncer or equivalent
- Monitoring: Query performance insights enabled

**Document Storage:**
- Cloud object storage (AWS S3/GCS/Azure Blob Storage) for document files
- Bucket structure: `documents/{company_ticker}/{period}/{document_type}/{filename}`
- Lifecycle policy: No expiration (retain all documents)
- Access control: Private buckets; pre-signed URLs for download (expires in 1 hour)
- Versioning enabled for audit trail

**Background Worker:**
- Separate Docker container running polling daemon
- Scheduled via cloud cron service:
  - **AWS**: EventBridge rule triggers ECS task at 2:00 AM IST daily
  - **GCP**: Cloud Scheduler triggers Cloud Run job at 2:00 AM IST daily
  - **Azure**: Azure Functions with timer trigger at 2:00 AM IST daily
- Writes directly to database and S3; no API calls
- Error notifications: Send alerts to monitoring system on failure

**Secrets Management:**
- API keys stored in cloud secrets manager (AWS Secrets Manager/GCP Secret Manager/Azure Key Vault)
- Database credentials rotated quarterly (manual rotation for MVP; automatic in Phase 2)
- JWT signing keys: RSA keypair (private key in secrets manager, public key in API server)
- Environment-specific secrets: Development, staging, production isolated

**Infrastructure as Code:**
- Terraform or CloudFormation for infrastructure provisioning
- Version-controlled infrastructure definitions in Git repository
- Separate environments: Development, staging, production

**Monitoring & Alerting:**
- Application performance monitoring (APM): DataDog, New Relic, or cloud-native (CloudWatch/Cloud Monitoring)
- Metrics tracked:
  - API response times (p50, p95, p99)
  - Error rates (4xx, 5xx)
  - Database query performance
  - Background job success/failure rates
- Alerts configured for:
  - API uptime <99%
  - Error rate >1%
  - Database connection failures
  - Background polling failures

**CI/CD Pipeline:**
- GitHub Actions, GitLab CI, or cloud-native (AWS CodePipeline/GCP Cloud Build)
- Pipeline stages:
  1. **Lint**: ESLint (frontend), Pylint/Black (backend)
  2. **Test**: Unit tests, integration tests
  3. **Build**: Docker image creation, frontend asset compilation
  4. **Deploy**: Push to container registry, update ECS/Cloud Run service
- Deployment strategy: Rolling update (zero downtime)
- Rollback capability: Previous Docker image version retained

### **Development Environment**

**Local Development Setup:**
- Docker Compose orchestrating all services:
  - Frontend (React/Vue dev server with hot reload)
  - Backend API (Flask/FastAPI/Express with auto-reload)
  - PostgreSQL database container
  - Redis container (for background job queue)
  - MinIO container (S3-compatible object storage for local development)
- Mock external API responses for faster development (no real API calls)
- Seed data script: Pre-loads 50 sample companies with financial metrics and documents
- Environment variables in `.env` file (not committed to Git)

**Development Database:**
- PostgreSQL container with volume mount for persistence
- Initial schema created via migration scripts (Alembic/Django migrations/Knex)
- Test data generator script for realistic development scenarios

**API Documentation:**
- OpenAPI (Swagger) specification for all REST endpoints
- Interactive API documentation at `/docs` endpoint (Swagger UI)
- Generated client libraries for frontend (optional, but recommended)

**Version Control:**
- Git repository with branching strategy:
  - `main`: Production-ready code
  - `develop`: Integration branch for features
  - Feature branches: `feature/<name>`
- Pull request workflow with code review required before merge

### **Security Considerations**

**Network Architecture:**
- Virtual Private Cloud (VPC) with private subnets for database and backend
- API Gateway or Application Load Balancer as public entry point (HTTPS only)
- Database accessible only from backend API server security group (no public IP)
- Object storage bucket policies restrict access to authenticated API server role

**Compliance:**
- No PII (Personally Identifiable Information) stored in MVP (single user, no registration)
- Financial data is public information (SEC filings, stock prices)
- GDPR/CCPA not applicable for MVP (single user); addressed in Phase 2 multi-user

**Disaster Recovery:**
- Database backups stored in separate region for disaster recovery
- Recovery Time Objective (RTO): 4 hours
- Recovery Point Objective (RPO): 24 hours (last daily backup)
- Manual restore process documented in runbook

## **9. Document Changes from v1.3 to v1.4**

### **Major Changes**

1. **Architecture Paradigm Shift: Desktop App → Cloud-Hosted Web Application**
   - Changed from single-user desktop application to client-server web architecture
   - Frontend: Browser-based SPA (React/Vue.js)
   - Backend: RESTful API with JWT authentication
   - Database: Centralized cloud database (PostgreSQL/MySQL on AWS RDS/GCP Cloud SQL)
   - Reasoning: Device-agnostic access, data resilience, scalability, security

2. **Terminology Clarification: "Local Storage"**
   - **Old meaning (v1.3)**: Ambiguous - could be interpreted as device-local storage
   - **New meaning (v1.4)**: "Cached data" = data persisted in centralized cloud database (not fetched from live APIs)
   - All documents, metrics, verdicts stored in cloud (not on user's device)
   - Reasoning: Eliminate confusion, establish cloud-first data philosophy

3. **Authentication & Security Enhancements**
   - Added JWT-based API authentication (access tokens, refresh tokens)
   - Comprehensive security requirements: encryption at rest, secrets management, rate limiting
   - All API endpoints require Bearer token (except /login and landing page)
   - Reasoning: Secure internet-facing API, foundation for multi-user Phase 2

4. **Background Services Architecture**
   - Server-side daemon for nightly polling (2:00 AM IST)
   - Independent background service (not user-triggered)
   - Fresh data ready when user logs in (no wait time)
   - Reasoning: Improved user experience, separation of concerns

5. **New Section: System Architecture Overview (FR-DATA-000)**
   - Explicit definition of client-server architecture
   - Data storage philosophy: cloud-first, no device-local storage
   - Component diagram: Frontend, Backend API, Database, Background Services
   - Reasoning: Remove architectural ambiguity, align stakeholders

6. **New Section: Deployment Architecture (Section 8)**
   - Detailed production environment specifications
   - Infrastructure components: CDN, container service, managed database, object storage
   - Development environment setup (Docker Compose)
   - CI/CD pipeline, monitoring, secrets management
   - Reasoning: Enable Phase Zero infrastructure planning, clarify operational requirements

7. **Scalability & Storage Updates**
   - Database sizing: <1GB (v1.3) → ~50GB (v1.4) to accommodate cloud document storage
   - Document storage: BLOBs in database OR S3/GCS object storage
   - Backend API: Single instance MVP with horizontal scaling design for Phase 2
   - Reasoning: Realistic cloud infrastructure planning

8. **Multi-Window Management (FR-ANL-003)**
   - Changed from "windows" to "browser tabs/windows"
   - Added optimistic locking for concurrent edit conflict prevention
   - Client-side state vs. server-side persistence distinction
   - Reasoning: Clarify web application behavior vs. desktop application

### **Minor Updates from v1.3**

9. **Complete Pane 2 Redesign (from v1.2 → v1.3)**
   - Removed: LLM-powered 8-section analysis, interactive chat, context compaction
   - Added: Document Repository Grid with multi-source document retrieval
   - Reasoning: Cost efficiency, user control, offline analysis flexibility

2. **Pane 3 Verdict Recording Updates**
   - Removed: Auto-fill of summary field from Pane 2 LLM analysis
   - Added: Blank-by-default summary, optional analysis report upload
   - Reasoning: Supports external LLM analysis (Perplexity) workflow

3. **New Section: Document Retrieval & Source Priority**
   - Detailed logic for deduplication, multi-source handling, format priority
   - MVP scope limited to SEC Edgar (US), NSE/BSE (India)

4. **New Section: Download Workflow**
   - Save As dialog workflow with pre-filled filenames
   - Bulk download with checkbox/radio button selection
   - No download tracking (simplified MVP)

5. **Updated Data Infrastructure Section**
   - Phased API provider rollout
   - Earnings announcement polling logic
   - Document storage and refresh thresholds

### **Deferred Features (Future Phases)**

- LLM-powered analysis moved to Phase 1.5
- Markdown conversion deferred to Phase 1.5
- Interactive chat interface deferred to Phase 1.5
- Email notifications deferred to Phase 1.5
- Multi-user support deferred to Phase 2

### **Minor Updates**

- Period format clarified (YYYY-MM format with Q/FY notation)
- Currency conversion logic expanded and clarified
- Results Tracker archival rules added
- Error handling and user feedback section expanded

***

## **10. Acceptance Criteria**

### **Screener Module**

- [ ] User can create screener with market cap + sector + momentum filters
- [ ] Screener execution returns matching companies with calculated metrics
- [ ] Results show analysis status (Not Analyzed / Analyzed [date] / New Results)
- [ ] Clicking company opens Analyzer in new window

### **Analyzer - Pane 1 (Metrics)**

- [ ] All metrics display with correct formulas and units
- [ ] Currency toggle switches between local and USD (with FX conversion)
- [ ] Period toggle switches quarterly/annual display
- [ ] Period range dropdown affects displayed columns (4-10 periods)
- [ ] Heat maps apply correct color gradients to designated metrics

### **Analyzer - Pane 2 (Documents)**

- [ ] Document grid displays with proper column (time periods) and row (document types) structure
- [ ] Period columns align with Pane 1 period range selector
- [ ] Documents auto-retrieved from SEC Edgar (US) and NSE/BSE (India)
- [ ] File format icons display for available documents
- [ ] Tooltip shows source, file size, retrieval date on hover
- [ ] Multiple documents stack vertically; multiple formats display horizontally
- [ ] Empty cells show "+Upload" button for manual document upload
- [ ] Clicking file icon initiates Save As download dialog
- [ ] Bulk download feature allows selecting multiple rows/columns
- [ ] Row drag-and-drop reordering works and persists per user
- [ ] "Other Documents" row is expandable showing additional files

### **Analyzer - Pane 3 (Verdict)**

- [ ] All six assessment parameters display as enumerated selections
- [ ] Free-text summary field is blank by default
- [ ] Summary field accepts 200-500 characters
- [ ] Optional "Upload Analysis Report" button works
- [ ] Auto-populated metadata (date, quarter, version) displays correctly
- [ ] User can save partial verdicts (subset of parameters)
- [ ] Version history displays all edits with timestamps and diffs
- [ ] Closing window with no verdict shows warning prompt
- [ ] All verdict data persists to database

### **Results Tracker**

- [ ] Earnings announcement dates poll nightly for Active companies
- [ ] Table displays companies sorted by status (Announced + Not Analyzed first)
- [ ] Clicking company opens Analyzer
- [ ] Archival toggle marks companies as Archived/Active
- [ ] Filters by date range, exchange, analysis status, lifecycle
- [ ] Results update upon verdict save in Analyzer

### **Data Infrastructure**

- [ ] Market data refreshes daily or upon new results announcement
- [ ] Currency conversions use latest daily FX rates
- [ ] All documents stored locally in database
- [ ] API failures gracefully degrade with cached data display
- [ ] Error messages inform user of missing data and suggest actions

### **General**

- [ ] Multi-window support: Multiple Analyzer windows open simultaneously
- [ ] Search autocomplete finds companies across US/India
- [ ] Authentication login/logout works
- [ ] Left navigation collapses/expands
- [ ] No crashes or unhandled exceptions during normal use
- [ ] Data persists across sessions

---

**End of PRD v1.4**

Prepared by: Product Requirements Team
Date: January 16, 2026
Status: Ready for Phase Zero Technical Architecture Review