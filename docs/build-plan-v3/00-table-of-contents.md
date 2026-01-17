# Investment Analysis Platform (iap-alpha) - Build Plan v3

**Generated**: January 17, 2026  
**Architecture Reference**: [architecture-design-v3.md](../architecture-design-v3.md)  
**Target**: AI-Built Platform using Antigravity

> [!NOTE]
> **Repository Name**: `iap-alpha` (Investment Analysis Platform - Alpha)
> 
> This is the alpha/MVP version of the Investment Analysis Platform for systematic equity analysis across US and Indian markets.

---

## About This Build Plan

This build plan is designed for a **non-technical founder** who will build the entire platform by prompting Antigravity. Each section contains:

- **Objective**: What you're building
- **Prerequisites**: What must be done before starting
- **Prompts**: Copy-paste ready prompts for Antigravity
- **Verification**: How to confirm things worked (tests + visual checks)
- **Git Checkpoint**: When and how to commit

### Key Principles

1. **Granular prompts** - Each prompt does ONE thing well
2. **Fail fast** - Small changes = easy rollbacks
3. **Local-first** - Build everything locally with Docker before touching AWS
4. **Test-driven** - Antigravity writes tests, you just confirm they pass
5. **CI Early** - Basic linting/testing from Week 1 to catch drift

> [!IMPORTANT]
> **Day 1 Requirement**: Before any coding, create `.antigravityconfig.md` in your project root. This file tells Antigravity about key project files, conventions, and commands. See Section 2.4.

---

## Table of Contents

### Foundation (Weeks 1-2)

| Section | File | Description |
|---------|------|-------------|
| **1** | [01-prerequisites-setup.md](./01-prerequisites-setup.md) | Install Docker, VS Code, Git on Ubuntu |
| **2** | [02-repository-initialization.md](./02-repository-initialization.md) | Create GitHub repo, clone, initial structure, **minimum CI**, **Antigravity config** |
| **3** | [03-docker-development-environment.md](./03-docker-development-environment.md) | Docker Compose setup for local dev |
| **4** | [04-database-foundation.md](./04-database-foundation.md) | PostgreSQL schema, migrations, seed data |

### Backend Core (Weeks 3-4)

| Section | File | Description |
|---------|------|-------------|
| **5** | [05-backend-core.md](./05-backend-core.md) | Rust workspace, Axum server, health checks, **OpenAPI generation** |
| **6** | [06-authentication-system.md](./06-authentication-system.md) | RS256 JWT auth, login/logout, middleware |

### Analyzer Module (Weeks 5-7) - Priority 1

| Section | File | Description |
|---------|------|-------------|
| **7** | [07-analyzer-module-backend.md](./07-analyzer-module-backend.md) | Metrics API, documents API, verdicts API |
| **8** | [08-analyzer-module-frontend.md](./08-analyzer-module-frontend.md) | React UI for Pane 0-3, data visualization |

### Screener Module (Weeks 8-9) - Priority 2

| Section | File | Description |
|---------|------|-------------|
| **9** | [09-screener-module.md](./09-screener-module.md) | Full screener: backend + frontend |

### Background Jobs (Weeks 10-11) - Priority 3

| Section | File | Description |
|---------|------|-------------|
| **10** | [10-background-jobs.md](./10-background-jobs.md) | Worker binary, nightly jobs, mock providers |

### Results Tracker (Week 12) - Priority 4

| Section | File | Description |
|---------|------|-------------|
| **11** | [11-results-tracker.md](./11-results-tracker.md) | Verdict history, performance tracking |

### Quality & Deployment (Weeks 13-14)

| Section | File | Description |
|---------|------|-------------|
| **12** | [12-testing-cicd.md](./12-testing-cicd.md) | E2E tests, expand CI/CD for deployment |
| **13** | [13-cloud-deployment.md](./13-cloud-deployment.md) | AWS account setup, Pulumi TypeScript IaC, deployment |

### Reference

