# Service endpoint tests

Bun integration tests exercise all 17 operations in `types/` using
`openapi-typescript-fetch`. They make real HTTP requests to the Rust services,
PostgreSQL, Ory Kratos, and Garage; they are not isolated Rust unit tests.

Start the development stack from the repository root with
`docker compose up --build -d`, then run:

```sh
cd testing
bun install
bun run typecheck
bun test
```

Run individual services with `bun test user.test.ts`, `bun test timekeeper.test.ts`,
or `bun test timesync.test.ts`. Tests do not skip unavailable dependencies.
Use a disposable development database: profile and timesheet rows have no delete
endpoint and remain after a run. Tags and splits are soft-deleted during cleanup.
The tests only modify their own uniquely named fixtures.

## Configuration

Bun loads environment variables from `.env`. All URLs should omit trailing slashes.

| Variable | Default | Purpose |
| --- | --- | --- |
| `API_BASE_URL` | `http://localhost:8000` | Kong gateway |
| `USER_BASE_URL` | `API_BASE_URL` | Direct user origin; paths already include `/user` |
| `TIMEKEEPER_BASE_URL` | `API_BASE_URL` | Direct timekeeper origin; paths already include `/timekeeper` |
| `TIMESYNC_BASE_URL` | `API_BASE_URL/timesync` | Timesync base; tests append `/` |
| `ORY_PUBLIC_URL` | `API_BASE_URL` | Native registration and session tokenization |
| `ORY_ADMIN_URL` | `http://localhost:4434` | Delete the test identities after each suite |
| `ORY_TOKEN_TEMPLATE` | `default_template` | Configured session JWT template |
| `CLOCK_TOLERANCE_MS` | `5000` | Allowed client/server clock difference |

The Ory helper uses `@ory/client-fetch` to register fresh email/password accounts,
obtains the native session token (including the `continue_with` variant), and
exchanges it for a JWT with `toSession({ tokenizeAs: "default_template" })`.
The services receive that JWT as a bearer token. Registration requires the
password method and session hook in the checked-in Kratos configuration. JWT
signing keys must match the services' certificate. The admin endpoint must be
reachable for identity cleanup, including when setup fails after registration.
See [Ory session tokenization](https://www.ory.sh/docs/identities/session-to-jwt-cors)
and [Bun test lifecycle hooks](https://bun.sh/docs/test/lifecycle).

Avatar tests upload a real PNG to the presigned Garage URL, retrieve its bytes,
replace it, and delete it. Object-storage requests use native fetch because
presigned URLs are outside the service OpenAPI schemas. Timesync responses omit
Content-Type; the client handles a textual JSON payload without changing its
schema or the service.

## Coverage

- User: profile create/read/update, missing and duplicate profiles, account
  isolation, avatar upload/replace/delete, and missing/invalid authentication
  for all five operations.
- Timekeeper: tag and split create/read/update/soft-delete, timer order,
  cross-account reads and mutations, timer create/read/update, latest selection,
  duplicate timers, unknown/deleted split timers, public reads, and invalid JWTs
  for all eleven operations.
- Timesync: repeated requests, response structure, safe integer microsecond
  timestamps, receive/send ordering and freshness.
- Infrastructure: both OpenAPI JSON documents and Scalar pages. Direct `GET /`
  health checks run when `USER_BASE_URL` and `TIMEKEEPER_BASE_URL` are set to
  direct service origins. Compose does not publish those ports, and Kong sends
  `/` to the frontend, so these two checks otherwise report as skipped.

Each lifecycle test creates its own resources and runs its steps sequentially;
it does not depend on another test having run. Do not use `--concurrent` because
some isolation assertions share an otherwise empty account. Network requests
have a 10-second deadline and account/lifecycle tests have a 30-second deadline.

Regenerate service types against a running instance with
`just create-schema http://localhost:8000`. Generated files are not edited by the
tests. Infrastructure routes are absent from these schemas, so their small
supplemental types live in `infrastructure.test.ts`.

## Live validation findings

The local run finished with **34 passing, 5 failing, and 2 skipped tests**;
`bun run typecheck` passed. It exposed timekeeper database problems: anonymous
tag and timer reads return 500 (table permissions), tag creation returns 422
(null `id`), and split creation returns 422 (sequence permissions on
`time_split_id_seq`). These failures prevent the dependent lifecycle steps from
running. Assertions retain the intended successful behavior; they are not
marked as expected failures. User and timesync checks passed on that stack.
