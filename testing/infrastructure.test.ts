import { expect, test } from "bun:test";
import { Fetcher } from "openapi-typescript-fetch";
import { baseUrl } from "./helpers/api";

// These infrastructure routes are deliberately absent from the generated API schemas.
// Keep their supplemental response types separate from generated files.
type Get<T> = { get: { responses: { 200: { content: { "application/json": T } } } } };
type InfrastructurePaths = {
  "/": Get<string>;
  "/user/openapi": Get<string>;
  "/timekeeper/openapi": Get<string>;
  "/user/openapi/json": Get<{ openapi: string; paths: Record<string, unknown> }>;
  "/timekeeper/openapi/json": Get<{ openapi: string; paths: Record<string, unknown> }>;
};
for (const service of ["user", "timekeeper"] as const) {
  const directUrl = process.env[`${service.toUpperCase()}_BASE_URL`];
  const client = Fetcher.for<InfrastructurePaths>();
  client.configure({ baseUrl: directUrl ?? baseUrl, use: [
    (url, init, next) => next(url, { ...init, signal: AbortSignal.timeout(10_000) }),
  ] });
  test(`${service} serves its OpenAPI document`, async () => {
    const result = await client.path(`/${service}/openapi/json`).method("get").create()({});
    expect(result.status).toBe(200);
    expect(result.data.openapi).toMatch(/^3\./);
    expect(Object.keys(result.data.paths)).toContain(service === "user" ? "/user/profile" : "/timekeeper/timer");
  });
  test(`${service} serves its Scalar API documentation`, async () => {
    const result = await client.path(`/${service}/openapi`).method("get").create()({});
    expect(result.status).toBe(200);
    expect(result.data.toLowerCase()).toContain("<html");
    expect(result.data.toLowerCase()).toContain("scalar");
  });
  // Kong routes / to the frontend. Health checks need a direct service address.
  test.skipIf(!directUrl)(`${service} GET / reports healthy`, async () => {
    const result = await client.path("/").method("get").create()({});
    expect(result.status).toBe(200);
    expect(result.data).toBe("up");
  });
}
