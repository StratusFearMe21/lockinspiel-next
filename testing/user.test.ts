import { beforeAll, describe, expect, test } from "bun:test";
import { accounts, type Account } from "./helpers/account";
import { expectStatus, timeout, userApi } from "./helpers/api";

const profile = { display_name: "Bun endpoint test 🕒", bio: "Testing profile persistence" };
describe("user", () => {
  const createAccount = accounts();
  let owner: Account;
  beforeAll(async () => { owner = await createAccount(); }, timeout);

  for (const [name, request] of Object.entries({
    "GET profile": (api: ReturnType<typeof userApi>) => api.get({}),
    "POST profile": (api: ReturnType<typeof userApi>) => api.create(profile),
    "PUT profile": (api: ReturnType<typeof userApi>) => api.update(profile),
    "PUT avatar": (api: ReturnType<typeof userApi>) => api.avatar({ file_extension: "png" }),
    "DELETE avatar": (api: ReturnType<typeof userApi>) => api.deleteAvatar({}),
  })) {
    test(`${name} requires authentication`, () => expectStatus(request(userApi()), 401));
    test(`${name} rejects an invalid JWT`, () => expectStatus(request(userApi("invalid")), 401));
  }

  test("profile creation, retrieval, duplicate rejection, update and account isolation", async () => {
    const account = await createAccount();
    await expectStatus(account.user.get({}), 404);
    expect((await account.user.create(profile)).status).toBe(200);
    expect((await account.user.get({})).data).toEqual({ ...profile, user_id: account.id, avatar_location: null });
    await expectStatus(account.user.create(profile), 422);
    const updated = { display_name: "Renamed", bio: "" };
    expect((await account.user.update(updated)).status).toBe(200);
    expect((await account.user.get({})).data).toMatchObject({ ...updated, user_id: account.id });
    await expectStatus(owner.user.get({}), 404);
  }, timeout);

  test("avatar operations require an existing profile", async () => {
    await expectStatus(owner.user.avatar({ file_extension: "png" }), 404);
    await expectStatus(owner.user.deleteAvatar({}), 404);
  });

  test("avatar upload, replacement and deletion persist and serve the uploaded bytes", async () => {
    const account = await createAccount();
    await account.user.create(profile);
    // A real one-pixel PNG; presigned object-storage URLs are outside the service schema.
    const png = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=", "base64");
    try {
      for (let replacement = 0; replacement < 2; replacement++) {
        const { data, status } = await account.user.avatar({ file_extension: "png" });
        expect(status).toBe(200);
        if (!data || typeof data !== "object" || !("url" in data) || typeof data.url !== "string") {
          throw new Error("Avatar response must contain a URL string");
        }
        expect(new URL(data.url).pathname).toContain(`avatars/${account.id}.png`);
        const upload = await fetch(data.url, { method: "PUT", body: png,
          headers: { "Content-Type": "image/png" }, signal: AbortSignal.timeout(10_000) });
        expect(upload.ok).toBe(true);
        const avatar = (await account.user.get({})).data.avatar_location;
        expect(avatar).toBeTruthy();
        const download = await fetch(avatar!, { signal: AbortSignal.timeout(10_000) });
        expect(download.status).toBe(200);
        expect(Buffer.from(await download.arrayBuffer())).toEqual(png);
      }
      expect((await account.user.deleteAvatar({})).status).toBe(200);
      expect((await account.user.get({})).data.avatar_location).toBeNull();
      expect((await account.user.deleteAvatar({})).status).toBe(200);
    } finally { await account.user.deleteAvatar({}); }
  }, timeout);
});
