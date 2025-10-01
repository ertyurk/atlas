# ATLAS Development Plan

## Scope & Intent
This plan sequences the work required to realize the ATLAS core framework described in `project_overview.md`. It organizes delivery into iterative phases that build the workspace skeleton, ship the core runtime, and validate the sample modules while keeping observability and quality gates in view.

## Guiding Principles
- Keep the workspace buildable at the end of every phase; avoid breaking the CLI entry points.
- Add automated verification (tests, linting, format) alongside features rather than deferring to the end.
- Prefer thin abstractions first; defer advanced ergonomics until the acceptance checklist is satisfied.
- Document crate- and module-level contracts as they are stabilized, especially any public traits.

## Phased Delivery Plan

### Phase 0 – Bootstrap & Tooling Readiness ✅
**Goals**: create the Cargo workspace, shared config scaffolding, and developer tooling.
- ✅ Create `Cargo.toml` workspace at the repo root with framework crates under `/crates`, shared `rust-toolchain.toml` (no nightly).
- ✅ Scaffold the project-specific `/src` tree (entrypoint, `utils/`, `modules/`) to host custom business logic separate from reusable generic core crates.
- ⚠️ Wire `cargo fmt`, `cargo clippy`, and a basic CI workflow (lint + test matrix). *[Partial - tooling exists but CI not implemented]*
- ✅ Establish `.env` loading, base `Settings` struct, and configuration file structure, including environment overlays for local, staging, and production (SurrealDB endpoints, telemetry exporters, auth policies).
- ✅ Deliverable: repository builds with `cargo check` across crates; CI green; sample environment configs committed. *[Builds successfully, CI pending]*

### Phase 1 – Kernel & Module Registry ✅
**Goals**: implement the module trait, inventory-based registration, lifecycle contexts, and deterministic ordering.
- ✅ Flesh out `crates/kernel` with `Module` trait, lifecycle hooks, settings/state structs, and registry loader.
- ⚠️ Add compile-time feature flags per module so the binary only links opted-in modules while keeping inventory registration deterministic. *[Registry implemented but feature flags not yet added]*
- ✅ Provide integration tests that register two dummy modules and assert `init/start/stop` sequencing, covering feature-flag toggles. *[Basic tests exist, feature flag tests pending]*
- ✅ Define error types for module bootstrap failures and surface them via `anyhow::Result` wrappers.
- ✅ Deliverable: `kernel` crate exposes stable API, registry unit/integration tests passing with and without module feature flags enabled. *[API stable, feature flag tests pending]*

### Phase 2 – HTTP Surface & Error Model ❌
**Goals**: stand up the Axum server crate and enforce uniform error responses.
- ❌ Build `crates/http` with router builder, global middlewares, per-module mount helpers, and OpenAPI aggregation hooks. *[Placeholder only]*
- ❌ Implement error response mapper returning `{details, message, code}`; add tests covering success, 4xx, 5xx. *[Not implemented]*
- ❌ Host Swagger UI at `/docs`; validate merging via sample OpenAPI fragments from dummy modules. *[Not implemented]*
- ❌ Deliverable: CLI can start an HTTP server serving health route; contract tests cover error schema. *[Not implemented]*

### Phase 3 – Persistence & Migrations ❌
**Goals**: connect SurrealDB and support module-scoped migrations.
- ❌ Implement connection factory with configurable WS/HTTP modes and health checks. *[Placeholder only]*
- ❌ Build migration planner/runner storing ledger rows in `_migrations`; ensure idempotency. *[Not implemented]*
- ⚠️ Provide fixture migrations for demo modules and integration test the ledger behavior using an embedded/temporary Surreal instance or harness mock. *[Sample migration exists but runner not implemented]*
- ❌ Deliverable: `cli migrate plan/up` operate against local SurrealDB; migration tests green. *[Not implemented]*
- ❌ Deliverable 2: `cli migrate generate {moduleName}` to generate a file for the module specific migrations file name to avoid collusions. *[Not implemented]*


