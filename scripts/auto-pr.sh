#!/bin/bash

#===============================================================================
# AUTOMATED PR & CI WORKFLOW SCRIPT (v3.0 - Production Ready)
#
# Architecture: Native Auto-Merge (Fire-and-Forget)
# Environment: Headless Hetzner Server via Remote-SSH
#
# This script:
# 1. Creates a PR on GitHub
# 2. Enables GitHub's native auto-merge
# 3. Exits immediately if auto-merge succeeds (GitHub handles the rest)
# 4. Only enters fix loop if CI is already failing
#
# The "Human-Agent Bridge":
# This script CANNOT invoke Antigravity directly. When CI fails, it pauses
# and instructs the human to copy logs into their local Antigravity IDE.
#
# Usage: ./scripts/auto-pr.sh [PR_TITLE]
#===============================================================================

set -euo pipefail

#-------------------------------------------------------------------------------
# Configuration (Dynamic)
#-------------------------------------------------------------------------------

REPO_DIR=$(git rev-parse --show-toplevel 2>/dev/null) || {
    echo "[ERROR] Not in a git repository."
    exit 1
}

CI_LOG_DIR="${REPO_DIR}/ci-logs"
CI_WORKFLOW_PATH="${REPO_DIR}/.github/workflows/ci.yml"
MAX_RETRIES=20
CI_REGISTRATION_WAIT=15  # Seconds to wait for GitHub to register checks

#-------------------------------------------------------------------------------
# Colors
#-------------------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

#-------------------------------------------------------------------------------
# Logging Functions
#-------------------------------------------------------------------------------

log_info()    { echo -e "${BLUE}[INFO]${NC} $1" >&2; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1" >&2; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1" >&2; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1" >&2; }

log_step() {
    echo "" >&2
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}" >&2
    echo -e "${CYAN}▶ $1${NC}" >&2
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}" >&2
    echo "" >&2
}

#-------------------------------------------------------------------------------
# Pre-flight Checks
#-------------------------------------------------------------------------------

preflight_checks() {
    log_step "Pre-flight Checks"

    # Check: GitHub CLI installed
    if ! command -v gh &>/dev/null; then
        log_error "GitHub CLI (gh) is not installed."
        log_info "Install: https://cli.github.com/"
        exit 1
    fi
    log_success "GitHub CLI installed"

    # Check: GitHub CLI authenticated
    if ! gh auth status &>/dev/null; then
        log_error "Not authenticated with GitHub CLI."
        log_info "Run: gh auth login"
        exit 1
    fi
    log_success "GitHub CLI authenticated"

    # Check: CI workflow exists (CRITICAL)
    if [[ ! -f "$CI_WORKFLOW_PATH" ]]; then
        log_error "CI workflow not found at: $CI_WORKFLOW_PATH"
        log_error "This script requires a CI workflow to function."
        log_info "See the Appendix in the documentation for a template."
        exit 1
    fi
    log_success "CI workflow found: .github/workflows/ci.yml"

    # Check: Not on main/master branch
    local current_branch
    current_branch=$(git branch --show-current)
    if [[ "$current_branch" == "main" || "$current_branch" == "master" ]]; then
        log_error "You are on the '$current_branch' branch."
        log_info "Create a feature branch first: git checkout -b feature/your-feature"
        exit 1
    fi
    log_success "On feature branch: $current_branch"

    # Check: Working directory clean or prompt to commit
    if ! git diff --quiet || ! git diff --staged --quiet; then
        log_warning "You have uncommitted changes."
        echo -e "${YELLOW}Commit all changes now? (y/n)${NC}"
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            echo -e "${YELLOW}Enter commit message:${NC}"
            read -r commit_message
            git add .
            git commit -m "$commit_message"
            log_success "Changes committed"
        else
            log_error "Please commit your changes before running this script."
            exit 1
        fi
    else
        log_success "Working directory clean"
    fi
}

#-------------------------------------------------------------------------------
# Git Operations
#-------------------------------------------------------------------------------

push_branch() {
    log_step "Pushing Branch to GitHub"

    local current_branch
    current_branch=$(git branch --show-current)

    if git push -u origin "$current_branch" 2>&1; then
        log_success "Pushed to origin/$current_branch"
    else
        log_error "Failed to push branch"
        exit 1
    fi
}

#-------------------------------------------------------------------------------
# PR Operations (Robust JSON Parsing)
#-------------------------------------------------------------------------------

