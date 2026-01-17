# Section 1: Prerequisites & Setup

**Time Required**: ~30 minutes  
**Difficulty**: Easy (all GUI-based)  
**Goal**: Install all tools needed for local development on Ubuntu

---

## Overview

Before you can start building, you need these tools installed:

| Tool | Purpose | Version |
|------|---------|---------|
| **Docker** | Run containers (PostgreSQL, backend, frontend) | Latest |
| **Docker Compose** | Orchestrate multiple containers | v2+ |
| **Git** | Version control | 2.40+ |
| **VS Code** | Code editor with Antigravity | Latest |
| **Node.js** | Optional - for npm commands | 20+ LTS |

---

## Step-by-Step Installation

### Step 1.1: Update System

Open a terminal (Ctrl+Alt+T) and run:

```bash
sudo apt update && sudo apt upgrade -y
```

**What this does**: Updates your package list and installs any pending updates.

---

### Step 1.2: Install Docker

Docker will run PostgreSQL and other services in isolated containers.

```bash
# Install Docker using official convenience script
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add yourself to docker group (avoid sudo for docker commands)
sudo usermod -aG docker $USER

# IMPORTANT: Log out and log back in, or run:
newgrp docker

# Verify installation
docker --version
docker compose version
```

**Expected output**:
```
Docker version 24.x.x or higher
Docker Compose version v2.x.x or higher
```

**Troubleshooting**:
- If `docker compose` doesn't work, try `docker-compose` (older syntax)
- If permission denied: run `sudo chmod 666 /var/run/docker.sock`

---

### Step 1.3: Install Git

```bash
# Install Git
sudo apt install git -y

# Configure Git with your name and email
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Verify
git --version
```

**Expected output**: `git version 2.40.x or higher`

---

### Step 1.4: Install VS Code

**Option A: Via Ubuntu Software Center** (easiest)
1. Open Ubuntu Software Center
2. Search for "Visual Studio Code"
3. Click Install

**Option B: Via Command Line**

```bash
# Download and install VS Code
sudo snap install code --classic

# Verify
code --version
```

---

### Step 1.5: Install Gemini Code Assist Extension

1. Open VS Code
2. Click the Extensions icon (left sidebar, looks like 4 squares)
3. Search for "Gemini Code Assist"
4. Click Install
5. Sign in with your Google account when prompted

> **Note**: This is how you'll access Antigravity. The extension provides the AI chat interface.

---

### Step 1.6: Install Node.js (Optional but Recommended)

Useful for running frontend commands directly if needed.

```bash
# Install Node.js 20 LTS via NodeSource
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Verify
node --version
npm --version
```

**Expected output**:
```
v20.x.x
10.x.x
```

---

### Step 1.7: Install Rust (Optional)

For running backend tests directly. Usually Docker handles this.

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart terminal or run:
source $HOME/.cargo/env

# Verify
rustc --version
cargo --version
```

---

## Verification Checklist

Run this command to verify everything is installed:

```bash
echo "=== Checking Prerequisites ===" && \
docker --version && \
docker compose version && \
git --version && \
code --version && \
echo "=== All Prerequisites Installed ==="
```

**Expected**: No errors, all versions displayed.

---

## Quick Commands Reference

Save these commands - you'll use them daily:

| Action | Command |
|--------|---------|
| Start Docker services | `docker compose up -d` |
| Stop Docker services | `docker compose down` |
| View running containers | `docker ps` |
| View container logs | `docker compose logs -f` |
| Stop following logs | `Ctrl+C` |
| Open VS Code in folder | `code .` |

---

## Git Checkpoint: None

No git checkpoint yet - you haven't created the repository.

---

## Next Step

**Proceed to**: [02-repository-initialization.md](./02-repository-initialization.md)

---

## Troubleshooting

### Docker permission denied
```bash
sudo chmod 666 /var/run/docker.sock
# Or restart after adding to docker group
```

### VS Code won't open
```bash
# Reinstall
sudo snap remove code
sudo snap install code --classic
```

### Git asks for password every time
```bash
# Set up credential caching
git config --global credential.helper store
# Next time you enter password, it will be saved
```
