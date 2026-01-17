# Section 14: Antigravity Best Practices

**Time Required**: Reference document (no prompts)  
**Goal**: Optimize your workflow with Antigravity for this AI-built project

---

## Overview

This section contains best practices for working with Antigravity effectively. Unlike other sections, this is a **reference guide** rather than a step-by-step build.

> [!IMPORTANT]
> **Prerequisite**: You should have created `.antigravityconfig.md` in Section 2. If you skipped it, go back and create it now. This file is essential for consistent AI assistance.

---

## 1. The .antigravityconfig.md File

### Why It's Critical

The `.antigravityconfig.md` file tells Antigravity about your project. Without it:
- Antigravity may use wrong conventions
- Prompts need to repeat context every time
- Mistakes are more likely

### Reference Your Config in Prompts

When starting a new feature, reference the config:

```
Implement [FEATURE NAME] for the Investment Research Platform.

Reference: @[.antigravityconfig.md] for project conventions.
Reference: @[docs/architecture-design-v3.md] section [X]

Requirements:
1. [Requirement 1]
...
```

### Keeping It Updated

Update `.antigravityconfig.md` when:
- You add new coding conventions
- You change key file locations
- You add new "do not modify" rules
- You change test requirements

---

## 2. Context Window Management

### The Problem
Antigravity (like all LLMs) has a limited context window. Very long conversations or large files can cause:
- Slower responses
- Lost context from earlier messages
- Errors or incomplete outputs

### Best Practices

| Practice | Why |
|----------|-----|
| **Start fresh conversations for new features** | Clean context = better results |
| **Keep prompts focused on ONE thing** | Less confusion, easier debugging |
| **Commit frequently** | Checkpoint progress, can reference in new context |
| **Don't paste entire files** | Describe what you need, let Antigravity read files |
| **Reference by file path** | Use @[path/to/file] notation |
| **Reference .antigravityconfig.md** | Ensures consistent conventions |

### When to Start a New Conversation

✅ Start new conversation when:
- Moving to a completely new feature
- Previous conversation getting slow
- Error keeps repeating despite fixes
- You've just committed working code

❌ Stay in same conversation when:
- Fixing a bug from the last change
- Continuing the same feature
- Need context from earlier discussion

---

## 3. Prompt Engineering for Antigravity

### Effective Prompt Structure

```
[Clear goal statement]

Reference: @[.antigravityconfig.md]
Reference: @[other relevant files]

[Specific requirements as bullet points]

[Expected verification/output]
```

### Example: Good vs Bad Prompts

**❌ Bad prompt:**
```
Make the login work
```

**✅ Good prompt:**
```
Create the login API endpoint for the Investment Research Platform.

Reference: @[.antigravityconfig.md] for project conventions
Reference: @[docs/architecture-design-v3.md] section 7 for JWT configuration

Requirements:
1. POST /api/v1/auth/login
2. Accept username and password in JSON body
3. Verify password using Argon2id (already implemented in auth/password.rs)
4. Generate RS256 JWT access token (24h) and refresh token (30d)
5. Store refresh token hash in database
6. Return LoginResponse with tokens and user info
7. Return 401 for invalid credentials without revealing which field was wrong

After implementation, tests in tests/integration/auth_test.rs should pass.
```

### Prompt Templates

**For new features:**
```
Implement [FEATURE NAME] for the Investment Research Platform.

Reference: @[.antigravityconfig.md]
Reference: @[docs/architecture-design-v3.md] section [X]
Reference: @[docs/prd-v1-4.md] section [Y]

Requirements:
1. [Requirement 1]
2. [Requirement 2]
3. [Requirement 3]

Create appropriate tests. All tests should pass after implementation.
```

**For bug fixes:**
```
Fix: [DESCRIPTION OF BUG]

Reference: @[.antigravityconfig.md]

Error message: [PASTE ERROR]

Expected behavior: [WHAT SHOULD HAPPEN]
Actual behavior: [WHAT IS HAPPENING]

The issue might be in: @[path/to/file.rs]
```

**For refactoring:**
```
Refactor [COMPONENT] to [GOAL].

Reference: @[.antigravityconfig.md]

Current issues:
1. [Problem 1]
2. [Problem 2]

Keep the same external API/behavior.
Ensure all existing tests still pass.
```

---

## 4. Error Recovery Strategies

### When Tests Fail

1. **Don't tell Antigravity the tests failed in the same conversation**
2. Start new conversation with:
   ```
   The tests in @[tests/integration/auth_test.rs] are failing.
   
   Reference: @[.antigravityconfig.md]
   
   Error output:
   [PASTE ERROR]
   
   The implementation is in @[backend/api/src/routes/auth.rs]
   
   Please investigate and fix.
   ```

### When Build Fails

1. Copy the exact error message
2. Start new conversation:
   ```
   Rust compilation error:
   
   [PASTE ERROR]
   
   This happened after implementing [FEATURE].
   The file is @[path/to/file.rs]
   
   Please fix the compilation error.
   ```

### When CI Fails