| Section | File | Description |
|---------|------|-------------|
| **14** | [14-antigravity-best-practices.md](./14-antigravity-best-practices.md) | Antigravity optimization, context management |

---

## How to Use This Build Plan

### Your Daily Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│                     DAILY DEVELOPMENT LOOP                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. START DOCKER                                                │
│     └──→ Open terminal, run: docker-compose up -d              │
│                                                                 │
│  2. OPEN ANTIGRAVITY                                            │
│     └──→ Open VS Code in project folder                        │
│                                                                 │
│  3. PICK NEXT PROMPT                                            │
│     └──→ Follow build plan sections in order                   │
│     └──→ Copy prompt, paste into Antigravity                   │
│                                                                 │
│  4. WAIT FOR COMPLETION                                         │
│     └──→ Antigravity runs tests automatically                  │
│     └──→ If tests fail, Antigravity fixes them                 │
│                                                                 │
│  5. VISUAL INSPECTION (when prompted)                           │
│     └──→ Open browser: http://localhost:3000                   │
│     └──→ Click through the feature                             │
│     └──→ Confirm it looks right                                │
│                                                                 │
│  6. GIT COMMIT (at checkpoints)                                 │
│     └──→ Follow the commit message provided                    │
│     └──→ Push to GitHub                                        │
│     └──→ CI checks run automatically                           │
│                                                                 │
│  7. STOP DOCKER (end of day)                                    │
│     └──→ Run: docker-compose down                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### When Things Go Wrong

| Problem | Solution |
|---------|----------|
| Docker won't start | Run `docker system prune` then try again |
| Tests keep failing | Start new Antigravity chat, paste: "The tests for [feature] are failing. Please investigate and fix. Here's the error: [paste error]" |
| Feature looks broken | Start new Antigravity chat, describe what's wrong visually |
| Need to undo changes | Run `git checkout .` to discard all uncommitted changes |
| Want to go back to last commit | Run `git reset --hard HEAD` |
| CI is failing | Check GitHub Actions tab, fix issues before continuing |

### Progress Tracking

After completing each section, update this checklist:

- [ ] Section 1: Prerequisites & Setup
- [ ] Section 2: Repository Initialization (includes CI setup!)
- [ ] Section 3: Docker Development Environment
- [ ] Section 4: Database Foundation
- [ ] Section 5: Backend Core
- [ ] Section 6: Authentication System
- [ ] Section 7: Analyzer Module Backend
- [ ] Section 8: Analyzer Module Frontend
- [ ] Section 9: Screener Module
- [ ] Section 10: Background Jobs
- [ ] Section 11: Results Tracker
- [ ] Section 12: Testing & CI/CD (expand for deployment)
- [ ] Section 13: Cloud Deployment
- [ ] Section 14: Antigravity Best Practices (Reference)

---

## Estimated Timeline

| Phase | Sections | Duration | Effort |
|-------|----------|----------|--------|
| Foundation | 1-4 | 2 weeks | 3-4 hrs/day |
| Backend | 5-6 | 2 weeks | 3-4 hrs/day |
| Analyzer | 7-8 | 3 weeks | 3-4 hrs/day |
| Screener | 9 | 2 weeks | 3-4 hrs/day |
| Jobs + Tracker | 10-11 | 2 weeks | 3-4 hrs/day |
| QA + Deploy | 12-13 | 2 weeks | 3-4 hrs/day |
| **Total** | | **~13 weeks** | |

> **Note**: Timeline assumes 3-4 hours/day as you specified. Actual time may vary based on issues encountered.

---

## Changes from Build Plan v2

> [!NOTE]
> This version addresses repository naming and architecture versioning:

| Change | Rationale |
|--------|-----------|
| **Repository name: `iap-alpha`** | Matches GitHub repo name for clarity |
| **Updated all architecture references to v3** | Ensures Antigravity uses latest design decisions |
| **Added "Investment Analysis Platform - Alpha" explanation** | Preserves full name for understanding |

---

## Next Steps

**Start with**: [01-prerequisites-setup.md](./01-prerequisites-setup.md)
