## Controls Bar Visual Layout Guide

### Full Layout (Pinned / Visible State)

```
╔════════════════════════════════════════════════════════════════════════════════════╗
║  [X]   Microsoft Corporation                                                       ║
║        MSFT • NASDAQ                                                                ║
║                                                                                     ║
║              ┌─────────┬─────────┐     PERIODS  ┌──────▼──┐                       ║
║              │Quarterly│ Annual  │               │    8    │                        ║
║              └─────────┴─────────┘               └─────────┘                        ║
║                                                                                     ║
║                                                    🔄  Refresh Data    │   📌      ║
╚════════════════════════════════════════════════════════════════════════════════════╝
```

### Detailed Element Breakdown

#### LEFT SECTION
```
┌──────────────────────────────────┐
│  [X]  Microsoft Corporation      │  ← Close button + Company Name (bold)
│       MSFT • NASDAQ               │  ← Symbol • Exchange (small, gray)
└──────────────────────────────────┘
```

#### CENTER SECTION - Period Type Toggle
```
┌─────────────────────────┐
│ ┌─────────┬──────────┐  │
│ │Quarterly│  Annual  │  │  ← Tabs component
│ └─────────┴──────────┘  │     (Active = blue bg, white text)
└─────────────────────────┘
```

#### CENTER SECTION - Period Count
```
┌──────────────────┐
│ PERIODS  [8 ▼]   │  ← Label + Dropdown
└──────────────────┘
   (Dropdown opens showing: 4, 5, 6, 7, 8, 9, 10)
```

#### RIGHT SECTION
```
┌────────────────────────────┐
│  🔄  Refresh Data  │  📌   │  ← Refresh button | Divider | Pin button
└────────────────────────────┘
   (Pin button: gray = unpinned, blue = pinned)
```

### Auto-Hide State (Unpinned, Mouse Away)

```
                                                                                        
╔════════════════════════════════════════════════════════════════════════════════════╗
║                                    ▼ ▼ ▼                                            ║  ← Only 4px visible
╚════════════════════════════════════════════════════════════════════════════════════╝
                                     ────                                                ← Hover trigger tab
```

### Component Dimensions

```
┌─────────────────────────────────────────────────────────────┐
│  Height: 56px (h-14)                                         │
│  Width: 100% (full width, max-w-1600px container)            │
│  Position: fixed top-0 (sticky at top)                       │
│  Z-index: 50 (above other content)                           │
└─────────────────────────────────────────────────────────────┘
```

### Color Scheme

```
Background:
  Light mode: rgba(255, 255, 255, 0.8) + backdrop-blur-xl
  Dark mode:  rgba(15, 23, 42, 0.8) + backdrop-blur-xl

Text Colors:
  Primary (company name):  slate-900 / white
  Secondary (symbol):      slate-500 / slate-400
  Interactive elements:    slate-600 → blue-600 on hover

Borders:
  Bottom border:           slate-200 / slate-800
  Shadow:                  subtle sm shadow
```

### States & Interactions

#### 1. Pin Button States
```
UNPINNED (default):
  ┌────┐
  │ 📌 │  Gray pin icon
  └────┘  Transparent background
  
PINNED:
  ┌────┐
  │ 📌 │  Pin-off icon
  └────┘  Blue background (bg-blue-50)
```

#### 2. Period Type Toggle States
```
QUARTERLY ACTIVE:
  ┌─────────┬──────────┐
  │Quarterly│  Annual  │  ← Blue bg on left
  └─────────┴──────────┘

ANNUAL ACTIVE:
  ┌─────────┬──────────┐
  │Quarterly│  Annual  │  ← Blue bg on right
  └─────────┴──────────┘
```

#### 3. Period Dropdown States
```
CLOSED:
  ┌──────▼──┐
  │    8    │
  └─────────┘

OPEN:
  ┌──────▲──┐
  │    8    │  ← Currently selected
  ├─────────┤
  │    4    │
  │    5    │
  │    6    │
  │    7    │
  │  →  8   │  ← Highlighted
  │    9    │
  │   10    │
  └─────────┘
```

### Animations

```
SLIDE DOWN (show):
  [Hidden]   transform: translateY(-calc(100% - 4px))  opacity: 0
     ↓       transition: 300ms ease-in-out
  [Visible]  transform: translateY(0)                  opacity: 100

SLIDE UP (hide):
  [Visible]  transform: translateY(0)                  opacity: 100
     ↓       transition: 300ms ease-in-out
  [Hidden]   transform: translateY(-calc(100% - 4px))  opacity: 0
```

### Hover Trigger Area

```
When hidden (unpinned):

    Only 4px of bar visible at top edge
                    ↓
  ═══════════════════════════════════════
                    ▼
  ┌─────────────────┐
  │      ────       │  ← Rounded tab (8px width)
  └─────────────────┘     hover area to reveal bar
          ↓
    Move mouse here to show controls
```

### Responsive Behavior

```
DESKTOP (≥1280px):
  ┌────────────────────────────────────────────────────────┐
  │ [X] Company Name    [Q][A] PERIODS [8]   🔄 Refresh │📌│
  └────────────────────────────────────────────────────────┘

TABLET/MOBILE:
  ┌──────────────────────────────────────┐
  │ [X] Company    [Q][A] [8]   🔄  │📌 │  ← Text truncates
  └──────────────────────────────────────┘
```

### Accessibility Features

```
All Interactive Elements:
  - Keyboard focusable
  - Tab navigation support
  - Clear focus indicators
  - ARIA labels where needed

Tooltips:
  📌 Pin: "Pin controls" / "Unpin controls"
  [X] Close: Implicit close action
  🔄 Refresh: "Refresh Data" label visible

Color Contrast:
  - Text: 4.5:1 minimum ratio
  - Interactive elements: visible hover states
  - Focus indicators: 3:1 minimum ratio
```

### Technical Implementation Details

```
Component Structure:
  <header>                                 ← Main container
    {/* Hover trigger (when unpinned) */}
    <div className="hover-trigger">       ← Small tab at bottom
    
    <div className="container">           ← Content container
      <div className="left">              ← Company info
      <div className="center">            ← Controls
      <div className="right">             ← Actions
    </div>
  </header>

State Management:
  - isPinned: boolean (localStorage)
  - isVisible: boolean (hover state)
  - periodType: string (parent state)
  - periodCount: number (parent state)

Event Handlers:
  - onMouseEnter → setIsVisible(true)
  - onMouseLeave → setIsVisible(false)
  - onClick (pin) → togglePinned()
  - onChange (period type) → onPeriodTypeChange()
  - onChange (period count) → onPeriodCountChange()
  - onClick (refresh) → onRefresh()
  - onClick (close) → onClose()
```

---

## How to Identify Each Element in Browser

When you open the Analyzer page, look for these specific visual cues:

1. **Close Button**: Round button with X icon, leftmost element
2. **Company Name**: Bold, larger text, dark color
3. **Symbol**: Smaller, gray, uppercase text below company name
4. **Period Toggle**: Two connected buttons (pills), one with blue background
5. **Period Dropdown**: Small box with number and down arrow
6. **Refresh Button**: Icon that looks like circular arrows + "Refresh Data" text
7. **Divider**: Thin vertical line between refresh and pin
8. **Pin Button**: Icon that looks like a thumbtack/pin
9. **Hover Trigger**: Small rounded tab below the bar when it's hidden

---

**This visual guide matches the implementation in ControlsBar.tsx**
