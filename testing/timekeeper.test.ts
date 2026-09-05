import { beforeAll, describe, expect, test } from "bun:test";
import { Temporal } from "@js-temporal/polyfill";
import { accounts, type Account } from "./helpers/account";
import { expectStatus, timeout, timekeeperApi } from "./helpers/api";
import type { components } from "./types/timekeeper";

const splitInput = (): components["schemas"]["InsertablePackagedTimeSplit"] => ({
  time_split: { name: `split-${crypto.randomUUID()}`, description: "Focus and rest" },
  timers: [{ name: "Focus", len: "PT25M", work: true }, { name: "Rest", len: "PT5M", work: false }],
});
const timerInput = (id: number, tags: number[] = []): components["schemas"]["InsertableTimer"] => ({
  time_split_timer: id, tags, start_time: Date.now() * 1000, end_time: (Date.now() + 1_500_000) * 1000,
});

describe("timekeeper", () => {
  const createAccount = accounts();
  let owner: Account;
  let other: Account;
  beforeAll(async () => { owner = await createAccount(); other = await createAccount(); }, timeout);

  for (const endpoint of ["tags", "splits", "timers"] as const) {
    test(`GET ${endpoint} allows anonymous reads`, async () => {
      const response = await timekeeperApi()[endpoint]({});
      expect(response.status).toBe(200);
      expect(Array.isArray(response.data)).toBe(true);
    });
  }

  const operations = {
    "GET tags": (api: ReturnType<typeof timekeeperApi>) => api.tags({}),
    "POST tag": (api: ReturnType<typeof timekeeperApi>) => api.createTag({ name: `tag-${crypto.randomUUID()}` }),
    "PUT tag": (api: ReturnType<typeof timekeeperApi>) => api.updateTag({ id: -1, name: "unused" }),
    "DELETE tag": (api: ReturnType<typeof timekeeperApi>) => api.deleteTag({ id: -1 }),
    "GET splits": (api: ReturnType<typeof timekeeperApi>) => api.splits({}),
    "POST split": (api: ReturnType<typeof timekeeperApi>) => api.createSplit(splitInput()),
    "PUT split": (api: ReturnType<typeof timekeeperApi>) => api.updateSplit({ id: -1, name: "unused" }),
    "DELETE split": (api: ReturnType<typeof timekeeperApi>) => api.deleteSplit({ id: -1 }),
    "GET timers": (api: ReturnType<typeof timekeeperApi>) => api.timers({}),
    "POST timer": (api: ReturnType<typeof timekeeperApi>) => api.createTimer(timerInput(-1)),
    "PUT timer": (api: ReturnType<typeof timekeeperApi>) => api.updateTimer(timerInput(-1)),
  };
  for (const [name, request] of Object.entries(operations)) {
    test(`${name} rejects invalid JWTs`, () => expectStatus(request(timekeeperApi("invalid")), 401));
  }
  for (const name of ["POST tag", "POST split", "POST timer", "PUT timer"] as const) {
    test(`${name} requires authentication`, () => expectStatus(operations[name](timekeeperApi()), 401));
  }

  test("tag CRUD persists changes, hides deleted tags and isolates accounts", async () => {
    const api = owner.timekeeper;
    const name = `tag-${crypto.randomUUID()}`;
    const { data: { id }, status } = await api.createTag({ name });
    expect(status).toBe(200);
    expect(Number.isInteger(id)).toBe(true);
    try {
      expect((await api.tags({})).data).toContainEqual({ id, name });
      await expectStatus(api.createTag({ name }), 422);
      expect((await other.timekeeper.tags({})).data.some(tag => tag.id === id)).toBe(false);
      await other.timekeeper.updateTag({ id, name: `foreign-${crypto.randomUUID()}` });
      await other.timekeeper.deleteTag({ id });
      expect((await api.tags({})).data).toContainEqual({ id, name });
      const updated = `${name}-updated`;
      expect((await api.updateTag({ id, name: updated })).status).toBe(200);
      expect((await api.tags({})).data).toContainEqual({ id, name: updated });
      expect((await api.deleteTag({ id })).status).toBe(200);
      expect((await api.tags({})).data.some(tag => tag.id === id)).toBe(false);
    } finally { await api.deleteTag({ id }); }
  }, timeout);

  test("time split CRUD preserves timer order and isolates accounts", async () => {
    const api = owner.timekeeper;
    const input = splitInput();
    const { data: split, status } = await api.createSplit(input);
    const id = split.time_split.id;
    expect(status).toBe(200);
    try {
      expect(split.time_split).toEqual({ id, ...input.time_split });
      expect(split.timers).toHaveLength(2);
      split.timers.forEach((timer, index) => {
        const { len, ...expected } = input.timers[index]!;
        // FIXME: Put this in the expect below this expect.
        // This is a regression in Bun
        expect(Number.isInteger(timer.id)).toBe(true);
        expect(timer).toMatchObject({
          ...expected, order_idx: index, time_split_id: id,
        });
        // Equivalent ISO 8601 durations may use different units (PT25M vs PT1500S).
        expect(Temporal.Duration.compare(timer.len, len)).toBe(0);
      });
      expect((await api.splits({})).data).toContainEqual(split);
      expect((await other.timekeeper.splits({})).data.some(s => s.time_split.id === id)).toBe(false);
      await other.timekeeper.updateSplit({ id, name: "foreign" });
      await other.timekeeper.deleteSplit({ id });
      expect((await api.splits({})).data).toContainEqual(split);
      const update = { id, name: "Updated split", description: "Updated description" };
      expect((await api.updateSplit(update)).status).toBe(200);
      expect((await api.splits({})).data.find(s => s.time_split.id === id)).toEqual({
        time_split: update, timers: split.timers,
      });
      expect((await api.deleteSplit({ id })).status).toBe(200);
      expect((await api.splits({})).data.some(s => s.time_split.id === id)).toBe(false);
      await expectStatus(api.createTimer(timerInput(split.timers[0]!.id)), 404);
    } finally { await api.deleteSplit({ id }); }
  }, timeout);

  test("timer creation, latest selection, pause update, duplicate and missing split errors", async () => {
    const account = await createAccount();
    const api = account.timekeeper;
    expect((await api.timers({})).data).toEqual([]);
    const { data: split } = await api.createSplit(splitInput());
    let tagId: number | undefined;
    try {
      const { data: tag } = await api.createTag({ name: `timer-tag-${crypto.randomUUID()}` });
      tagId = tag.id;
      const input = timerInput(split.timers[0]!.id, [tag.id]);
      const expected = { ...input, time_split_data: { time_split_id: split.time_split.id, work: true } };
      expect((await api.createTimer(input)).data).toEqual(expected);
      expect((await api.timers({})).data).toEqual([expected]);
      await expectStatus(api.createTimer(input), 422);
      const later = { ...input, start_time: input.start_time + 1000 };
      const latest = { ...expected, ...later };
      await api.createTimer(later);
      expect((await api.timers({})).data).toEqual([latest]);
      const paused = { ...later, end_time: later.start_time + 500, tags: [] };
      expect((await api.updateTimer(paused)).data).toEqual({ ...latest, ...paused });
      expect((await api.timers({})).data).toEqual([{ ...latest, ...paused }]);
      expect((await other.timekeeper.timers({})).data).toEqual([]);
      await expectStatus(api.createTimer(timerInput(-1)), 404);
      await expectStatus(api.updateTimer({ ...paused, time_split_timer: -1 }), 404);
    } finally {
      if (tagId !== undefined) await api.deleteTag({ id: tagId });
      await api.deleteSplit({ id: split.time_split.id });
    }
  }, timeout);
});
