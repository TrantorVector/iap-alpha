# Section 3: Docker Development Environment

**Time Required**: ~45 minutes  
**Difficulty**: Medium  
**Goal**: Set up Docker Compose to run PostgreSQL, MinIO (S3), backend, and frontend locally

---

## Overview

You've never used Docker before, so let me explain simply:

**Docker** = A way to run software in isolated "containers" (like mini virtual machines)
**Docker Compose** = A tool that starts multiple containers together

For our project, Docker will run:
- **PostgreSQL** - Database
- **MinIO** - S3-compatible storage (for documents)
- **Backend** - Rust API server
- **Frontend** - React development server

---

## Essential Docker Commands

Before we start, memorize these commands:

| Command | What It Does |
|---------|--------------|
| `docker compose up -d` | Start all containers in background |
| `docker compose down` | Stop all containers |
| `docker compose logs -f` | Show container logs (Ctrl+C to stop) |
| `docker compose logs -f api` | Show only API logs |
| `docker compose restart api` | Restart just the API container |
| `docker ps` | List running containers |
| `docker compose build` | Rebuild containers after code changes |
| `docker system prune` | Clean up unused Docker data |

---

## Step-by-Step

### Step 3.1: Create Docker Configuration Files

**Copy-paste this prompt into Antigravity:**

---

#### 📋 PROMPT 3.1.1: Create Docker Compose Configuration

```
Create a complete Docker Compose development environment for the Investment Research Platform.

Create `docker-compose.yml` in the project root with these services:

1. **postgres** service:
   - Image: postgres:15
   - Environment: POSTGRES_DB=irp_dev, POSTGRES_USER=postgres, POSTGRES_PASSWORD=dev
   - Ports: 5432:5432
   - Volume: postgres_data:/var/lib/postgresql/data
   - Healthcheck: pg_isready command

2. **minio** service (S3-compatible storage):
   - Image: minio/minio
   - Command: server /data --console-address ":9001"
   - Environment: MINIO_ROOT_USER=minioadmin, MINIO_ROOT_PASSWORD=minioadmin
   - Ports: 9000:9000 (API), 9001:9001 (console)
   - Volume: minio_data:/data

3. **api** service:
   - Build from backend/Dockerfile.dev
   - Environment: DATABASE_URL, RUST_LOG=debug, S3_ENDPOINT, etc.
   - Ports: 8080:8080
   - Volumes: mount backend folder for live reload
   - Depends on: postgres, minio
   - Command: cargo watch -x run
   - Working directory: /app/api

4. **frontend** service:
   - Build from frontend/Dockerfile.dev
   - Ports: 3000:3000
   - Volumes: mount frontend folder, separate volume for node_modules
   - Command: npm run dev
   - Environment: VITE_API_URL=http://localhost:8080

5. **Define volumes**: postgres_data, minio_data

Also create a `docker-compose.override.yml` for local development overrides (optional settings).
```

**Verification**: File exists at project root.

---

#### 📋 PROMPT 3.1.2: Create Backend Development Dockerfile

```
Create `backend/Dockerfile.dev` for local Rust development with hot reload.

Requirements:
1. Use rust:1.75-slim as base image
2. Install cargo-watch for hot reload
3. Install build dependencies (libssl-dev, pkg-config, etc.)
4. Set working directory to /app
5. Copy Cargo.toml and Cargo.lock first for dependency caching
6. Copy source code
7. Default command: cargo watch -x run

The Dockerfile should be optimized for:
- Layer caching (dependencies cached separately from code)
- Fast rebuild times
- Development experience (readable errors, debug symbols)

Also create a `.dockerignore` in the backend folder to exclude:
- target/
- .git/
- *.log
```

**Verification**: File exists at `backend/Dockerfile.dev`.

---

#### 📋 PROMPT 3.1.3: Create Frontend Development Dockerfile

```
Create `frontend/Dockerfile.dev` for React development with Vite HMR (Hot Module Replacement).

Requirements:
1. Use node:20-slim as base image
2. Set working directory to /app
3. Copy package.json and package-lock.json first
4. Run npm ci for reproducible installs
5. Copy source code
6. Expose port 3000
7. Default command: npm run dev -- --host

The Dockerfile should:
- Enable Vite HMR by binding to 0.0.0.0
- support volume mounting for live code updates
- Cache node_modules properly

Also create a `.dockerignore` in the frontend folder to exclude:
- node_modules/
- dist/
- .git/
```

**Verification**: File exists at `frontend/Dockerfile.dev`.

---

### Step 3.2: Create Placeholder Backend Code

The Docker container needs something to run. Let's create a minimal "hello world" backend.

---

#### 📋 PROMPT 3.2.1: Create Minimal Rust Backend

