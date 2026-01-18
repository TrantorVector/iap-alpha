# Analyzer Page URL Guide

## Issue

You navigated to `http://localhost:3000/analyzer/AAPL` but the analyzer expects a **company UUID**, not a symbol.

## Solution

Use the company UUID instead:

```
http://localhost:3000/analyzer/10000000-0000-0000-0000-000000000001
```

## Quick Reference: Company IDs

| Symbol | Company ID |
|--------|------------|
| AAPL | `10000000-0000-0000-0000-000000000001` |
| MSFT | `10000000-0000-0000-0000-000000000002` |
| JPM | `10000000-0000-0000-0000-000000000003` |
| JNJ | `10000000-0000-0000-0000-000000000004` |
| TSLA | `10000000-0000-0000-0000-000000000005` |

## Complete URLs for Testing

### Apple (AAPL)
```
http://localhost:3000/analyzer/10000000-0000-0000-0000-000000000001
```

### Microsoft (MSFT)
```
http://localhost:3000/analyzer/10000000-0000-0000-0000-000000000002
```

### Tesla (TSLA)
```
http://localhost:3000/analyzer/10000000-0000-0000-0000-000000000005
```

## Why Does It Work This Way?

The analyzer route is defined as `/analyzer/:companyId` where `companyId` is expected to be a UUID from the database, not a stock symbol. This is because:

1. **UUIDs are unique** - they never change
2. **Symbols can be reused** - companies can be delisted and symbols reused
3. **Database design** - the primary key is the UUID

## Future Enhancement

If you'd like to support symbol-based URLs (e.g., `/analyzer/AAPL`), we could add a redirect or resolver that:

1. Looks up the symbol in the database
2. Gets the UUID
3. Redirects to `/analyzer/{UUID}`

This would be implemented in the frontend router or as a backend endpoint.
