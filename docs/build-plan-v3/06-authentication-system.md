# Section 6: Authentication System

**Time Required**: ~1-2 hours  
**Difficulty**: Medium-High  
**Goal**: Implement RS256 JWT-based authentication with login/logout, token refresh

---

## Overview

Authentication components:
- **Password hashing**: Argon2id (as specified in architecture)
- **JWT tokens**: RS256 asymmetric signing (RSA keys)
- **Access tokens**: 24-hour expiry
- **Refresh tokens**: 30-day expiry, stored in database
- **Middleware**: Protect routes requiring authentication

> [!IMPORTANT]
> This platform uses **RS256 (asymmetric RSA keys) exclusively**. We do NOT use HS256 (symmetric shared secret). This is important because:
> - RS256 allows the public key to be shared for token validation (useful for microservices)
> - Key rotation is safer (only need to update the private key for signing)
> - Follows security best practices from architecture-design-v3.md section 7

---

## Step-by-Step

### Step 6.1: Password Hashing

---

#### 📋 PROMPT 6.1.1: Create Password Hashing Module

```
Create a password hashing module using Argon2id.

Add to `backend/api/Cargo.toml`:
- argon2 = "0.5"
- rand_core = { version = "0.6", features = ["std"] }

Create `backend/api/src/auth/password.rs` with:

1. Constants following architecture spec (section 7.3):
   - MEMORY_SIZE = 65536 (64 MiB)
   - ITERATIONS = 3
   - PARALLELISM = 4
   - OUTPUT_LENGTH = 32 bytes

2. `hash_password(password: &str) -> Result<String, AppError>`:
   - Generate random salt using rand_core
   - Hash with Argon2id and above parameters
   - Return base64-encoded hash string (PHC format)

3. `verify_password(password: &str, hash: &str) -> Result<bool, AppError>`:
   - Parse the stored hash
   - Verify password matches
   - Return true/false, never expose timing information

The implementation should be constant-time to prevent timing attacks.
```

**Verification**: Unit tests pass.

---

### Step 6.2: JWT Token Handling

---

#### 📋 PROMPT 6.2.1: Create JWT Token Module (RS256 Only)

```
Create JWT token handling with RS256 asymmetric signing.

IMPORTANT: Use ONLY RS256 (RSA asymmetric keys), NOT HS256.

Add to `backend/api/Cargo.toml`:
- jsonwebtoken = "9.0"

Create `backend/api/src/auth/jwt.rs` with:

1. `Claims` struct (token payload):
   - sub: String (user_id as string)
   - iat: i64 (issued at timestamp)
   - exp: i64 (expiration timestamp)
   - iss: String (issuer, "iap-api")
   - scope: Vec<String> (permissions)

2. `JwtService` struct:
   - encoding_key: EncodingKey (RSA private key for signing)
   - decoding_key: DecodingKey (RSA public key for verification)
   - access_token_duration: Duration (24 hours)
   - refresh_token_duration: Duration (30 days)

3. Constructor:
   ```rust
   impl JwtService {
       pub fn new(
           private_key_pem: &str,  // RSA private key in PEM format
           public_key_pem: &str,   // RSA public key in PEM format
       ) -> Result<Self, AppError>
   }
   ```

4. Methods:
   - `create_access_token(&self, user_id: Uuid, scopes: Vec<String>) -> Result<String>`
   - `create_refresh_token(&self, user_id: Uuid) -> Result<(String, String)>` 
     - Returns (token_string, token_hash) - hash stored in DB
   - `validate_access_token(&self, token: &str) -> Result<Claims>`
   - `decode_without_validating(&self, token: &str) -> Result<Claims>`
     - For inspecting expired tokens (refresh flow)

5. For development, create a function to generate RSA keypair:
   - `generate_dev_keypair() -> (String, String)` (private, public PEM)

6. Algorithm validation:
   - When validating tokens, ONLY accept RS256 algorithm
   - Reject HS256 or any other algorithm (prevents algorithm confusion attacks)

Reference architecture-design-v3.md section 7.1 and 7.2 for token configuration.
```

**Verification**: Unit tests for token creation and validation.

---

### Step 6.3: Authentication Middleware

---

#### 📋 PROMPT 6.3.1: Create Auth Middleware

