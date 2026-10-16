import { expect, test as base, type BrowserContext, type Frame, type Page } from "@playwright/test";

export const projectId = "demo-gomoku2d-acceptance";
const firestoreRoot = `http://127.0.0.1:18081/v1/projects/${projectId}/databases/(default)/documents`;

export const test = base.extend<{ isolatedNetwork: void }>({
  isolatedNetwork: [async ({ context }, use, testInfo) => {
    const external: string[] = [];
    await protectContext(context, external);
    await use();
    await testInfo.attach("unexpected-network.json", { body: JSON.stringify(external), contentType: "application/json" });
    expect(external).toEqual([]);
  }, { auto: true }],
});
export { expect };

export async function protectContext(context: BrowserContext, external?: string[]): Promise<void> {
  await context.route("**/*", async (route) => {
    const url = new URL(route.request().url());
    // The SDK's supported popup/redirect transport needs static GAPI even in
    // emulator mode. Live identity and database endpoints are always blocked.
    if (url.hostname === "apis.google.com" && (url.pathname === "/js/api.js"
      || url.pathname.startsWith("/_/scs/abc-static/"))) return route.continue();
    // Optional emulator-widget decoration, and the SDK's generic connectivity
    // probe, are fulfilled locally. No auth, rule, or write response is mocked.
    if ((url.hostname === "fonts.googleapis.com" && ["/css2", "/icon"].includes(url.pathname))
      || (url.hostname === "unpkg.com" && url.pathname.endsWith(".css"))) {
      return route.fulfill({ body: "", contentType: "text/css" });
    }
    if (url.hostname === "unpkg.com" && url.pathname === "/material-components-web@10/dist/material-components-web.min.js") {
      return route.fulfill({ body: "", contentType: "application/javascript" });
    }
    if (url.hostname === "apis.google.com" && url.pathname === "/js/gen_204") {
      return route.fulfill({ status: 204, body: "" });
    }
    if (url.hostname === "www.google.com" && url.pathname === "/images/cleardot.gif") {
      return route.fulfill({ body: Buffer.from("R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7", "base64"), contentType: "image/gif" });
    }
    if (["http:", "https:"].includes(url.protocol) && !["127.0.0.1", "localhost"].includes(url.hostname)) {
      await route.abort();
      const target = url.origin + url.pathname;
      if (external) external.push(target);
      else throw new Error(`Unexpected external request: ${target}`);
    } else await route.continue();
  });
}

export async function readProfile(uid: string): Promise<Record<string, any> | null> {
  const response = await fetch(`${firestoreRoot}/profiles/${uid}`, { headers: { Authorization: "Bearer owner" } });
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(await response.text());
  const value = await response.json() as { fields: Record<string, unknown> };
  return decodeFields(value.fields);
}

function decodeFields(fields: Record<string, any>): Record<string, any> {
  return Object.fromEntries(Object.entries(fields).map(([key, field]) => [key, decodeValue(field)]));
}
function decodeValue(field: Record<string, any>): any {
  if ("nullValue" in field) return null;
  if ("timestampValue" in field) return field.timestampValue;
  if ("mapValue" in field) return decodeFields(field.mapValue.fields ?? {});
  if ("arrayValue" in field) return (field.arrayValue.values ?? []).map(decodeValue);
  if ("integerValue" in field) return Number(field.integerValue);
  return Object.values(field)[0];
}

export async function makeProfileEligible(uid: string): Promise<void> {
  const response = await fetch(`${firestoreRoot}/profiles/${uid}?updateMask.fieldPaths=created_at&updateMask.fieldPaths=updated_at`, {
    method: "PATCH", headers: { Authorization: "Bearer owner", "Content-Type": "application/json" },
    body: JSON.stringify({ fields: {
      created_at: { timestampValue: "2000-01-01T00:00:00Z" },
      updated_at: { timestampValue: "2000-01-02T00:00:00Z" },
    } }),
  });
  if (!response.ok) throw new Error(await response.text());
}

export async function signIn(page: Page | Frame, email: string, options: { navigate?: boolean; redirect?: boolean } = {}): Promise<string> {
  if (options.navigate !== false) await page.goto("/profile");
  const popupPromise = options.redirect ? Promise.resolve(page) : (page as Page).waitForEvent("popup");
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
  const popup = await popupPromise;
  await popup.waitForLoadState("domcontentloaded");
  if (options.redirect) await page.waitForURL("http://127.0.0.1:19099/**");
  const closed = options.redirect ? Promise.resolve() : (popup as Page).waitForEvent("close");
  if (await popup.getByText(email, { exact: true }).count()) {
    await popup.getByText(email, { exact: true }).click();
  } else {
    await popup.getByRole("button", { name: "Add new account", exact: true }).click();
    await popup.locator("#email-input").fill(email);
    await popup.getByRole("button", { name: /^Sign in with Google/i }).click();
  }
  await closed;
  await expect(page.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();
  const response = await fetch(`http://127.0.0.1:19099/identitytoolkit.googleapis.com/v1/projects/${projectId}/accounts:batchGet?maxResults=1000`,
    { headers: { Authorization: "Bearer owner" } });
  const data = await response.json() as { users: Array<{ email: string; localId: string }> };
  const user = data.users.find((user) => user.email === email);
  if (!user) throw new Error(`Emulator user ${email} missing`);
  return user.localId;
}
