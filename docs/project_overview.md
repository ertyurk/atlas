# ATLAS - Core SaaS Framework — Project Requirements Document (Rust + Axum + SurrealDB)

Goal: a **generic, modular, AI-ready** Rust framework that we can reuse across SaaS/e-commerce projects. Vector/semantic search is **owned by modules**; the **core pack** provides the foundation: module lifecycle, HTTP, auth hooks, migrations, logging, errors, events, and observability.

---

## 1) Repository layout

```
Cargo.toml                     # workspace + atlas-app binary wrapper
/src                           # project-specific code (non-reusable)
  main.rs
  /utils                       # helpers unique to this deployment
  /modules
    /books
      /routes
      /models.rs
      /migrations

/crates                        # reusable framework crates
  /kernel                      # traits, module registry, lifecycle, settings, state
  /http                        # axum server, middlewares, openapi, error mappers
  /db                          # surreal client factory, migration runner
  /authz                       # casbin integration, guards (no concrete user storage)
  /telemetry                   # logging facade, tracing, otel, metrics
  /events                      # in-proc event bus, outbox helpers (optional)
  /cli                         # binary: run server, run migrations, dev tools
  /modules                     # example built-ins (purely for samples/tests)
    /demo-auth                 # demo-only: signup/login for testing
    /demo-books                # sample CRUD with validation & errors (see §8)

/config                        # base + environment overlays, Casbin artifacts
/docs                          # requirements, plans, module authoring guides
```

**Workspace** uses `resolver = "2"` and enables `-Zfeatures=it-just-works` if you prefer nightly (optional).

---

## 2) Core capabilities (what “done” means)

1. **Module system**

   * Build-time registration of modules.
   * Each module can: register routes, provide OpenAPI fragments, contribute migrations, hook lifecycle (`init/start/stop`), subscribe/publish events, and add middlewares (scoped).
   * Deterministic bootstrap order; graceful shutdown.

2. **HTTP**

   * Axum router with:

     * global middlewares: request ID, trace, CORS, compression, timeout.
     * per-module route mounting under `/api/{module}`.
   * Auto OpenAPI (Swagger UI at `/docs`) merged from all modules.
   * Unified error mapping to a **required JSON schema** (see §7).

3. **AuthN/AuthZ hooks (minimal in core)**

   * **Extractors** + **guards** only (no user DB): token decoder trait + RBAC via Casbin.
   * Modules are free to implement identity providers.

4. **DB & Migrations**

   * SurrealDB client bootstrap (WS/HTTP selectable).
   * Migration runner that:

     * traverses all modules’ `*.surql` (or embedded strings),
     * stores applied set per `{module, id}` in `_migrations`,
     * runs in stable order and is idempotent.

5. **Telemetry**

   * **Logging facade** (no `println!`). Supports **pretty text** or **structured JSON** with fields:

     * timestamp, level, trace\_id, span\_id, module, route, method, status, latency\_ms, message, fields…
   * OpenTelemetry traces (OTLP) and Prometheus `/metrics`.

6. **Event system**

   * In-proc bus (`broadcast` + `mpsc`) with typed events.
   * Simple outbox pattern interface (optional to turn on).

7. **Error & Log standards**

   * Errors must implement `IntoResponse` and emit:

     ```json
      {
        "error": {
          "code": "validation_error",
          "message": "This is a test validation error to demonstrate the new error format with trace_id and timestamp",
          "details": [
            {
              "field": "slug",
              "error": "required"
            }
          ],
          "trace_id": "3afaa2bb-d0ea-4c2f-82f3-fb7fd2ec4b33",
          "timestamp": "2025-10-01 18:53:32.601016 +00:00:00"
        }
      }
     ```
   * Logging via `logger.warn(...)`, `logger.info(...)`, etc. (facade over `tracing`).

8. **Module boiler plate CLI**

   * Add a Module boilerplate with its folders and initilizers filled in so that start would be easier.

9. **Samples**

   * “Books” demo shows route, validation, unique constraint error, swagger docs, logging, and tests.

---

## 3) Key crates & APIs

### 3.1 `kernel` (module traits, lifecycle, settings, state)

**Traits**

