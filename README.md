# ATLAS (Work in Progress)

This repository contains the early scaffolding for ATLAS, a reusable Rust framework for SaaS/e-commerce backends driven by Axum and SurrealDB. The long-term vision, architecture, and acceptance criteria live in `docs/project_overview.md`.

## Status

⚠️ **Not ready for anything yet**. The codebase is under active design; expect breaking changes, missing features, and incomplete tooling.

## Getting Oriented

- Review `docs/development_plan.md` for the phased delivery roadmap and open questions.
- Follow updates to the acceptance checklist in `docs/project_overview.md` to track progress toward a usable release.
- Framework crates live under `crates/`; run workspace commands from the repo root (e.g. `cargo test`) to build everything.
- Project-specific code now resides in `src/` (`utils/`, `modules/`, `main.rs`) so you can iterate on custom features without touching the reusable crates.


## Local SurrealDB (Docker)

Use `docker-compose.yml` to launch a local SurrealDB instance that mirrors the environments expected by the framework:

```bash
docker compose up -d
```

The service exposes the HTTP interface on `http://localhost:8000` with default credentials `root / root`. Override `SURREAL_USER` and `SURREAL_PASS` in your shell or an `.env` file when you need custom credentials.

Contributions are welcome once the core interfaces stabilize; until then, feedback on the design documents is especially helpful.