```
Create authentication middleware for Axum.

Create `backend/api/src/middleware/auth.rs` with:

1. `auth_middleware` async function:
   - Extract Authorization header
   - Validate Bearer token format
   - Validate JWT using JwtService (RS256 only)
   - Insert Claims into request extensions
   - Return 401 Unauthorized for invalid/missing tokens

2. `optional_auth_middleware`:
   - Same as above but doesn't fail on missing token
   - For routes that work with or without auth

3. `Claims` extractor for route handlers:
   - Implement FromRequestParts for Claims
   - Extract from request extensions

4. Example usage in handler:
   ```rust
   async fn protected_handler(
       Extension(claims): Extension<Claims>,  // Or custom extractor
       State(state): State<AppState>,
   ) -> Result<Json<Response>, AppError> {
       let user_id = Uuid::parse_str(&claims.sub)?;
       // ...
   }
   ```

The middleware should:
- Log authentication attempts
- Not leak information about why auth failed
- Be efficient (no database calls, just token validation)
```

**Verification**: Middleware compiles and can be applied to routes.

---

### Step 6.4: Auth Route Handlers

---

#### 📋 PROMPT 6.4.1: Create Login Handler

```
Create the login route handler.

Update `backend/api/src/routes/auth.rs` with:

1. `LoginRequest` struct:
   - username: String
   - password: String

2. `LoginResponse` struct:
   - access_token: String
   - refresh_token: String
   - token_type: "Bearer"
   - expires_in: i64 (seconds)
   - user: UserInfo { id, username, email }

3. `login` handler (POST /api/v1/auth/login):
   - Validate request body
   - Find user by username (using UserRepository)
   - Verify password hash
   - Generate access token and refresh token
   - Store refresh token hash in database
   - Return LoginResponse
   - Log login attempts (success/failure, not passwords)

4. Error handling:
   - Invalid credentials: 401 with generic "Invalid username or password"
   - Don't reveal if username exists vs wrong password

5. Add OpenAPI annotations with #[utoipa::path]

Implement rate limiting note: For MVP, log attempts. Rate limiting comes later.
```

**Verification**: Login works with test user credentials.

---

#### 📋 PROMPT 6.4.2: Create Refresh Token Handler

```
Create the token refresh route handler.

Add to `backend/api/src/routes/auth.rs`:

1. `RefreshRequest` struct:
   - refresh_token: String

2. `RefreshResponse` struct:
   - access_token: String
   - token_type: "Bearer"
   - expires_in: i64

3. `refresh_token` handler (POST /api/v1/auth/refresh):
   - Hash the provided refresh token
   - Look up hash in database
   - Verify token is not expired
   - Verify token is not revoked
   - Generate new access token
   - Return RefreshResponse

4. Error cases:
   - Invalid/expired refresh token: 401
   - Revoked token: 401

5. Add OpenAPI annotations

Note: Refresh token rotation (issuing new refresh token) is deferred to Phase 2.
```

**Verification**: Token refresh works for valid refresh tokens.

---

#### 📋 PROMPT 6.4.3: Create Logout Handler

```
Create the logout route handler.

Add to `backend/api/src/routes/auth.rs`:

1. `logout` handler (POST /api/v1/auth/logout):
   - Requires authentication (use auth middleware)
   - Extract user_id from claims
   - Revoke all refresh tokens for user
   - Return 204 No Content

2. Optional: Accept refresh_token in body to revoke only that token

3. Add OpenAPI annotations

The logout should:
- Be idempotent (multiple logouts are fine)
- Log the logout event
- Not fail if no active sessions exist
```

**Verification**: Logout revokes tokens in database.

---

### Step 6.5: Generate Development Keys

For development, we need RSA keys. Run this once:

```bash
# Generate RSA private key
openssl genrsa -out private_key.pem 2048

# Extract public key
openssl rsa -in private_key.pem -pubout -out public_key.pem

# Move to a secure location
mkdir -p secrets
mv private_key.pem secrets/
mv public_key.pem secrets/

# Add to .gitignore (should already be there from section 2)
echo "secrets/*.pem" >> .gitignore
```

For Docker, we'll mount these or use environment variables.

---

### Step 6.6: Update Docker Configuration

---

#### 📋 PROMPT 6.6.1: Add JWT Keys to Docker Environment