```rust
// crates/kernel/src/module.rs
use async_trait::async_trait;
use axum::Router;
use utoipa::openapi::OpenApi;

pub struct InitCtx<'a> {
  pub settings: &'a crate::settings::Settings,
  pub db: &'a surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
  pub events: &'a crate::events::EventBus,
}

#[async_trait]
pub trait Module: Sync + Send {
  fn name(&self) -> &'static str;

  async fn init(&self, _ctx: &InitCtx<'_>) -> anyhow::Result<()> { Ok(()) }
  fn routes(&self) -> Router { Router::new() }
  fn openapi(&self) -> Option<OpenApi> { None }
  fn migrations(&self) -> Vec<crate::migrations::Migration> { vec![] }

  async fn start(&self, _ctx: &InitCtx<'_>) -> anyhow::Result<()> { Ok(()) }
  async fn stop(&self) -> anyhow::Result<()> { Ok(()) }
}
```

**Registry**

```rust
// crates/kernel/src/registry.rs
inventory::collect!(Box<dyn Module>);
pub fn all() -> impl Iterator<Item=&'static Box<dyn Module>> {
  inventory::iter::<Box<dyn Module>>
}
```

**Settings & State**

```rust
#[derive(Clone)]
pub struct AppState {
  pub settings: Settings,
  pub db: surrealdb::Surreal<Client>,
  pub events: EventBus,
  pub enforcer: std::sync::Arc<casbin::Enforcer>,
  pub logger: telemetry::Logger, // facade
}
```

### 3.2 `http` (axum server, middlewares, docs, error mappers)

* Build router, mount module routes, merge OpenAPI, expose `/docs`.
* Standard middlewares: request ID, trace, cors, gzip, timeout.
* Map `AppError` → shared JSON error schema.

### 3.3 `db` (surreal client & migrations)

* Connect using settings (addresses, ns/db, creds).
* Migration runner with persistent ledger `_migrations`.

```rust
pub struct Migration {
  pub id: &'static str,
  pub up: &'static str,
}
```

### 3.4 `authz` (guards only)

* Casbin enforcer bootstrap (model from file or embedded).
* Axum extractor `CurrentIdentity` via pluggable `TokenDecoder` trait; modules can implement JWT/PASETO/OIDC.

### 3.5 `telemetry` (logger, tracing, otel, metrics)

* **Logger facade**: ergonomic API over `tracing` that **must be used** (no `println!`).

```rust
pub struct Logger;
impl Logger {
  pub fn info<T: Into<String>>(&self, msg: T, fields: impl Into<tracing::ValueSet>) { /* … */ }
  pub fn warn<T: Into<String>>(&self, msg: T, fields: impl Into<tracing::ValueSet>) { /* … */ }
  pub fn error<T: Into<String>>(&self, msg: T, fields: impl Into<tracing::ValueSet>) { /* … */ }
  pub fn debug<T: Into<String>>(&self, msg: T, fields: impl Into<tracing::ValueSet>) { /* … */ }
}
```

* Config:

  * `log.format = "pretty" | "json"`
  * `log.level = "info" | "debug" | ...`
  * `otel.enabled = true|false`, `otlp.endpoint = "..."`
  * `metrics.prometheus.enabled = true|false`, path `/metrics`

* Span & log fields automatically include: `trace_id`, `span_id`, `module`, `route`, `method`, `status`, `latency_ms`.

### 3.6 `events` (in-proc bus, typed)

```rust
pub enum DomainEvent {
  BookCreated { id: String, slug: String },
  // …
}
pub struct EventBus { /* broadcast::Sender<DomainEvent> */ }
```

### 3.7 `cli` (dev ergonomics)

* Subcommands:

  * `server` (runs migrations → starts modules → http)
  * `migrate up` (collect & run)
  * `migrate plan` (preview)
  * `enforcer check` (optional)

---

## 4) Configuration (typed)

Example `Settings`:

```toml
[server]
host = "0.0.0.0"
port = 8080
request_timeout_ms = 15000

[db]
url = "ws://127.0.0.1:8000"
namespace = "core"
database = "core"
username = "root"
password = "root"

[log]
format = "json"   # "pretty" | "json"
level  = "info"

[otel]
enabled  = true
endpoint = "http://otel-collector:4317"

[metrics]
prometheus = true
```

---

## 5) Middleware & request lifecycle

