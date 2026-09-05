import { afterAll } from "bun:test";
import { Configuration, FrontendApi, IdentityApi } from "@ory/client-fetch";
import { baseUrl, timeout, timekeeperApi, userApi } from "./api";

const config = (basePath: string) => new Configuration({ basePath,
  fetchApi: (input: string | Request | URL, init?: RequestInit) => fetch(input, { ...init, signal: AbortSignal.timeout(10_000) }),
});
const ory = new FrontendApi(config(process.env.ORY_PUBLIC_URL ?? baseUrl));
const admin = new IdentityApi(config(process.env.ORY_ADMIN_URL ?? "http://localhost:4434"));

// Register cleanup before setup so partially completed setup is also cleaned up.
export function accounts() {
  const ids: string[] = [];
  afterAll(async () => {
    const results = await Promise.allSettled(ids.map(id => admin.deleteIdentity({ id })));
    const failures = results.filter(result => result.status === "rejected");
    if (failures.length) throw new Error(`Failed to delete ${failures.length} test identities`);
  }, timeout);
  return async () => {
    const flow = await ory.createNativeRegistrationFlow();
    const registration = await ory.updateRegistrationFlow({ flow: flow.id,
      updateRegistrationFlowBody: { method: "password", password: `Aa1!${crypto.randomUUID()}`,
        traits: { email: `endpoint-${crypto.randomUUID()}@example.com` } },
    });
    ids.push(registration.identity.id);
    const token = registration.session_token ?? registration.continue_with?.find(
      step => step.action === "set_ory_session_token",
    )?.ory_session_token;
    if (!token) throw new Error("Ory registration did not issue a session token; enable the session hook");
    const session = await ory.toSession({ xSessionToken: token,
      tokenizeAs: process.env.ORY_TOKEN_TEMPLATE ?? "default_template" });
    if (!session.tokenized) throw new Error("Ory did not issue a JWT; check the tokenizer configuration");
    return { id: registration.identity.id, user: userApi(session.tokenized),
      timekeeper: timekeeperApi(session.tokenized) };
  };
}
export type Account = Awaited<ReturnType<ReturnType<typeof accounts>>>;