```
Update the Docker configuration to support RS256 JWT keys for development.

1. Update `docker-compose.yml`:
   - Add volume mount for secrets folder
   - Add environment variables:
     - JWT_PRIVATE_KEY_FILE=/app/secrets/private_key.pem
     - JWT_PUBLIC_KEY_FILE=/app/secrets/public_key.pem

2. Update `.env.example`:
   - Add JWT_PRIVATE_KEY_FILE and JWT_PUBLIC_KEY_FILE
   - Add note about generating RSA keys with openssl
   - REMOVE any reference to JWT_SECRET (we don't use HS256)

3. Update Config to read keys from file or environment variable

4. Create a development key generation script:
   - `scripts/generate-dev-keys.sh`
   - Generates RSA keypair if not exists
   - Outputs them in format suitable for .env file

For development convenience, Config should:
- First check for key files
- Then check for environment variables (inline PEM)
- Finally, generate temporary keys for local dev (with warning)
```

**Verification**: API starts with JWT configuration.

---

### Step 6.7: Write Auth Unit Tests

---

#### 📋 PROMPT 6.7.1: Create Authentication Tests

```
Create comprehensive tests for the authentication system.

Create `backend/api/src/auth/tests.rs` with:

1. Password hashing tests:
   - `test_hash_password_creates_valid_hash`
   - `test_verify_correct_password_succeeds`
   - `test_verify_wrong_password_fails`
   - `test_hash_is_not_deterministic` (salt works)

2. JWT tests (RS256 specific):
   - `test_create_access_token_is_valid`
   - `test_access_token_contains_correct_claims`
   - `test_expired_token_is_rejected`
   - `test_invalid_signature_is_rejected`
   - `test_wrong_algorithm_is_rejected` (HS256 tokens must fail)
   - `test_token_signed_with_wrong_key_is_rejected`

3. Create integration test file `tests/auth_integration.rs`:
   - `test_login_with_valid_credentials_returns_tokens`
   - `test_login_with_invalid_password_returns_401`
   - `test_login_with_nonexistent_user_returns_401`
   - `test_refresh_with_valid_token_returns_new_access_token`
   - `test_refresh_with_expired_token_returns_401`
   - `test_logout_revokes_refresh_token`
   - `test_protected_route_without_token_returns_401`
   - `test_protected_route_with_valid_token_succeeds`

Use the test database and seed data from section 4.
Test user credentials: testuser / TestPass123!
```

**Verification**: All tests pass with `cargo test`.

---

### Step 6.8: Git Checkpoint

```bash
# Make sure Docker is running
docker compose up -d

# Wait for compilation and run tests inside container
docker compose exec api cargo test

# Also test manually
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "TestPass123!"}'

# Should return access_token and refresh_token

# Commit
git add .

git commit -m "feat(auth): implement RS256 JWT authentication system

- Argon2id password hashing (memory=64MB, iterations=3, parallelism=4)
- RS256 JWT tokens with RSA asymmetric keys (NOT HS256)
- 24h access tokens, 30d refresh tokens
- Login endpoint with secure credential verification
- Token refresh endpoint
- Logout with token revocation
- Auth middleware for protected routes
- Development key generation scripts
- Comprehensive unit and integration tests

Security: Constant-time password verification, no info leakage on failed auth.
Algorithm validation prevents confusion attacks."

git push origin develop
```

---

## Verification Checklist

After completing this section, verify:

- [ ] `cargo test` passes all auth tests
- [ ] Login returns tokens for valid credentials
- [ ] Login returns 401 for invalid credentials
- [ ] Token refresh works with valid refresh token
- [ ] Protected routes reject requests without token
- [ ] Protected routes accept requests with valid token
- [ ] Logout revokes tokens
- [ ] Using RS256 (not HS256) - check with jwt.io
- [ ] CI passes
- [ ] Commit pushed to GitHub

---

## Manual Testing Commands

```bash
# 1. Login
ACCESS_TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "TestPass123!"}' \
  | jq -r '.access_token')

echo "Access Token: $ACCESS_TOKEN"

# 2. Verify it's RS256 (paste token at jwt.io, header should show "alg": "RS256")

# 3. Access protected endpoint (we'll add one)
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  http://localhost:8080/api/v1/users/me

# 4. Refresh token (save from login response)
curl -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "YOUR_REFRESH_TOKEN"}'

# 5. Logout
curl -X POST http://localhost:8080/api/v1/auth/logout \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

---

## Next Step

**Proceed to**: [07-analyzer-module-backend.md](./07-analyzer-module-backend.md)