* **TraceLayer**: start span per request with route template.
* **RequestID**: generate UUIDv7; add header `x-request-id`.
* **Auth extractor** (optional): sets `CurrentIdentity` if token present.
* **RBAC guard**: check Casbin policy when applied.
* **Compression, CORS, Timeout**: enabled globally.

---

## 6) OpenAPI (Swagger)

* Each module may return a `utoipa::OpenApi` piece; core merges.
* Swagger UI served at `/docs`, spec at `/docs/openapi.json`.
* Route macros or derive models in modules to auto-generate.

---

## 7) Error model (mandatory)

**Wire format**

```json
{
  "details": [ /* arbitrary records (objects or strings) */ ],
  "message": "string",
  "code": "string"
}
```

**Rust shape & mapper**

```rust
#[derive(serde::Serialize)]
pub struct ErrorBody {
  pub details: Vec<serde_json::Value>,
  pub message: String,
  pub code: String,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
  #[error("validation error")]
  Validation { details: Vec<serde_json::Value>, code: String, message: String },

  #[error("conflict")]
  Conflict { details: Vec<serde_json::Value>, code: String, message: String },

  #[error("not found")]
  NotFound { message: String, code: String },

  #[error("unauthorized")]
  Unauthorized { message: String, code: String },

  #[error(transparent)]
  Internal(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for AppError {
  fn into_response(self) -> axum::response::Response {
    use axum::{Json, http::StatusCode};
    let (status, body) = match self {
      AppError::Validation{details, code, message} =>
        (StatusCode::UNPROCESSABLE_ENTITY, ErrorBody{details, code, message}),
      AppError::Conflict{details, code, message} =>
        (StatusCode::CONFLICT, ErrorBody{details, code, message}),
      AppError::NotFound{message, code} =>
        (StatusCode::NOT_FOUND, ErrorBody{details: vec![], code, message}),
      AppError::Unauthorized{message, code} =>
        (StatusCode::UNAUTHORIZED, ErrorBody{details: vec![], code, message}),
      AppError::Internal(e) => {
        let body = ErrorBody{
          details: vec![serde_json::json!({"reason": e.to_string()})],
          message: "Internal server error".into(),
          code: "internal_error".into(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, body)
      }
    };
    (status, Json(body)).into_response()
  }
}
```

**Usage example (thrower)**

```rust
return Err(AppError::Conflict{
  details: vec![serde_json::json!({"slug": slug})],
  message: format!("Slug: {} is already exist.", slug),
  code: "slug_already_exist".into(),
});
```

---

## 8) Sample module: Books (CRUD + validation + logging)

**Routes**

* `POST /api/books` — create
* `GET  /api/books/:id` — read
* `GET  /api/books` — list (with pagination)
* `PUT  /api/books/:id` — update
* `DELETE /api/books/:id` — delete

**DTO**

```rust
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateBook {
  pub name: String,
  pub slug: String,
  pub author: String,
}
```

**Create handler (essentials)**

```rust
pub async fn create_book(
  State(state): State<AppState>,
  Json(input): Json<CreateBook>
) -> Result<Json<Book>, AppError> {
  // basic validation
  if input.slug.trim().is_empty() {
    return Err(AppError::Validation{
      details: vec![serde_json::json!({"slug": "required"})],
      message: "Validation failed".into(),
      code: "validation_error".into(),
    });
  }

  // uniqueness check (Surreal)
  let q = "SELECT * FROM book WHERE slug = $slug LIMIT 1;";
  let exists = state.db.query(q).bind(("slug", &input.slug)).await?;
  if exists.take::<Option<serde_json::Value>>(0)?.is_some() {
    return Err(AppError::Conflict{
      details: vec![serde_json::json!({"slug": &input.slug})],
      message: format!("Slug: {} is already exist.", &input.slug),
      code: "slug_already_exist".into(),
    });
  }

  // insert
  let up = "CREATE book CONTENT $data;";
  let created = state.db
    .query(up)
    .bind(("data", &input))
    .await?
    .take::<Book>(0)?;

  // log with route/module fields auto-injected by the span
  state.logger.info("book.created", tracing::field::display(&created.slug));

  // event
  state.events.publish(DomainEvent::BookCreated { id: created.id.clone(), slug: created.slug.clone() });

  Ok(Json(created))
}
```