1. Check GitHub Actions for specific failure
2. Start new conversation:
   ```
   CI is failing on the [check-backend/integration-tests/etc] step.
   
   Error from GitHub Actions:
   [PASTE ERROR]
   
   Please fix this issue.
   ```

### When Something "Looks Wrong"

1. Be specific about what's wrong:
   ```
   The metrics table is displaying correctly but the heat map colors are inverted.
   
   Expected: Green for high values, orange for low
   Actual: Orange for high values, green for low
   
   The component is @[frontend/src/components/analyzer/MetricsDashboard.tsx]
   The color calculation is in @[frontend/src/lib/heatmap.ts]
   
   Please fix the color inversion.
   ```

---

## 5. Git Workflow for AI-Built Projects

### Commit Message Format

```
type(scope): description

[Detail what changed and why]
```

Types: feat, fix, refactor, test, docs, chore, style

### Commit Frequency

| Milestone | Commit? |
|-----------|---------|
| New endpoint working | ✅ Yes |
| Component rendering | ✅ Yes |
| Tests passing | ✅ Yes |
| CI green | ✅ Yes |
| "Almost working" | ❌ No |
| "In progress" | ❌ No |

### Recovery Workflow

```bash
# See what changed recently
git log --oneline -5

# Undo last commit (keep changes)
git reset --soft HEAD~1

# Undo last commit (discard changes)
git reset --hard HEAD~1

# Discard all uncommitted changes
git checkout .

# Stash changes temporarily
git stash
git stash pop
```

---

## 6. Testing Strategy for AI Code

### Why Tests Are Critical

Without tests, you can't verify AI-generated code works correctly. Tests are your safety net.

### Test Commands Quick Reference

```bash
# Run all Rust tests
cargo test --workspace

# Run specific test
cargo test test_login_success

# Run with output visible
cargo test -- --nocapture

# Run frontend tests
cd frontend && npm test

# Run E2E tests
npx playwright test

# Run single E2E test
npx playwright test tests/e2e/login.spec.ts
```

### When to Run Tests

| Action | Run Tests? |
|--------|------------|
| After any code change | ✅ Yes |
| Before committing | ✅ Yes, all |
| Before starting new feature | ✅ Yes, ensure baseline |
| After pulling updates | ✅ Yes |

---

## 7. CI as Your Safety Net

### Why CI from Day 1

Since we set up CI in Section 2, every push runs:
- Formatting checks (`cargo fmt`, `prettier`)
- Linting (`clippy`, `eslint`)
- Tests (`cargo test`, `npm test`)

This catches problems before they compound.

### What to Do When CI Fails

1. **Don't ignore it** - Fix before continuing
2. **Check the specific failure** in GitHub Actions
3. **Run locally first**: 
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test --workspace
   ```
4. If local passes but CI fails, check for environment differences

---

## 8. Daily Workflow Summary

```
┌─────────────────────────────────────────────────────────┐
│                 YOUR DAILY ROUTINE                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  MORNING                                                │
│  1. docker compose up -d                                │
│  2. code .   (open VS Code)                             │
│  3. Check where you left off in build plan              │
│  4. Check CI status on GitHub                           │
│                                                         │
│  WORKING                                                │
│  5. Copy prompt from build plan                         │
│  6. Reference @[.antigravityconfig.md] in prompt        │
│  7. Paste into Antigravity chat                         │
│  8. Wait for completion                                 │
│  9. If tests mentioned, confirm they pass               │
│  10. If visual check needed, test in browser            │
│  11. git add . && git commit -m "..."                   │
│  12. git push origin develop                            │
│  13. Check CI passes                                    │
│  14. Repeat from step 5                                 │
│                                                         │
│  END OF DAY                                             │
│  15. Ensure CI is green before stopping                 │
│  16. docker compose down                                │
│  17. Update progress in build plan TOC                  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## 9. Key Files to Reference in Prompts

| Path | When to Reference |
|------|-------------------|
| `@[.antigravityconfig.md]` | **ALWAYS** - project conventions |
| `@[docs/architecture-design-v3.md]` | Any technical implementation |
| `@[docs/prd-v1-4.md]` | Feature requirements, UX details |
| `@[docs/database-design-v1.md]` | Database schema, queries |
| `@[docs/api-data-mapping.md]` | External API integrations |

---

## 10. When to Ask for Help

### Ask Antigravity for help when:
- Error message you don't understand
- Code isn't behaving as expected
- Need to understand how something works
- Want suggestions for implementation

### DON'T ask Antigravity:
- "Is this correct?" (run tests instead)
- Vague questions ("make it work")
- Questions about external services (AWS console, GitHub UI)

### Use web search for:
- AWS service documentation
- Specific library versions
- "How do I do X in AWS console"
- GitHub settings and configuration

---

## Summary

1. **Create and reference `.antigravityconfig.md`** - Your project's AI instruction manual
2. **Start fresh conversations** for new features
3. **Use specific, detailed prompts** with references
4. **Commit working code** before trying new things
5. **Run tests** after every change
6. **Keep CI green** - don't accumulate failures
7. **Visual inspect** when prompted
8. **Git is your undo button** - use it liberally

---

*End of Build Plan Reference Guide*