get_or_create_pr() {
    log_step "Creating/Finding Pull Request"

    local current_branch pr_number pr_title
    current_branch=$(git branch --show-current)
    pr_title="${1:-$current_branch}"

    # Check if PR already exists (robust JSON parsing, no regex)
    pr_number=$(gh pr view --json state,number --jq 'select(.state == "OPEN") | .number' 2>/dev/null || echo "")

    if [[ -n "$pr_number" ]]; then
        log_info "PR #$pr_number already exists for this branch"
        echo "$pr_number"
        return
    fi

    # Create new PR
    log_info "Creating new PR: $pr_title"
    gh pr create \
        --title "$pr_title" \
        --body "Automated PR created by auto-pr.sh" \
        --head "$current_branch" \
        >/dev/null 2>&1

    # Get PR number using JSON (robust, no regex)
    pr_number=$(gh pr view --json number --jq '.number')
    log_success "Created PR #$pr_number"
    echo "$pr_number"
}

#-------------------------------------------------------------------------------
# Auto-Merge Operations
#-------------------------------------------------------------------------------

enable_auto_merge() {
    local pr_number=$1

    log_step "Enabling Auto-Merge"

    if gh pr merge --auto --squash --delete-branch "$pr_number" 2>&1; then
        return 0  # Success
    else
        return 1  # Failed (likely CI already failing)
    fi
}

#-------------------------------------------------------------------------------
# CI Status Check (with Race Condition Fix)
#-------------------------------------------------------------------------------

wait_for_checks_to_register() {
    local pr_number=$1
    local max_wait=120
    local waited=0

    log_info "Polling GitHub until checks appear (max ${max_wait}s)..."

    while [[ $waited -lt $max_wait ]]; do
        local status
        status=$(get_ci_status "$pr_number")

        if [[ "$status" != "none" ]]; then
            log_success "CI checks registered and in status: $status"
            return 0
        fi

        sleep 5
        waited=$((waited + 5))
        if (( waited % 20 == 0 )); then
            log_info "Still waiting for checks to register... (${waited}s)"
        fi
    done

    log_warning "No checks appeared after ${max_wait}s. This might cause an immediate merge if you are an admin."
}

get_ci_status() {
    # Returns: "success", "failure", "pending", or "none"
    local pr_number=$1
    local states

    states=$(gh pr checks "$pr_number" --json 'state' --jq '.[].state' 2>/dev/null | sort | uniq || echo "")

    if [[ -z "$states" ]]; then
        echo "none"
    elif echo "$states" | grep -qE "PENDING|IN_PROGRESS|QUEUED"; then
        echo "pending"
    elif echo "$states" | grep -qE "FAILURE|ERROR|CANCELLED"; then
        echo "failure"
    elif echo "$states" | grep -q "SUCCESS"; then
        echo "success"
    else
        echo "none"
    fi
}

wait_for_ci_initial_status() {
    local pr_number=$1
    local max_wait=120
    local waited=0

    log_info "Checking initial CI status..."

    while [[ $waited -lt $max_wait ]]; do
        local status
        status=$(get_ci_status "$pr_number")

        case "$status" in
            success)
                log_success "CI checks passed!"
                return 0
                ;;
            failure)
                log_warning "CI checks failed"
                return 1
                ;;
            pending)
                log_info "CI running... (${waited}s elapsed)"
                sleep 15
                waited=$((waited + 15))
                ;;
            none)
                log_info "Waiting for checks to appear... (${waited}s)"
                sleep 10
                waited=$((waited + 10))
                ;;
        esac
    done

    log_warning "CI status check timed out. Proceeding anyway."
    return 2
}

#-------------------------------------------------------------------------------
# Error Extraction
#-------------------------------------------------------------------------------