**Swagger snippet**

```rust
#[utoipa::path(
  post,
  path = "/api/books",
  request_body = CreateBook,
  responses(
    (status = 201, body = Book),
    (status = 409, body = ErrorBody, description = "Slug exists"),
    (status = 422, body = ErrorBody, description = "Validation error"),
  )
)]
```

**Migration example**

```sql
-- crates/modules/demo-books/migrations/0001_init.surql
DEFINE TABLE book SCHEMAFULL;
DEFINE FIELD name   ON book TYPE string ASSERT $value != "";
DEFINE FIELD slug   ON book TYPE string ASSERT $value != "";
DEFINE FIELD author ON book TYPE string ASSERT $value != "";
DEFINE INDEX book_slug_unique ON book FIELDS slug UNIQUE;
```

---

## 9) Logger requirements (facade over `tracing`)

**Requirements**

* No `println!` in codebase.
* Provide `logger.info/warn/error/debug`.
* Configurable output: `"pretty"` or `"json"`.
* Auto fields: `trace_id`, `span_id`, `module`, `route`, `method`, `status`, `latency_ms`.
* Support custom key/value fields.

**Bootstrap**

```rust
pub fn init_logger(cfg: &LogConfig) -> Logger {
  // build tracing subscriber with fmt layer:
  // - EnvFilter(cfg.level)
  // - fmt.pretty() or fmt.json()
  // - with current_span + otel context
  // - add custom layer to inject module/route fields
  Logger {}
}
```

**Usage in handlers/services**

```rust
state.logger.warn(
  "books.slug_conflict",
  tracing::field::display(format!("slug={}", input.slug))
);
```

---

## 10) Lifecycle & bootstrap order

1. Load `Settings`, init `telemetry` (logger, otel, metrics).
2. Connect SurrealDB.
3. Init `EventBus`.
4. Init Casbin enforcer (policy store; empty rules allowed).
5. **Module `init()`** (schema hints, policy registration).
6. Run **migrations** (collect → plan → apply).
7. **Module `start()`** (background tasks).
8. Build router; serve HTTP.
9. On shutdown: `module.stop()` → flush traces → close DB.

---

## 11) Acceptance criteria (checklist)

* [ ] Module trait & registry with two demo modules registered.
* [ ] Axum server mounts module routers under `/api/{module}`.
* [ ] Swagger UI at `/docs` with merged OpenAPI.
* [ ] SurrealDB client factory; migration ledger `_migrations`.
* [ ] Migration runner executes per-module migrations once.
* [ ] Telemetry: logger facade with pretty/json, route/module fields; no `println!`.
* [ ] OTel traces to OTLP; Prometheus `/metrics`.
* [ ] Error model conforms exactly to `{details[], message, code}` for all mapped errors.
* [ ] Event bus publishes & listens in demo code.
* [ ] Sample Books module demonstrates validation, conflict error, logging, events, docs.
* [ ] CLI: `server`, `migrate up`, `migrate plan`.

---

## 12) Dependencies (concise)

* runtime: `tokio`, `axum`, `tower`, `tower-http`
* registry: `inventory`, `async-trait`, `once_cell`
* db: `surrealdb`
* config: `config` or `figment`, `dotenvy`
* errors: `anyhow`, `thiserror`
* docs: `utoipa`, `utoipa-axum`, `utoipa-swagger-ui`
* telemetry: `tracing`, `tracing-subscriber`, `tracing-opentelemetry`, `opentelemetry-otlp`, `metrics`, `metrics-exporter-prometheus`
* authz: `casbin`
* misc: `serde`, `serde_json`, `uuid`, `time`

---

## 13) Notes about vector/semantic search

* **Core** does **not** define vector/semantic primitives.
* Modules may:

  * Define vector fields and indexes in their migrations.
  * Expose `/index` or `/search` routes.
  * Plug external embedding/reranker services.
* Core remains agnostic and only provides DB, events, logging, and errors.

---

If you want, I can convert this PRD directly into a ready-to-run workspace skeleton with the Books sample wired up (routes, migrations, OpenAPI, logger, error mapping) so you can `cargo run -p cli -- server` and see `/docs` + the error shapes immediately.
