import { expect, test } from "bun:test";
import { timeSync } from "./helpers/api";

test("GET timesync returns fresh Unix timestamps in microseconds across repeated requests", async () => {
  const tolerance = Number(process.env.CLOCK_TOLERANCE_MS ?? 5000) * 1000;
  for (let i = 0; i < 3; i++) {
    const sent = Date.now() * 1000;
    const { status, data } = await timeSync({});
    const received = Date.now() * 1000;
    expect(status).toBe(200);
    expect(Object.keys(data).sort()).toEqual(["n2", "n3"]);
    for (const timestamp of [data.n2, data.n3]) {
      expect(Number.isSafeInteger(timestamp)).toBe(true);
      expect(timestamp).toBeGreaterThanOrEqual(sent - tolerance);
      expect(timestamp).toBeLessThanOrEqual(received + tolerance);
    }
    expect(data.n3).toBeGreaterThanOrEqual(data.n2);
  }
});
