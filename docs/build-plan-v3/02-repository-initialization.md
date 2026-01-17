# Section 2: Repository Initialization

**Time Required**: ~30 minutes  
**Difficulty**: Easy  
**Goal**: Create GitHub repository, clone locally, set up initial project structure, **minimum CI**, and **Antigravity configuration**

---

## Overview

You'll create the repository on GitHub first, then clone it locally. This ensures:
- Remote backup from day 1
- Proper `.gitignore` files
- Branch protection ready
- **CI running from the first commit**
- **Antigravity knows your project structure**

---

## Step-by-Step

### Step 2.1: Create GitHub Repository

1. Go to [github.com](https://github.com)
2. Sign in (or create account if needed)
3. Click the green **"New"** button (top left, or press `+` icon)
4. Fill in:
   - **Repository name**: `iap-alpha`
   - **Description**: `Investment Analysis Platform - Alpha: AI-built investment research platform for systematic equity analysis`
   - **Visibility**: Public (you can make it private later)
   - **Initialize with README**: ✅ Check this box
   - **Add .gitignore**: Select "Rust" from dropdown
   - **Choose a license**: MIT (or your preference)
5. Click **"Create repository"**

> [!NOTE]
> **Repository Name**: `iap-alpha` stands for **Investment Analysis Platform - Alpha**. This is the alpha/MVP version of the platform.

---

### Step 2.2: Clone Repository Locally

Open terminal (Ctrl+Alt+T) and run:

```bash
# Navigate to where you want the project
cd ~/Documents

# Clone the repository (replace YOUR_USERNAME)
git clone https://github.com/YOUR_USERNAME/iap-alpha.git

# Enter the project folder
cd iap-alpha

# Open in VS Code
code .
```

---

### Step 2.3: Set Up Branch Strategy

Create the `develop` branch for active development:

```bash
# Create and switch to develop branch
git checkout -b develop

# Push develop branch to GitHub
git push -u origin develop
```

Your branches:
- `main` - Production-ready code, tagged releases
- `develop` - Active development, features merge here

---

### Step 2.4: Create Antigravity Configuration

> [!IMPORTANT]
> This is a **Day 1 requirement**. Create this file before any coding to ensure Antigravity understands your project structure.

**Copy-paste this prompt into Antigravity:**

---

#### 📋 PROMPT 2.4.1: Create Antigravity Configuration File

```
Create `.antigravityconfig.md` in the project root with project configuration for AI development.

The file should contain:

# Investment Analysis Platform (iap-alpha) - Antigravity Configuration

## Project Type
Full-stack web application:
- Backend: Rust + Axum
- Frontend: React + TypeScript + Vite
- Database: PostgreSQL
- Infrastructure: AWS ECS Fargate + Pulumi TypeScript

## Key Architecture Files
Always reference these when implementing features:
- @[docs/architecture-design-v3.md] - System design and decisions
- @[docs/prd-v1-4.md] - Product requirements
- @[docs/database-design-v1.md] - Database schema

## Code Conventions
### Rust
- Follow Clippy lints (cargo clippy -- -D warnings)
- Use thiserror for error types
- Use tracing for logging (not println!)
- Never use .unwrap() in production code
- Tests go in the same file under #[cfg(test)]

### TypeScript
- Strict mode enabled
- Use Shadcn/UI components
- Use TanStack Query for data fetching
- Use react-hook-form + zod for forms

## Critical Commands
- `cargo fmt --all` - Format Rust code
- `cargo clippy --all-targets` - Lint Rust code
- `cargo test --workspace` - Run Rust tests
- `npm test` - Run frontend tests
- `docker compose up -d` - Start development environment

## Do NOT Modify
- `golden-copy/` folder (test fixtures)
- Migration files once committed (create new migrations instead)
- Public API endpoints without updating tests and OpenAPI spec

## Test Requirements
- All new endpoints must have integration tests
- All business logic must have unit tests
- Target 80% coverage on core calculations

## Authentication
- RS256 JWT only (asymmetric, RSA keys)
- Access tokens: 24 hours
- Refresh tokens: 30 days
- Never log passwords or tokens
```

**Verification**: File exists at `.antigravityconfig.md` in project root.

---

### Step 2.5: Create Project Directory Structure

**Copy-paste this prompt into Antigravity:**

---

#### 📋 PROMPT 2.5.1: Create Project Directory Structure

```
Create the initial project directory structure for the Investment Analysis Platform (iap-alpha).

Create these directories:
- backend/api/src/routes
- backend/api/src/middleware
- backend/core/src/domain
- backend/core/src/services
- backend/core/src/ports
- backend/core/src/periods
- backend/core/src/metrics
- backend/db/src/repositories
- backend/db/src/models
- backend/db/migrations
- backend/worker/src/jobs
- backend/providers/src/alpha_vantage
- backend/providers/src/mock
- frontend/src/api
- frontend/src/components/ui
- frontend/src/components/layout
- frontend/src/components/screener
- frontend/src/components/analyzer
- frontend/src/components/tracker
- frontend/src/hooks
- frontend/src/pages
- frontend/src/stores
- frontend/src/lib
- frontend/src/types
- frontend/public
- infra
- tests/integration
- tests/e2e
- .github/workflows
- docs

Create a placeholder .gitkeep file in each empty directory to ensure they're tracked by git.
```

**Verification**: Run `tree -d -L 3` to see the directory structure.

---

#### 📋 PROMPT 2.5.2: Create Root Configuration Files

```
Create the following root-level configuration files for the Investment Analysis Platform (iap-alpha).

1. Create `.gitignore` with comprehensive ignore patterns for:
   - Rust: target/
   - **DO NOT ignore Cargo.lock** (we need it for reproducibility)
   - Node: node_modules/, dist/, .next/
   - IDE: .vscode/ (except settings), .idea/
   - Environment: .env, .env.local, .env.*.local
   - OS: .DS_Store, Thumbs.db
   - Docker: volumes and data
   - Logs and temporary files
   - Secrets: secrets/, *.pem (except public keys)

2. Create `.env.example` with placeholder values for:
   - DATABASE_URL=postgres://postgres:dev@localhost:5432/irp_dev
   - JWT_PRIVATE_KEY_FILE=./secrets/private_key.pem
   - JWT_PUBLIC_KEY_FILE=./secrets/public_key.pem
   - ALPHA_VANTAGE_API_KEY=your-api-key
   - RUST_LOG=info
   - S3_ENDPOINT=http://localhost:9000
   - S3_ACCESS_KEY=minioadmin
   - S3_SECRET_KEY=minioadmin
   - ENVIRONMENT=development

3. Create a root `README.md` with:
   - Project title: Investment Analysis Platform (iap-alpha)
   - Full name expansion: Investment Analysis Platform - Alpha
   - Quick start instructions (Docker)
   - Link to docs folder
   - Technology stack summary

4. Create `.editorconfig` for consistent formatting:
   - 4 spaces for Rust, 2 spaces for TypeScript
   - LF line endings
   - Insert final newline
```

> [!WARNING]
> Note: We intentionally **do not ignore `Cargo.lock`**. For application repositories (not libraries), keeping `Cargo.lock` in git ensures reproducible builds across machines and CI.

**Verification**: Check that files exist with `ls -la`.

---

### Step 2.6: Create Minimum CI Workflow

> [!IMPORTANT]
> Setting up CI early prevents compounding drift. Every push will now run basic checks.

---

#### 📋 PROMPT 2.6.1: Create Minimum CI Workflow

```
Create a minimal GitHub Actions CI workflow that runs on every push and PR.

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  check-backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: rustfmt, clippy
      
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend
      
      - name: Check formatting
        run: cd backend && cargo fmt --all -- --check
      
      - name: Run clippy
        run: cd backend && cargo clippy --all-targets -- -D warnings
      
      - name: Run tests
        run: cd backend && cargo test --workspace
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/test

  check-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json
      
      - name: Install dependencies
        run: cd frontend && npm ci
      
      - name: Check formatting
        run: cd frontend && npx prettier --check .
      
      - name: Run ESLint
        run: cd frontend && npm run lint
      
      - name: Run tests
        run: cd frontend && npm test -- --passWithNoTests
```

This minimal CI will:
- Run on every push to main/develop
- Check Rust formatting and lints
- Run Rust tests
- Check frontend formatting and lints
- Run frontend tests (once we have them)

The workflow will initially pass with the placeholder code and grow with the project.
```

**Verification**: Push to GitHub and check Actions tab for green checkmark.

---

### Step 2.7: Copy Existing Documentation

If you created the project in a different folder initially, copy your docs over:

```bash
# Skip if you're starting fresh
cp -r /path/to/old-project/docs/* ./docs/
```

---

### Step 2.8: Git Checkpoint

Time to commit your initial structure:

```bash
# Stage all files
git add .

# Commit with descriptive message
git commit -m "chore: initial project structure with CI

- Created monorepo directory structure
- Added configuration files (.gitignore, .editorconfig, .env.example)
- Added README with quick start guide
- Created .antigravityconfig.md for AI context
- Added minimum CI workflow (fmt, clippy, test)
- Folder structure follows architecture-design-v3.md"

# Push to develop
git push origin develop
```

After pushing, check GitHub Actions to verify CI runs successfully.

---

## Verification Checklist

After completing this section, verify:

- [ ] GitHub repository exists and is accessible
- [ ] `develop` branch created and pushed
- [ ] `.antigravityconfig.md` exists in project root
- [ ] Directory structure created (run `tree -d -L 3`)
- [ ] `.gitignore` does NOT ignore `Cargo.lock`
- [ ] `.env.example` uses RS256 key file paths (not JWT_SECRET)
- [ ] `.github/workflows/ci.yml` exists
- [ ] GitHub Actions shows green checkmark (or yellow if still building)
- [ ] `README.md` exists and is readable
- [ ] Commit pushed to GitHub

---

## Directory Structure Reference

After completion, your structure should look like:

```
iap-alpha/
├── backend/
│   ├── api/
│   │   └── src/
│   │       ├── routes/
│   │       └── middleware/
│   ├── core/
│   │   └── src/
│   │       ├── domain/
│   │       ├── services/
│   │       ├── ports/
│   │       ├── periods/
│   │       └── metrics/
│   ├── db/
│   │   ├── src/
│   │   │   ├── repositories/
│   │   │   └── models/
│   │   └── migrations/
│   ├── worker/
│   │   └── src/
│   │       └── jobs/
│   └── providers/
│       └── src/
│           ├── alpha_vantage/
│           └── mock/
├── frontend/
│   └── src/
│       ├── api/
│       ├── components/
│       │   ├── ui/
│       │   ├── layout/
│       │   ├── screener/
│       │   ├── analyzer/
│       │   └── tracker/
│       ├── hooks/
│       ├── pages/
│       ├── stores/
│       ├── lib/
│       └── types/
├── infra/
├── tests/
│   ├── integration/
│   └── e2e/
├── docs/
├── .github/
│   └── workflows/
│       └── ci.yml
├── .antigravityconfig.md    ← NEW: AI context file
├── .gitignore
├── .env.example
├── .editorconfig
└── README.md
```

---

## Next Step

**Proceed to**: [03-docker-development-environment.md](./03-docker-development-environment.md)

---

## Troubleshooting

### Git push asks for username/password
```bash
# Use SSH instead of HTTPS
git remote set-url origin git@github.com:YOUR_USERNAME/iap-alpha.git

# Or set up credential helper
git config --global credential.helper store
```

### Permission denied (publickey)
You need to set up SSH keys:
```bash
# Generate SSH key
ssh-keygen -t ed25519 -C "your.email@example.com"

# Add to ssh-agent
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519

# Copy public key
cat ~/.ssh/id_ed25519.pub
# Paste this into GitHub → Settings → SSH Keys → New SSH Key
```

### Branch doesn't exist error
```bash
# Make sure you're on the right branch
git branch
git checkout develop
```

### CI is failing
```bash
# Check what's failing in GitHub Actions
# Common issues:
# - Formatting: run `cargo fmt --all` locally
# - Clippy: run `cargo clippy --all-targets` and fix warnings
# - Tests: run `cargo test --workspace` locally
```