```
Create a minimal Rust Axum backend that Docker can run.

Create the following files:

1. `backend/Cargo.toml` - Workspace manifest with members: api, core, db, worker, providers

2. `backend/api/Cargo.toml` - Dependencies:
   - axum = "0.7"
   - tokio = { version = "1.0", features = ["full"] }
   - tracing = "0.1"
   - tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
   - tower-http = { version = "0.5", features = ["cors", "trace"] }

3. `backend/api/src/main.rs` - Minimal Axum server with:
   - Health check endpoint: GET /health returns {"status": "healthy"}
   - Tracing initialization with JSON formatting
   - Listening on 0.0.0.0:8080
   - CORS middleware for development

4. Create empty `lib.rs` files for other crates (core, db, worker, providers) with placeholder Cargo.toml files

The server should log startup message and respond to health checks.
```

**Verification**: 
- `cargo check` runs without errors in backend folder
- Health endpoint is configured

---

#### 📋 PROMPT 3.2.2: Create Minimal React Frontend

```
Create a minimal React + Vite + TypeScript frontend that Docker can run.

Initialize the frontend with these files:

1. `frontend/package.json` with dependencies:
   - react, react-dom (^18)
   - @types/react, @types/react-dom
   - typescript (^5)
   - vite, @vitejs/plugin-react
   - tailwindcss, postcss, autoprefixer
   - @tanstack/react-query

2. `frontend/vite.config.ts` with:
   - React plugin
   - Server configured for 0.0.0.0:3000 (for Docker)
   - Proxy /api to http://api:8080 for Docker networking

3. `frontend/tsconfig.json` with strict TypeScript settings

4. `frontend/tailwind.config.js` and `frontend/postcss.config.js`

5. `frontend/index.html` - Standard Vite HTML entry

6. `frontend/src/main.tsx` - React entry point

7. `frontend/src/App.tsx` - Simple component that:
   - Displays "Investment Research Platform"
   - Shows a loading state
   - Fetches /api/health and displays the result
   - Uses Tailwind for basic styling

8. `frontend/src/index.css` with Tailwind imports

The app should show "Connected to API" when backend is healthy, or "Connecting..." when loading.
```

**Verification**: 
- `npm install` runs without errors
- `npm run dev` starts (if running outside Docker)

---

### Step 3.3: Test Docker Setup

Now let's start everything!

Open terminal in project root and run:

```bash
# Build and start all containers
docker compose up --build -d

# Watch the logs (press Ctrl+C to stop watching)
docker compose logs -f
```

**Wait 2-3 minutes** for:
1. PostgreSQL to start
2. MinIO to start
3. Rust to compile (first time is slow!)
4. Frontend to start

---

### Step 3.4: Verify Everything Works

**Check container status:**
```bash
docker ps
```

You should see 4 containers running: postgres, minio, api, frontend.

**Check each service:**

| Service | URL | Expected Result |
|---------|-----|-----------------|
| Frontend | http://localhost:3000 | React app loads |
| API Health | http://localhost:8080/health | `{"status": "healthy"}` |
| MinIO Console | http://localhost:9001 | Login page (minioadmin/minioadmin) |

**Check API from frontend:**
- Open http://localhost:3000
- Should show "Connected to API" or similar success message

---

### Step 3.5: Common Docker Operations

**Restart a service after code changes:**
```bash
docker compose restart api
```

**Force rebuild a service:**
```bash
docker compose up --build api -d
```

**View errors in a specific container:**
```bash
docker compose logs api --tail 50
```

**Stop everything:**
```bash
docker compose down
```

**Stop and remove all data (fresh start):**
```bash
docker compose down -v  # -v removes volumes (database data!)
```

---

### Step 3.6: Git Checkpoint

Time to commit your Docker setup:

```bash
# Add all changes
git add .

# Commit
git commit -m "feat: add Docker development environment

- docker-compose.yml with postgres, minio, api, frontend services
- Backend Dockerfile.dev with cargo-watch hot reload
- Frontend Dockerfile.dev with Vite HMR
- Minimal Axum health check endpoint
- Minimal React app with API connection test

Closes #1"

# Push
git push origin develop
```

---

## Verification Checklist

After completing this section, verify:

- [ ] `docker compose up -d` starts all 4 containers
- [ ] `docker ps` shows all containers healthy
- [ ] http://localhost:8080/health returns JSON
- [ ] http://localhost:3000 shows React app
- [ ] http://localhost:9001 shows MinIO login
- [ ] Frontend displays API connection status
- [ ] Changes committed and pushed to GitHub

---

## Next Step

**Proceed to**: [04-database-foundation.md](./04-database-foundation.md)

---

## Troubleshooting

### Container keeps restarting
```bash
# Check the logs for errors
docker compose logs api --tail 100
```

### Port already in use
```bash
# Find what's using the port
sudo lsof -i :8080

# Or change ports in docker-compose.yml
```

### Rust compilation is slow
First build is slow (~5 min). Subsequent rebuilds are faster due to caching.

### Frontend shows "Connecting..." forever
```bash
# Check API is running
curl http://localhost:8080/health

# Check Docker network
docker network ls
docker compose logs frontend
```

### Database connection refused
```bash
# Check postgres is running
docker compose logs postgres

# Database might still be starting
docker compose restart api
```