extract_ci_errors() {
    local pr_number=$1
    local attempt=$2
    local timestamp
    timestamp=$(date '+%Y%m%d_%H%M%S')

    mkdir -p "$CI_LOG_DIR"
    local log_file="${CI_LOG_DIR}/ci-errors_attempt${attempt}_${timestamp}.md"

    cat > "$log_file" << EOF
# CI Error Log - Attempt $attempt

**Repository:** $(basename "$REPO_DIR")
**Branch:** $(git branch --show-current)
**PR:** #$pr_number
**Timestamp:** $(date '+%Y-%m-%d %H:%M:%S')

---

## Failed Checks

EOF

    # Get failed checks
    local failed_checks
    failed_checks=$(gh pr checks "$pr_number" --json 'name,state,detailsUrl' \
        --jq '.[] | select(.state == "FAILURE" or .state == "ERROR") | "\(.name)|\(.detailsUrl)"' 2>/dev/null || echo "")

    if [[ -z "$failed_checks" ]]; then
        echo "No failed checks detected yet." >> "$log_file"
    else
        echo "$failed_checks" | while IFS='|' read -r check_name details_url; do
            {
                echo "### ❌ $check_name"
                echo ""
                echo "**Details:** $details_url"
                echo ""

                # Extract run ID and fetch logs
                local run_id
                run_id=$(echo "$details_url" | grep -oE '/runs/[0-9]+' | grep -oE '[0-9]+' | head -1 || echo "")

                if [[ -n "$run_id" ]]; then
                    echo '```'
                    gh run view "$run_id" --log-failed 2>/dev/null || echo "Could not fetch logs"
                    echo '```'
                fi

                echo ""
                echo "---"
                echo ""
            } >> "$log_file"
        done
    fi

    cat >> "$log_file" << 'EOF'

## Instructions for Antigravity

Please analyze the errors above and fix them. After making changes:

1. Save all files in the IDE
2. Return to the terminal
3. Press ENTER to commit and push the fixes

**Common Issues:**
- Rust: `cargo fmt`, `cargo clippy`, test failures
- TypeScript: ESLint, Prettier, test failures
- Build errors, missing dependencies

EOF

    echo "$log_file"
}

#-------------------------------------------------------------------------------
# Human-Agent Bridge (Blocking Wait)
#-------------------------------------------------------------------------------

prompt_human_to_fix() {
    local log_file=$1

    echo ""
    echo -e "${RED}╔════════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                                                                            ║${NC}"
    echo -e "${RED}║   ❌ CI FAILED - HUMAN INTERVENTION REQUIRED                               ║${NC}"
    echo -e "${RED}║                                                                            ║${NC}"
    echo -e "${RED}╠════════════════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${RED}║                                                                            ║${NC}"
    echo -e "${RED}║   1. OPEN Antigravity (your local IDE)                                     ║${NC}"
    echo -e "${RED}║                                                                            ║${NC}"
    echo -e "${RED}║   2. PASTE THE CONTENTS OF THIS FILE:                                      ║${NC}"
    echo -e "${RED}║      ${BOLD}${log_file}${NC}${RED}                                         ║${NC}"
    echo -e "${RED}║                                                                            ║${NC}"
    echo -e "${RED}║   3. ASK ANTIGRAVITY TO FIX THE ERRORS                                     ║${NC}"
    echo -e "${RED}║                                                                            ║${NC}"
    echo -e "${RED}║   4. AFTER FIXES ARE SAVED, PRESS ENTER HERE                               ║${NC}"
    echo -e "${RED}║                                                                            ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    # Display log contents for easy copying
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━ ERROR LOG CONTENTS ━━━━━━━━━━━━━━━━${NC}"
    cat "$log_file"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━ END OF ERROR LOG ━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    echo -e "${GREEN}Press ENTER when Antigravity has completed the fix...${NC}"
    read -r
}

commit_and_push_fixes() {
    local attempt=$1

    log_step "Committing and Pushing Fixes"

    # Check for changes
    if git diff --quiet && git diff --staged --quiet; then
        log_warning "No changes detected."
        echo -e "${YELLOW}Options: [Enter] retry | [s] skip | [q] quit${NC}"
        read -r response
        case "$response" in
            s|S) return 0 ;;
            q|Q) exit 0 ;;
            *) commit_and_push_fixes "$attempt"; return ;;
        esac
    fi

    git add .
    git commit -m "fix: CI errors - attempt $attempt (automated)"
    git push

    log_success "Fixes pushed"
}

#-------------------------------------------------------------------------------
# Branch Protection Reminder
#-------------------------------------------------------------------------------

