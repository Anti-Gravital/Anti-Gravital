# ag-auth — Plan de Implementacion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar autenticacion completa: JWT Ed25519, Passkeys/WebAuthn, OAuth2 (Google/GitHub), API keys con hash BLAKE3, y refresh tokens con rotacion.

**Architecture:** Un struct `AgAuth` con metodos por flujo. Persistencia feature-gated (`features = ["persistent"]`). Migraciones SQL propias en `crates/ag-auth/migrations/`. Rama: `fase-4/ag-auth`.

**Precondicion:** `fase-4/ag-observe` mergeado a `fase-4` (ag-auth usa `tracing`).

**Tech Stack:** `webauthn-rs`, `oauth2`, `blake3`, `base64ct`, `jsonwebtoken`, `ring` (ya en workspace).

**Spec:** `docs/superpowers/specs/2026-05-22-fase4-modulos-estandar-design.md` seccion ag-auth.
**Arquitectura:** `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.1.

---

## Mapa de archivos

### Crear
- `crates/ag-auth/Cargo.toml`
- `crates/ag-auth/src/lib.rs` — `AgAuth`, `AuthConfig`, re-exports
- `crates/ag-auth/src/config.rs` — `AuthConfig::from_env()`
- `crates/ag-auth/src/jwt.rs` — firma/verificacion JWT Ed25519, `Claims`
- `crates/ag-auth/src/webauthn.rs` — registro y autenticacion FIDO2
- `crates/ag-auth/src/oauth2_providers.rs` — Google, GitHub preconfigurados
- `crates/ag-auth/src/api_keys.rs` — generacion, hash BLAKE3, verificacion
- `crates/ag-auth/src/sessions.rs` — refresh tokens con rotacion (feature `persistent`)
- `crates/ag-auth/migrations/0001_ag_sessions.sql`
- `crates/ag-auth/migrations/0002_ag_api_keys.sql`
- `crates/ag-auth/migrations/0003_ag_webauthn_credentials.sql`

### Modificar
- `Cargo.toml` (root) — dependencias nuevas + miembro `crates/ag-auth`

---

## Task 1: Cargo.toml y migraciones SQL

- [ ] Agregar a `[workspace.members]`: `"crates/ag-auth"`
- [ ] Agregar a `[workspace.dependencies]`:
  ```toml
  webauthn-rs  = { version = "0.5", features = ["danger-allow-state-serialisation"] }
  oauth2       = "4"
  blake3       = "1"
  base64ct     = { version = "1", features = ["alloc"] }
  testcontainers = "0.23"
  testcontainers-modules = { version = "0.11", features = ["postgres"] }
  ```
- [ ] Crear `crates/ag-auth/Cargo.toml` con features `default = []` y `persistent = ["dep:ag-data"]`
- [ ] Crear las tres migraciones SQL (ver esquemas en spec):
  - `0001_ag_sessions.sql`: tabla `ag_sessions(jti UUID PK, user_id UUID, expires_at TIMESTAMPTZ, revoked BOOLEAN DEFAULT false)`
  - `0002_ag_api_keys.sql`: tabla `ag_api_keys(id UUID PK, user_id UUID, key_hash TEXT, name TEXT, created_at TIMESTAMPTZ, last_used_at TIMESTAMPTZ, revoked BOOLEAN DEFAULT false)`
  - `0003_ag_webauthn_credentials.sql`: tabla `ag_webauthn_credentials(id UUID PK, user_id UUID, credential_id TEXT UNIQUE, public_key BYTEA, counter BIGINT DEFAULT 0, created_at TIMESTAMPTZ)`
- [ ] Verificar: `cargo check -p ag-auth 2>&1 | grep "^error" | head -5`
- [ ] Commit: `chore(auth): Cargo.toml, deps workspace, migraciones SQL`

---

## Task 2: AuthConfig

**Files:** `crates/ag-auth/src/config.rs`

- [ ] TDD: test `auth_config_reads_jwt_secret_from_env` — con `JWT_SECRET_KEY` en env, `AuthConfig::from_env()` no retorna error
- [ ] Correr para verificar que falla
- [ ] Implementar `AuthConfig` con campos: `jwt_private_key_pem: String`, `jwt_public_key_pem: String`, `webauthn_rp_id: String`, `webauthn_rp_origin: String`, `oauth_google_client_id/secret: Option<String>`, `oauth_github_client_id/secret: Option<String>`
- [ ] `from_env()` lee `JWT_PRIVATE_KEY`, `JWT_PUBLIC_KEY`, `WEBAUTHN_RP_ID`, `WEBAUTHN_ORIGIN` de variables de entorno
- [ ] Correr test
- [ ] Commit: `feat(auth): AuthConfig from_env`

---

## Task 3: JWT Ed25519

**Files:** `crates/ag-auth/src/jwt.rs`

- [ ] TDD (unitarios, sin DB):
  - `jwt_sign_and_verify_roundtrip` — firmar claims, verificar, comprobar que `sub` coincide
  - `jwt_expired_token_returns_error` — crear token con exp en el pasado, verificar retorna `Err`
  - `jwt_tampered_token_returns_error` — modificar un byte del signature, verificar retorna `Err`
- [ ] Correr para verificar que fallan
- [ ] Implementar `JwtSigner::new(private_pem, public_pem)`, `sign(claims) -> String`, `verify(token) -> Result<Claims>` usando `jsonwebtoken` con algoritmo `EdDSA`
- [ ] `Claims` con campos estandar: `sub`, `exp`, `iat`, `jti` + campo `role: String`
- [ ] Correr tests
- [ ] Integrar con ag-observe: `tracing::warn!` en verificacion fallida con motivo
- [ ] Commit: `feat(auth): JWT Ed25519 — sign/verify con jsonwebtoken EdDSA`

---

## Task 4: Passkeys / WebAuthn

**Files:** `crates/ag-auth/src/webauthn.rs`

- [ ] TDD (unitarios con mocks de ceremonia):
  - `webauthn_registration_options_are_valid` — `start_registration()` produce challenge no vacio
  - `webauthn_auth_options_are_valid` — `start_authentication()` produce challenge no vacio
- [ ] Correr para verificar que fallan
- [ ] Implementar `WebAuthnManager::new(config)` wrapeando `webauthn_rs::Webauthn`
- [ ] Exponer: `start_registration(user_id, username) -> (RegistrationOptions, PasskeyRegistration)`, `finish_registration(reg_state, response) -> Result<Passkey>`, `start_authentication(credentials) -> (AuthenticationOptions, PasskeyAuthentication)`, `finish_authentication(auth_state, response) -> Result<AuthenticationResult>`
- [ ] Correr tests
- [ ] Commit: `feat(auth): WebAuthn — registro y autenticacion FIDO2`

---

## Task 5: OAuth2 (Google, GitHub)

**Files:** `crates/ag-auth/src/oauth2_providers.rs`

- [ ] TDD (unitarios):
  - `google_auth_url_contains_accounts_google` — `GoogleProvider::new(config).auth_url(state)` produce URL con `accounts.google.com`
  - `github_auth_url_contains_github` — idem para GitHub
- [ ] Correr para verificar que fallan
- [ ] Implementar `GoogleProvider` y `GithubProvider` cada uno con:
  - `new(client_id, client_secret, redirect_url) -> Self`
  - `auth_url(state: &str) -> (Url, CsrfToken)` — URL de redireccion al proveedor
  - `exchange_code(code: &str) -> Result<TokenResponse>` — intercambio de codigo por token
  - `fetch_user_info(access_token: &str) -> Result<OAuthUserInfo>` — email + nombre
- [ ] `OAuthUserInfo { email: String, name: String, provider_id: String }`
- [ ] Correr tests
- [ ] Commit: `feat(auth): OAuth2 providers — Google y GitHub`

---

## Task 6: API Keys

**Files:** `crates/ag-auth/src/api_keys.rs`

- [ ] TDD (unitarios, sin DB):
  - `api_key_hash_is_deterministic` — misma clave produce mismo hash BLAKE3
  - `api_key_verify_returns_true_for_correct_key`
  - `api_key_verify_returns_false_for_wrong_key`
  - `generated_key_has_expected_prefix` — `ApiKey::generate("sk")` produce clave con prefijo `sk_`
- [ ] Correr para verificar que fallan
- [ ] Implementar:
  - `ApiKey::generate(prefix: &str) -> (raw_key: String, key_hash: String)` — genera 32 bytes aleatorios, los codifica en base64url, prefija con `{prefix}_`, hashea con BLAKE3
  - `ApiKey::verify(raw_key: &str, stored_hash: &str) -> bool`
- [ ] Correr tests
- [ ] Commit: `feat(auth): API keys — generacion y verificacion BLAKE3`

---

## Task 7: Refresh tokens con rotacion (feature `persistent`)

**Files:** `crates/ag-auth/src/sessions.rs`

- [ ] TDD (integracion con testcontainers PostgreSQL):
  - `session_create_and_find` — crear sesion, buscarla por JTI, debe existir
  - `session_rotate_invalidates_old` — rotar token, el JTI anterior debe estar revocado
  - `session_revoke_marks_as_revoked` — revocar, verificar campo `revoked = true`
- [ ] Correr para verificar que fallan (requieren DB)
- [ ] Implementar `SessionStore::new(pool: DbPool)` con:
  - `create(user_id: Uuid, ttl: Duration) -> Result<(jti: Uuid, refresh_token: String)>`
  - `find(jti: Uuid) -> Result<Option<Session>>`
  - `rotate(old_jti: Uuid) -> Result<(new_jti: Uuid, new_token: String)>` — revoca el viejo, crea el nuevo atomicamente en una transaccion
  - `revoke(jti: Uuid) -> Result<()>`
- [ ] Correr tests (levantan contenedor PostgreSQL automaticamente via testcontainers)
- [ ] Commit: `feat(auth): refresh tokens con rotacion — SessionStore sobre PostgreSQL`

---

## Task 8: AgAuth facade y verificacion final

**Files:** `crates/ag-auth/src/lib.rs`

- [ ] Implementar `AgAuth` como facade que agrupa `JwtSigner`, `WebAuthnManager`, `GoogleProvider`, `GithubProvider` y opcionalmente `SessionStore`
- [ ] Metodos principales: `verify_jwt(token: &str) -> Result<Claims>`, `create_api_key(user_id, prefix) -> (raw, hash)`, `verify_api_key(raw, hash) -> bool`
- [ ] `cargo fmt --all && cargo clippy -p ag-auth -- -D warnings`
- [ ] `cargo test -p ag-auth 2>&1 | tail -10`
- [ ] `cargo doc -p ag-auth --no-deps 2>&1 | grep "^error" | head -5`
- [ ] Commit: `feat(auth): AgAuth facade — API publica unificada`
- [ ] Merge: `git checkout fase-4 && git merge --no-ff fase-4/ag-auth -m "feat(auth): ag-auth completo — JWT/WebAuthn/OAuth2/API keys/sessions"`
