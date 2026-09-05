# Repository Guidelines

## Project Structure & Module Organization

The Rust 2024 workspace lives in `rust/`, with its manifest and lockfile at that level.
- `rust/crates/backend/`: user, timekeeper, and timesync services, plus shared authentication, configuration, and telemetry code in `lockinspiel-backend-common`.
- `rust/crates/schemas/`: shared types and Diesel database schemas; service schema crates contain SQL `migrations/`.
- `rust/crates/utoipa-e2e/`: procedural macros and an API example.
- `rust/crates/backend/lockinspiel-user/static/`: SVG assets.
- `testing/`: Bun TypeScript endpoint integration tests; `helpers/` contains typed API clients and Ory account setup, and `types/` contains generated OpenAPI declarations.
- `docker-compose.yaml`, `docker/`, `postgres/`, and `volumes/`: local infrastructure, initialization scripts, and service configuration.
- `.github/workflows/docker-publish.yml`: container build and publishing workflow.

## Build, Test, and Development Commands

Run Cargo commands from `rust/`:
- `cargo build --workspace`: compile all workspace crates.
- `cargo run -p lockinspiel-timesync`: run the timesync service locally.
- `cargo test --workspace`: run workspace tests and documentation tests.
- `cargo bench -p lockinspiel-timesync --bench benchmark`: run the Criterion request/response benchmark.
- `cargo fmt --all -- --check`: check Rust formatting.
- `cargo clippy --workspace --all-targets`: check for common Rust mistakes.

From the repository root, use `docker compose up --build` to build and start the configured development stack. Consult Compose environment settings before running database-backed services directly.

Run endpoint-test commands from `testing/` against the running development stack:
- `bun install`: install test dependencies.
- `bun run typecheck`: check TypeScript without emitting files.
- `bun test`: run all service endpoint tests.
- `bun test timekeeper.test.ts`: run one service's tests (also `user.test.ts` and `timesync.test.ts`).
- `just create-schema http://localhost:8000`: regenerate OpenAPI types after API changes.

## Coding Style & Naming Conventions

Use standard rustfmt formatting with four-space indentation. Name modules and functions `snake_case`, types and traits `PascalCase`, and constants `SCREAMING_SNAKE_CASE`. Follow existing `lockinspiel-*` crate names. Keep shared backend behavior in the common crate and database types in schema crates. Update migrations and API documentation alongside persistence or route changes.

## Testing Guidelines

Use Bun's built-in `bun:test` framework in `testing/*.test.ts` for endpoint behavior. These are integration tests against real services, PostgreSQL, Ory Kratos, and Garage. Cover successful request lifecycles, persisted results, validation errors, authentication, and account isolation when adding or changing endpoints. Keep focused Rust unit tests in `#[cfg(test)]` modules and Rust integration tests in each crate's `tests/` directory; Criterion benchmarks remain available for timesync.

- Make service requests with `openapi-typescript-fetch` and the generated declarations in `testing/types/`. Reuse `testing/helpers/api.ts`; regenerate types instead of editing generated files. Infrastructure routes absent from the schemas use supplemental types in `infrastructure.test.ts`. Native `fetch` is appropriate for presigned object-storage URLs outside the service schemas.
- Use `testing/helpers/account.ts` and `@ory/client-fetch` to register unique test accounts. Exchange the native session token for a JWT using the configured Ory tokenizer template, then send that JWT as bearer authentication to services. Register identity cleanup before setup and delete identities through the Ory admin API after the suite.
- Create uniquely named fixtures within each lifecycle test, run dependent steps sequentially, and clean up owned resources in `finally` blocks. Tests must not depend on another test running first. Do not use `--concurrent` with the current shared-account isolation checks.
- Use a disposable development database. Tags and splits are soft-deleted; profile and timesheet rows remain because they have no delete endpoint. Never modify unrelated fixtures to make a test pass.
- Compare ISO 8601 timer lengths semantically with `Temporal.Duration.compare` from `@js-temporal/polyfill`, rather than comparing duration strings. Check numeric IDs separately with `Number.isInteger`: Bun 1.4.0's `toMatchObject` with `expect.any(Number)` was observed to mutate the received ID into a matcher, corrupting subsequent requests.
- Keep request deadlines and lifecycle timeouts bounded (currently 10 and 30 seconds). Unavailable dependencies should fail tests. Direct health checks may skip when direct service URLs are unset, since Kong routes `/` to the frontend.

See `testing/README.md` for URL overrides, authentication requirements, and service setup. Run `bun run typecheck` and relevant Bun tests for test or endpoint changes; run formatting, Clippy, and relevant Rust tests for Rust changes. Report failures and skips accurately rather than weakening assertions to accept defects. Current CI primarily builds containers; no coverage threshold is established.

## Commit & Pull Request Guidelines

History uses short, imperative subjects such as `Port timekeeper microservice to Rust`; follow that style. Keep commits focused. PRs should explain the behavior change, affected services, validation commands and results, and any migration or configuration steps. Link related issues when applicable.

## Security & Configuration

Treat checked-in Compose credentials and keys as development fixtures. Do not add production secrets; supply deployment-specific values through environment configuration or secret management.
