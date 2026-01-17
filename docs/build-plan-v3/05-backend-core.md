# Section 5: Backend Core

**Time Required**: ~1-2 hours  
**Difficulty**: Medium  
**Goal**: Complete Rust workspace, Axum server configuration, core services layer, **OpenAPI generation**

---

## Overview

Now we'll build out the full backend structure:
- Rust workspace with all crates
- Configuration management
- Error handling
- Core service layer (ports & adapters pattern)
- API routing skeleton
- **OpenAPI specification generation** (contract-first)

---

## Step-by-Step

### Step 5.1: Complete Rust Workspace Setup

---

#### 📋 PROMPT 5.1.1: Set Up Complete Rust Workspace

```
Complete the Rust workspace setup for the Investment Research Platform.

Update `backend/Cargo.toml` (workspace root) with:
- All workspace members: api, core, db, worker, providers
- Shared dependencies in [workspace.dependencies]
- Common version definitions

Create/update Cargo.toml files for each crate:

1. `backend/core/Cargo.toml`:
   - async-trait = "0.1"
   - thiserror = "1.0"
   - uuid = { version = "1.0", features = ["v4", "serde"] }
   - chrono = { version = "0.4", features = ["serde"] }
   - serde = { version = "1.0", features = ["derive"] }
   - serde_json = "1.0"
   - tracing = "0.1"

2. `backend/api/Cargo.toml`:
   - axum = "0.7"
   - tokio = { version = "1.0", features = ["full"] }
   - tower = "0.4"
   - tower-http = { version = "0.5", features = ["cors", "trace"] }
   - tracing = "0.1"
   - tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
   - utoipa = { version = "4.0", features = ["axum_extras"] }
   - utoipa-swagger-ui = { version = "6.0", features = ["axum"] }
   - Dependencies on internal crates: core, db, providers

3. `backend/worker/Cargo.toml`:
   - tokio = { version = "1.0", features = ["full"] }
   - Dependencies on: core, db, providers

4. `backend/providers/Cargo.toml`:
   - reqwest = { version = "0.11", features = ["json"] }
   - async-trait = "0.1"
   - serde = { version = "1.0", features = ["derive"] }
   - serde_json = "1.0"
   - chrono = "0.4"
   - Dependencies on: core (for trait definitions)

Ensure all crates compile with: cargo check --workspace
```

**Verification**: `cargo check --workspace` succeeds.

---

### Step 5.2: Configuration Management

---

#### 📋 PROMPT 5.2.1: Create Configuration System

```
Create a type-safe configuration system for the API.

Create `backend/api/src/config.rs` with:

1. `Config` struct containing:
   - `database_url: String`
   - `jwt_private_key_path: Option<String>` (RS256 private key file)
   - `jwt_public_key_path: Option<String>` (RS256 public key file)
   - `jwt_private_key: Option<String>` (inline PEM, alternative to file)
   - `jwt_public_key: Option<String>` (inline PEM, alternative to file)
   - `server_host: String` (default: 0.0.0.0)
   - `server_port: u16` (default: 8080)
   - `cors_origins: Vec<String>`
   - `alpha_vantage_api_key: Option<String>`
   - `s3_endpoint: String`
   - `s3_access_key: String`
   - `s3_secret_key: String`
   - `environment: Environment` (enum: Development, Staging, Production)

2. `Config::from_env()` function that:
   - Reads from environment variables
   - For JWT keys: first try file paths, then try inline environment variables
   - Provides sensible defaults for development
   - Validates required fields
   - Returns Result<Config, ConfigError>

3. Environment detection:
   - Check ENVIRONMENT or RUST_ENV variable
   - Default to Development if not set

4. Derive Debug for Config but redact sensitive fields (jwt keys, etc.)

Note: We use RS256 (asymmetric RSA keys) NOT HS256 (symmetric secret).
Do NOT include a jwt_secret field - that's for HS256 which we don't use.

Use std::env for reading, no external config library needed for MVP.
```

**Verification**: `cargo check -p api` compiles.

---

### Step 5.3: Error Handling

---

#### 📋 PROMPT 5.3.1: Create Comprehensive Error Types

