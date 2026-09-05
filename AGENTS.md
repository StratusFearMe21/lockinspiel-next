# Repository Guidelines

## Project Structure & Module Organization

The Rust 2024 workspace lives in `rust/`, with its manifest and lockfile at that level.
- `rust/crates/backend/`: user, timekeeper, and timesync services, plus shared authentication, configuration, and telemetry code in `lockinspiel-backend-common`.
- `rust/crates/schemas/`: shared types and Diesel database schemas; service schema crates contain SQL `migrations/`.
- `rust/crates/utoipa-e2e/`: procedural macros and an API example.
- `rust/crates/backend/lockinspiel-user/static/`: SVG assets.
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

## Coding Style & Naming Conventions

Use standard rustfmt formatting with four-space indentation. Name modules and functions `snake_case`, types and traits `PascalCase`, and constants `SCREAMING_SNAKE_CASE`. Follow existing `lockinspiel-*` crate names. Keep shared backend behavior in the common crate and database types in schema crates. Update migrations and API documentation alongside persistence or route changes.

## Testing Guidelines

There is no established unit-test suite or coverage threshold; Criterion benchmarks exist for timesync. Add focused regression tests for behavior changes using Rust's test harness, with descriptive `snake_case` names. Place unit tests in `#[cfg(test)]` modules and integration tests in each crate's `tests/` directory. Document required database or authentication setup. Run formatting, Clippy, and relevant tests locally; current CI primarily builds containers.

## Commit & Pull Request Guidelines

History uses short, imperative subjects such as `Port timekeeper microservice to Rust`; follow that style. Keep commits focused. PRs should explain the behavior change, affected services, validation commands and results, and any migration or configuration steps. Link related issues when applicable.

## Security & Configuration

Treat checked-in Compose credentials and keys as development fixtures. Do not add production secrets; supply deployment-specific values through environment configuration or secret management.
