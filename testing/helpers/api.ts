import { expect } from "bun:test";
import { ApiError, Fetcher, type Middleware } from "openapi-typescript-fetch";
import type { paths as UserPaths } from "../types/user";
import type { paths as TimekeeperPaths } from "../types/timekeeper";
import type { paths as TimesyncPaths } from "../types/timesync";

export const baseUrl = (process.env.API_BASE_URL ?? "http://localhost:8000").replace(/\/$/, "");
export const timeout = 30_000;
const deadline: Middleware = (url, init, next) => next(url, {
  ...init, signal: AbortSignal.timeout(10_000),
});
export function userApi(token?: string) {
  const client = Fetcher.for<UserPaths>();
  client.configure({ baseUrl: process.env.USER_BASE_URL ?? baseUrl, use: [deadline],
    init: { headers: token ? { Authorization: `Bearer ${token}` } : {} } });
  return {
    get: client.path("/user/profile").method("get").create(),
    create: client.path("/user/profile").method("post").create(),
    update: client.path("/user/profile").method("put").create(),
    avatar: client.path("/user/profile/avatar").method("put").create(),
    deleteAvatar: client.path("/user/profile/avatar").method("delete").create(),
  };
}
export function timekeeperApi(token?: string) {
  const client = Fetcher.for<TimekeeperPaths>();
  client.configure({ baseUrl: process.env.TIMEKEEPER_BASE_URL ?? baseUrl, use: [deadline],
    init: { headers: token ? { Authorization: `Bearer ${token}` } : {} } });
  return {
    tags: client.path("/timekeeper/tag").method("get").create(),
    createTag: client.path("/timekeeper/tag").method("post").create(),
    updateTag: client.path("/timekeeper/tag/{id}").method("put").create(),
    deleteTag: client.path("/timekeeper/tag/{id}").method("delete").create(),
    splits: client.path("/timekeeper/time-split").method("get").create(),
    createSplit: client.path("/timekeeper/time-split").method("post").create(),
    updateSplit: client.path("/timekeeper/time-split/{id}").method("put").create(),
    deleteSplit: client.path("/timekeeper/time-split/{id}").method("delete").create(),
    timers: client.path("/timekeeper/timer").method("get").create(),
    createTimer: client.path("/timekeeper/timer").method("post").create(),
    updateTimer: client.path("/timekeeper/timer").method("put").create(),
  };
}
const sync = Fetcher.for<TimesyncPaths>();
sync.configure({ baseUrl: process.env.TIMESYNC_BASE_URL ?? `${baseUrl}/timesync`, use: [deadline,
  async (url, init, next) => {
    const response = await next(url, init);
    // The TCP service deliberately omits Content-Type, so the library returns text.
    if (typeof response.data === "string") return { ...response, data: JSON.parse(response.data) };
    return response;
  },
] });
export const timeSync = sync.path("/").method("get").create();
export async function expectStatus(request: Promise<unknown>, status: number) {
  try { await request; } catch (error) {
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).status).toBe(status);
    return;
  }
  throw new Error(`Expected HTTP ${status}, but request succeeded`);
}