```
Create a comprehensive error handling system.

Create `backend/core/src/error.rs` following architecture-design-v3.md section 4.4:

1. Define `AppError` enum with variants:
   - `AuthError(String)` - 401 Unauthorized
   - `ForbiddenError(String)` - 403 Forbidden
   - `NotFound { resource: &'static str, id: String }` - 404
   - `ValidationError(String)` - 400 Bad Request
   - `DatabaseError(sqlx::Error)` - 500 (via From trait)
   - `ExternalApiError { provider: String, message: String }` - 502
   - `RateLimitExceeded` - 429
   - `OptimisticLockConflict { resource: &'static str, id: String, current_version: i32 }` - 409
   - `InternalError(String)` - 500

2. Implement thiserror::Error for user-friendly messages

3. Create `ApiErrorResponse` struct for JSON response:
   - `error.code: String`
   - `error.message: String`
   - `error.details: Option<serde_json::Value>`

4. Create `backend/api/src/error.rs` that:
   - Implements `axum::response::IntoResponse` for AppError
   - Maps each variant to appropriate HTTP status code
   - Returns JSON error response
   - Logs errors appropriately (warn for client errors, error for server errors)

The error handling should be consistent across all API endpoints.
```

**Verification**: `cargo check --workspace` compiles.

---

### Step 5.4: Core Ports (Traits)

---

#### 📋 PROMPT 5.4.1: Define Core Port Traits

```
Create the port traits (interfaces) for the core layer.

Reference architecture-design-v3.md sections 4.2 and 4.3.

Create `backend/core/src/ports/mod.rs` with module declarations.

Create these trait files:

1. `backend/core/src/ports/market_data.rs`:
   ```rust
   #[async_trait]
   pub trait MarketDataProvider: Send + Sync {
       async fn get_company_overview(&self, symbol: &str) -> Result<CompanyOverview, AppError>;
       async fn get_income_statement(&self, symbol: &str) -> Result<Vec<IncomeStatement>, AppError>;
       async fn get_balance_sheet(&self, symbol: &str) -> Result<Vec<BalanceSheet>, AppError>;
       async fn get_cash_flow(&self, symbol: &str) -> Result<Vec<CashFlowStatement>, AppError>;
       async fn get_daily_prices(&self, symbol: &str, output_size: OutputSize) -> Result<Vec<DailyPrice>, AppError>;
       async fn get_earnings_calendar(&self) -> Result<Vec<EarningsEvent>, AppError>;
   }
   ```

2. `backend/core/src/ports/document_provider.rs`:
   ```rust
   #[async_trait]
   pub trait DocumentProvider: Send + Sync {
       async fn get_earnings_transcript(&self, symbol: &str, year: i32, quarter: i32) -> Result<Transcript, AppError>;
       async fn list_sec_filings(&self, symbol: &str, filing_type: &str) -> Result<Vec<Filing>, AppError>;
   }
   ```

3. `backend/core/src/ports/storage.rs`:
   ```rust
   #[async_trait]
   pub trait ObjectStorage: Send + Sync {
       async fn put_object(&self, key: &str, data: Bytes, content_type: &str) -> Result<(), AppError>;
       async fn get_object(&self, key: &str) -> Result<Bytes, AppError>;
       async fn get_presigned_url(&self, key: &str, expires_in: Duration) -> Result<String, AppError>;
       async fn delete_object(&self, key: &str) -> Result<(), AppError>;
   }
   ```

These traits define what the core layer needs, without knowing HOW it's implemented.
```

**Verification**: `cargo check -p core` compiles.

---

### Step 5.5: Application State

---

#### 📋 PROMPT 5.5.1: Create Application State

```
Create the shared application state for the Axum server.

Create `backend/api/src/state.rs` with:

1. `AppState` struct containing:
   - `db: PgPool` - Database connection pool
   - `config: Arc<Config>` - Application configuration
   - `market_data: Arc<dyn MarketDataProvider>` - Market data provider
   - `storage: Arc<dyn ObjectStorage>` - S3 storage
   - `jwt_service: Arc<JwtService>` - JWT token service (RS256)

2. Implement `Clone` for AppState (needed by Axum)

3. Create `AppState::new()` async function that:
   - Initializes database pool
   - Creates appropriate provider implementations based on config.environment
   - Loads RSA keys from file paths or environment variables
   - Creates JwtService with RS256 keys
   - Returns configured AppState

4. Create `JwtService` struct in `backend/api/src/auth/jwt.rs` with:
   - `encoding_key: EncodingKey` (RSA private key for signing)
   - `decoding_key: DecodingKey` (RSA public key for verification)
   - Method to create access tokens
   - Method to create refresh tokens
   - Method to validate tokens
   - RS256 algorithm ONLY (no HS256)

The state should be created once at startup and shared across all handlers.
```

**Verification**: `cargo check -p api` compiles.

---

### Step 5.6: Router Setup with OpenAPI

---

#### 📋 PROMPT 5.6.1: Create Router Structure with OpenAPI

```
Create the complete router structure for the API with OpenAPI documentation.

Create `backend/api/src/routes/mod.rs` with:

1. `create_router(state: AppState) -> Router` function that assembles all routes:
   ```rust
   Router::new()
       // OpenAPI documentation
       .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
       
       // Public routes (no auth)
       .route("/health", get(health_check))
       .route("/api/v1/auth/login", post(login))
       .route("/api/v1/auth/refresh", post(refresh_token))
       
       // Protected routes (require auth)
       .nest("/api/v1", protected_routes(state.clone()))
       
       // Add middleware layers
       .layer(TraceLayer::new_for_http())
       .layer(CorsLayer::permissive()) // Tighten for production
       .with_state(state)
   ```

2. Create `protected_routes(state: AppState) -> Router<AppState>`:
   - Apply auth middleware
   - Nest module routes:
     - /companies -> companies_router()
     - /screeners -> screeners_router()
     - /users/me -> user_router()

3. Create OpenAPI documentation struct:
   ```rust
   #[derive(OpenApi)]
   #[openapi(
       paths(
           health::health_check,
           // More paths added as we implement them
       ),
       components(schemas(
           HealthResponse,
           // More schemas added as we implement them
       )),
       tags(
           (name = "health", description = "Health check endpoints"),
           (name = "auth", description = "Authentication endpoints"),
           (name = "companies", description = "Company data endpoints"),
           (name = "screeners", description = "Screener endpoints"),
           (name = "verdicts", description = "Verdict endpoints"),
       )
   )]
   pub struct ApiDoc;
   ```

4. Create placeholder router files (we'll implement later):
   - `backend/api/src/routes/health.rs` - Health check handler with #[utoipa::path]
   - `backend/api/src/routes/auth.rs` - Auth handlers (placeholder)
   - `backend/api/src/routes/companies.rs` - Company routes (placeholder)
   - `backend/api/src/routes/screeners.rs` - Screener routes (placeholder)
   - `backend/api/src/routes/users.rs` - User routes (placeholder)

5. Update `backend/api/src/main.rs` to:
   - Load configuration
   - Initialize app state
   - Create router
   - Start server with graceful shutdown

The server should start and respond to /health requests.
OpenAPI docs should be available at /swagger-ui.
```

**Verification**: 
- `cargo build -p api`
- Server starts and /health returns 200
- /swagger-ui shows API documentation

---

### Step 5.7: Git Checkpoint

```bash
docker compose restart api

# Wait for compilation
docker compose logs -f api

# Test health endpoint
curl http://localhost:8080/health

# Test OpenAPI docs (open in browser)
# http://localhost:8080/swagger-ui/

# If working, commit
git add .

git commit -m "feat(api): backend core infrastructure with OpenAPI

- Complete Rust workspace with all crates
- Configuration system with RS256 JWT key paths
- Comprehensive error handling with HTTP status mapping
- Core port traits (MarketDataProvider, DocumentProvider, ObjectStorage)
- Application state with dependency injection
- Router structure with public/protected routes
- OpenAPI documentation via utoipa + swagger-ui
- Middleware setup (CORS, tracing)

Architecture follows clean architecture principles from architecture-design-v3.md.
Contract-first approach: OpenAPI spec generated from code annotations."

git push origin develop
```

---

## Verification Checklist

After completing this section, verify:

- [ ] `cargo check --workspace` compiles without errors
- [ ] `cargo build -p api` builds successfully
- [ ] Docker API container starts without errors
- [ ] `curl http://localhost:8080/health` returns JSON
- [ ] `/swagger-ui/` shows OpenAPI documentation
- [ ] Config uses RS256 key paths (not JWT_SECRET)
- [ ] Commit pushed to GitHub
- [ ] CI passes

---

## Next Step

**Proceed to**: [06-authentication-system.md](./06-authentication-system.md)
