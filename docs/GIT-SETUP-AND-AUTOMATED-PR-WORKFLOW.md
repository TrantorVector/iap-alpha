# Git Setup & Automated PR Workflow Guide

> **Production-Ready Guide for Headless Hetzner Server via Remote-SSH**
>
> This guide provides complete setup instructions for Git, GitHub CLI, and the automated PR workflow.

---

## Table of Contents

1. [Part 1: Git Configuration](#part-1-git-configuration)
2. [Part 2: SSH Key Setup](#part-2-ssh-key-setup)
3. [Part 3: GitHub CLI Installation](#part-3-github-cli-installation)
4. [Part 4: Repository Configuration](#part-4-repository-configuration)
5. [Part 5: Using the Automation](#part-5-using-the-automation)
6. [Troubleshooting](#troubleshooting)
7. [Appendix A: CI Workflow Template](#appendix-a-ci-workflow-template)

---

## Part 1: Git Configuration

### Step 1.1: Verify Git Installation

```bash
git --version
```

If not installed:

```bash
sudo apt update && sudo apt install git -y
```

### Step 1.2: Configure Identity

```bash
git config --global user.name "Your Full Name"
git config --global user.email "your.email@example.com"
```

> ⚠️ **Use the same email as your GitHub account.**

### Step 1.3: Verify Configuration

```bash
git config --global --list
```

### Step 1.4: Recommended Settings

```bash
git config --global init.defaultBranch main
git config --global color.ui auto
git config --global core.editor nano
git config --global pull.rebase false
```

---

## Part 2: SSH Key Setup

### Step 2.1: Check Existing Keys

```bash
ls -la ~/.ssh
```

If `id_ed25519` exists, skip to Step 2.4.

### Step 2.2: Generate SSH Key

```bash
ssh-keygen -t ed25519 -C "your.email@example.com"
```

- Press **Enter** for default location
- Enter passphrase (optional)

### Step 2.3: Add Key to SSH Agent

```bash
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519
```

### Step 2.4: Copy Public Key

```bash
cat ~/.ssh/id_ed25519.pub
```

Copy the entire output.

### Step 2.5: Add to GitHub

1. Go to: **https://github.com/settings/keys**
2. Click **New SSH key**
3. Title: "Hetzner Server" (or similar)
4. Paste your key
5. Click **Add SSH key**

### Step 2.6: Test Connection

```bash
ssh -T git@github.com
```

Expected: `Hi username! You've successfully authenticated...`

### Step 2.7: Switch Repository to SSH

```bash
cd /home/founder/iap-alpha
git remote set-url origin git@github.com:YOUR-USERNAME/iap-alpha.git
```

---

## Part 3: GitHub CLI Installation

### Step 3.1: Add Repository

```bash
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
  | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg

sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg

echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
  | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
```

### Step 3.2: Install

```bash
sudo apt update && sudo apt install gh -y
```

### Step 3.3: Verify

```bash
gh --version
```

### Step 3.4: Authenticate (Headless Device Code Flow)

> 🔴 **CRITICAL FOR HEADLESS SERVERS**
>
> Since you are on a remote server without a browser, you must use the **Device Code Flow**. This involves authorizing on your **local laptop** while the server waits.

Run:

```bash
gh auth login
```

Answer the prompts:

1. **"What account do you want to log into?"**
   → Select: `GitHub.com`

2. **"What is your preferred protocol for Git operations?"**
   → Select: `SSH`

3. **"Upload your SSH public key?"**
   → Select: `Skip` (already done in Part 2)

4. **"How would you like to authenticate GitHub CLI?"**
   → Select: `Login with a web browser`

5. **A one-time code appears (e.g., `ABCD-1234`)**

   ```
   ! First copy your one-time code: ABCD-1234
   Press Enter to open github.com in your browser...
   ```

   > ⚠️ **DO NOT PRESS ENTER YET!**

6. **On your LOCAL laptop/desktop:**
   - Open a browser
   - Go to: **https://github.com/login/device**
   - Log in to GitHub if prompted
   - Paste the one-time code from step 5
   - Click **Continue**
   - Click **Authorize github**
   - You should see: "Congratulations, you're all set!"

7. **Now return to your server terminal:**
   - Press **Enter**
   - The CLI will detect the authorization

**Expected output:**

```
✓ Authentication complete.
- gh config set -h github.com git_protocol ssh
✓ Configured git protocol
✓ Logged in as YOUR-USERNAME
```

### Step 3.5: Verify Authentication

```bash
gh auth status
```

---

## Part 4: Repository Configuration

### Step 4.1: Enable Auto-Merge (GitHub Settings)

1. Go to your repository on GitHub
2. Click **Settings** → **General**
3. Scroll to **Pull Requests**
4. Enable: **☑ Allow auto-merge**
5. Click **Save**

### Step 4.2: Configure Branch Protection

1. Go to **Settings** → **Branches**
2. Click **Add branch protection rule**
3. **Branch name pattern:** `main`
4. Enable these options:
   - ☑ **Require a pull request before merging**
   - ☑ **Require status checks to pass before merging**
   - ☑ **Require branches to be up to date before merging**

5. **Add required status checks:**

   > ⚠️ **CRITICAL: Use Job IDs, NOT Display Names**
   >
   > The "Required status checks" field expects the **Job ID** from your `ci.yml` file, not the workflow display name.
   >
   > For example, if your `ci.yml` has:
   > ```yaml
   > jobs:
   >   build_and_test:    # ← This is the JOB ID (use this!)
   >     name: "Build & Test"  # ← This is the display name (don't use this)
   > ```
   >
   > ⚠️ **IMPORTANT:** Status checks only appear in the search after CI has run at least once!
   >
   > If nothing appears, you need to create a PR first to trigger CI, then come back and add the checks.

   Your repository uses these job IDs (from `.github/workflows/ci.yml`):
   - `check-backend`
   - `check-frontend`

6. Click **Create** or **Save changes**

### Step 4.3: Ensure CI Workflow Exists

The auto-pr script requires `.github/workflows/ci.yml` to exist.

If missing, create it using the template in **Appendix A**.

---

## Part 5: Using the Automation

### Quick Start

```bash
./scripts/auto-pr.sh "Add new feature"
```

### What Happens

```
┌─────────────────────────────────────────────────────────────────────┐
│                      WORKFLOW DIAGRAM                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. Push branch & create PR                                         │
│              │                                                      │
│              ▼                                                      │
│  2. Enable auto-merge: gh pr merge --auto --squash                  │
│              │                                                      │
│              ▼                                                      │
│  ┌─────────────────────────────────────────┐                        │
│  │ SUCCESS?                                │                        │
│  │   YES → Exit! GitHub merges when        │                        │
│  │         CI passes. Terminal free.       │                        │
│  └─────────────────────────────────────────┘                        │
│              │ NO (CI already failing)                              │
│              ▼                                                      │
│  3. Extract error logs → ci-logs/                                   │
│              │                                                      │
│              ▼                                                      │
│  4. PAUSE: "Paste this into Antigravity"                            │
│              │                                                      │
│              ▼                                                      │
│  5. [Human fixes with Antigravity IDE]                              │
│              │                                                      │
│              ▼                                                      │
│  6. Press ENTER → Commit & Push → Retry auto-merge                  │
│              │                                                      │
│              └──────── Loop until success ─────────────────────────▶│
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Example: Happy Path

```bash
$ ./scripts/auto-pr.sh "Add authentication module"

╔═══════════════════════════════════════════════════════════════════════════╗
║   🚀 AUTO-PR WORKFLOW (v3.0 - Native Auto-Merge)                          ║
╚═══════════════════════════════════════════════════════════════════════════╝

[SUCCESS] GitHub CLI authenticated
[SUCCESS] CI workflow found: .github/workflows/ci.yml
[SUCCESS] On feature branch: feature/add-auth
[SUCCESS] Pushed to origin/feature/add-auth
[SUCCESS] Created PR #42

╔═══════════════════════════════════════════════════════════════════════════╗
║   ✅ AUTO-MERGE ENABLED                                                   ║
║                                                                           ║
║   PR #42 will be automatically merged when CI passes.                     ║
║                                                                           ║
║   You can close this terminal. GitHub handles the rest!                   ║
╚═══════════════════════════════════════════════════════════════════════════╝
```

### Example: Fix Loop

```bash
[WARNING] Could not enable auto-merge. Checking CI status...
[WARNING] CI checks failed

╔════════════════════════════════════════════════════════════════════════════╗
║   ❌ CI FAILED - HUMAN INTERVENTION REQUIRED                               ║
║                                                                            ║
║   1. OPEN Antigravity (your local IDE)                                     ║
║   2. PASTE THE CONTENTS OF THIS FILE:                                      ║
║      /home/founder/iap-alpha/ci-logs/ci-errors_attempt1_20260120.md        ║
║   3. ASK ANTIGRAVITY TO FIX THE ERRORS                                     ║
║   4. AFTER FIXES ARE SAVED, PRESS ENTER HERE                               ║
╚════════════════════════════════════════════════════════════════════════════╝

Press ENTER when Antigravity has completed the fix...
```

---

## Troubleshooting

### "Permission denied (publickey)"

SSH key not configured correctly.

```bash
ssh-add ~/.ssh/id_ed25519
ssh -T git@github.com
```

### "gh: command not found"

GitHub CLI not installed. Re-run Part 3.

### "CI workflow not found"

Create `.github/workflows/ci.yml` using Appendix A.

### "Could not enable auto-merge"

Check GitHub repository settings:
- Is "Allow auto-merge" enabled?
- Are branch protection rules configured?
- Do required status checks match job IDs?

### Auto-merge enabled but PR not merging

- CI might still be running
- Branch protection checks might not be passing
- Check: `gh pr checks <PR_NUMBER>`

---

## Appendix A: CI Workflow Template

If `.github/workflows/ci.yml` does not exist, create it with this template:

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  # ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  # IMPORTANT: The job ID below ('build_and_test') is what you must
  # enter in GitHub's "Required status checks" field.
  # Do NOT use the display name ("Build & Test").
  # ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  build_and_test:
    name: "Build & Test"
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: test_db
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend

      - name: Check formatting
        run: cargo fmt --all --manifest-path backend/Cargo.toml -- --check

      - name: Run clippy
        run: cargo clippy --all-targets --manifest-path backend/Cargo.toml -- -D warnings

      - name: Run tests
        run: cargo test --workspace --manifest-path backend/Cargo.toml
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/test_db
```

### For Frontend (if applicable)

Add this as a separate job:

```yaml
  frontend_checks:
    name: "Frontend Checks"
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

      - name: Lint
        run: cd frontend && npm run lint

      - name: Type check
        run: cd frontend && npm run typecheck

      - name: Test
        run: cd frontend && npm test -- --passWithNoTests
```

Then add `frontend_checks` to your branch protection required checks.

---

## Quick Reference

| Action | Command |
|--------|---------|
| Create feature branch | `git checkout -b feature/name` |
| Run automation | `./scripts/auto-pr.sh "Title"` |
| Check PR status | `gh pr view` |
| Check CI status | `gh pr checks` |
| View PR in browser | `gh pr view --web` |
| Manual merge | `gh pr merge --squash` |

---

*Last updated: 2026-01-20*