### Phase 4 – Telemetry & Observability ❌
**Goals**: deliver logging facade, tracing, and metrics pipeline.
- ❌ Implement logger init with pretty/json modes, contextual fields, and OTLP exporter wiring. *[Placeholder only]*
- ❌ Add Prometheus `/metrics` endpoint via `tower_http::trace` + `metrics-exporter-prometheus`. *[Not implemented]*
- ⚠️ Instrument HTTP handlers with traces and latency metrics; ensure no `println!` usage remains. *[Basic tracing exists but no HTTP instrumentation]*
- ❌ Deliverable: manual verification via example run showing structured logs and metrics scrape. *[Not implemented]*

### Phase 5 – Auth Hooks & Policy Integration ❌
**Goals**: expose token decoding trait, Casbin guard middleware, and policy bootstrap.
- ❌ Define extractor traits, guard middleware, and error mapping when auth fails. *[Placeholder only]*
- ⚠️ Provide sample Casbin model/policy files and configure hot-reload or startup load. *[Config files exist but not integrated]*
- ❌ Add tests covering authorized vs unauthorized flows in demo routes. *[Not implemented]*
- ❌ Deliverable: demo auth module authenticates requests using stub token provider; policy enforcement tested. *[Not implemented]*

### Phase 6 – Hardening, Docs, and Release Prep ❌
**Goals**: polish developer experience, documentation, and release assets.
- ⚠️ Write module authoring guide, configuration reference, and operations runbook under `docs/`. *[Basic docs exist, detailed guides pending]*
- ❌ Add smoke tests (CLI e2e) and coverage of shutdown path (graceful stop of modules). *[Not implemented]*
- ❌ Prepare release checklist, license audit, and sample deployment manifests (Docker Compose, optional Helm chart). *[Not implemented]*
- ❌ Deliverable: acceptance checklist in `project_overview.md` fully satisfied; docs ready for onboarding. *[Not implemented]*

### Phase 7 – Events, Outbox, and Sample Modules ❌
**Goals**: finalize intra-process event bus, optional outbox interface, and the demo modules.
- ❌ Implement event bus with broadcast + mpsc channels, typed payload support, and background task management. *[Placeholder only]*
- ❌ Stub optional outbox trait to persist events; document how modules can plug storage. *[Not implemented]*
- ⚠️ Build `demo-auth` and `demo-books` modules with routes, migrations, logging, openapi documentation, and tests; ensure modules honour feature-flag gating across build profiles. *[Basic books module exists but incomplete]*
- ❌ Deliverable: CLI server exposes `/api/books` CRUD, emits events, and logs slug conflict handling. *[Not implemented]*

## Cross-Cutting Tasks
- **Testing**: maintain unit + integration suites per crate; introduce contract tests for public traits; add Nix or devcontainer only if requested.
- **CI/CD**: extend initial workflow with caching and SurrealDB service for migration tests by Phase 3.
- **Environments**: enforce parity across local/staging/prod by providing config overlays, secrets loading guidance, and smoke tests targeting each environment profile.
- **Security**: run `cargo audit` in CI; document secrets management for telemetry/auth providers.
- **Documentation**: update `docs/project_overview.md` status checkboxes as phases complete; mirror key decisions in ADRs if scope changes.

## Open Questions / Follow-ups
- Decide whether OTLP + Prometheus exporters run by default or behind feature flags for dev ergonomics.
- Clarify expectations for CLI UX (subcommand ergonomics, config discovery, interactive prompts).
- Evaluate timing and requirements for runtime module discovery beyond compile-time feature flags (e.g., plugin marketplace, tenant-specific deployments) and document trigger conditions if we plan to pursue it later.

Keeping these items visible during execution will help steer scope and ensure the delivered framework aligns with the PRD.