show_setup_reminder() {
    echo ""
    echo -e "${YELLOW}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║                        SETUP REMINDER                                     ║${NC}"
    echo -e "${YELLOW}╠═══════════════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${YELLOW}║  For auto-merge to work, ensure:                                          ║${NC}"
    echo -e "${YELLOW}║                                                                           ║${NC}"
    echo -e "${YELLOW}║  1. Repository Settings → General → Allow auto-merge ✓                   ║${NC}"
    echo -e "${YELLOW}║                                                                           ║${NC}"
    echo -e "${YELLOW}║  2. Settings → Branches → Branch protection for 'main':                  ║${NC}"
    echo -e "${YELLOW}║     • Require status checks to pass ✓                                    ║${NC}"
    echo -e "${YELLOW}║     • Add required checks (use JOB IDs from ci.yml, e.g. 'build_and_test')║${NC}"
    echo -e "${YELLOW}║                                                                           ║${NC}"
    echo -e "${YELLOW}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

#-------------------------------------------------------------------------------
# Main
#-------------------------------------------------------------------------------

main() {
    local pr_title="${1:-}"

    echo ""
    echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                                           ║${NC}"
    echo -e "${CYAN}║   🚀 AUTO-PR WORKFLOW (v3.0 - Native Auto-Merge)                          ║${NC}"
    echo -e "${CYAN}║                                                                           ║${NC}"
    echo -e "${CYAN}║   Architecture: Fire-and-Forget with Human-Agent Bridge                   ║${NC}"
    echo -e "${CYAN}║                                                                           ║${NC}"
    echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    cd "$REPO_DIR"

    # Phase 1: Pre-flight
    preflight_checks

    # Phase 2: Push & Create PR
    push_branch
    local pr_number
    pr_number=$(get_or_create_pr "$pr_title")

    # Phase 3: Wait for checks to register (race condition fix)
    wait_for_checks_to_register "$pr_number"

    # Phase 4: Try to enable auto-merge (Fire-and-Forget)
    if enable_auto_merge "$pr_number"; then
        echo ""
        echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║                                                                           ║${NC}"
        echo -e "${GREEN}║   ✅ AUTO-MERGE ENABLED                                                   ║${NC}"
        echo -e "${GREEN}║                                                                           ║${NC}"
        echo -e "${GREEN}║   PR #$pr_number will be automatically merged when CI passes.                    ║${NC}"
        echo -e "${GREEN}║                                                                           ║${NC}"
        echo -e "${GREEN}║   You can close this terminal. GitHub handles the rest!                   ║${NC}"
        echo -e "${GREEN}║                                                                           ║${NC}"
        echo -e "${GREEN}║   Monitor: gh pr view $pr_number --web                                           ║${NC}"
        echo -e "${GREEN}║                                                                           ║${NC}"
        echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
        echo ""
        show_setup_reminder
        exit 0
    fi

    # Phase 5: Auto-merge failed - Check why
    log_warning "Could not enable auto-merge. Checking CI status..."

    wait_for_ci_initial_status "$pr_number"
    local ci_result=$?

    if [[ $ci_result -eq 0 ]]; then
        # CI passed - try again
        if enable_auto_merge "$pr_number"; then
            log_success "Auto-merge enabled after CI passed!"
            exit 0
        fi
    fi

    # Phase 6: Enter Fix Loop
    local attempt=0

    while [[ $attempt -lt $MAX_RETRIES ]]; do
        attempt=$((attempt + 1))

        log_step "Fix Loop - Attempt $attempt of $MAX_RETRIES"

        # Extract errors
        local log_file
        log_file=$(extract_ci_errors "$pr_number" "$attempt")

        # Human-Agent Bridge: Pause and instruct
        prompt_human_to_fix "$log_file"

        # Commit and push fixes
        commit_and_push_fixes "$attempt"

        # Wait for GitHub to process
        log_info "Waiting for GitHub to process new commit..."
        sleep 10

        # Retry auto-merge
        log_step "Retrying Auto-Merge"

        if enable_auto_merge "$pr_number"; then
            echo ""
            echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
            echo -e "${GREEN}║                                                                           ║${NC}"
            echo -e "${GREEN}║   ✅ AUTO-MERGE ENABLED AFTER FIX                                         ║${NC}"
            echo -e "${GREEN}║                                                                           ║${NC}"
            echo -e "${GREEN}║   PR #$pr_number will merge when CI passes.                                      ║${NC}"
            echo -e "${GREEN}║   Attempts: $attempt                                                             ║${NC}"
            echo -e "${GREEN}║                                                                           ║${NC}"
            echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
            echo ""
            exit 0
        fi

        log_warning "Auto-merge still failing. Waiting for CI to complete..."
        sleep 30

        # Check if CI passed
        local status
        status=$(get_ci_status "$pr_number")

        if [[ "$status" == "success" ]]; then
            if enable_auto_merge "$pr_number"; then
                log_success "CI passed! Auto-merge enabled."
                exit 0
            else
                log_info "CI passed. Manual merge: gh pr merge $pr_number --squash"
                exit 0
            fi
        fi

        log_warning "CI still failing. Continuing fix loop..."
    done

    log_error "Max retries ($MAX_RETRIES) reached."
    log_info "Continue manually: gh pr merge --auto --squash $pr_number"
    show_setup_reminder
    exit 1
}

main "$@"
