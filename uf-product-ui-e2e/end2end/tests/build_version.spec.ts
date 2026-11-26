import { test, expect, waitForHydrated } from "./fixtures";
import type { Page } from "@playwright/test";

/**
 * New-version prompt. The host serves an unstamped `pkg`, so each test that
 * wants a prompt sets the e2e build override first: the next page load then
 * thinks it runs `pkg-e2e-old`, while `/_uf/build.json` reports `pkg`.
 */
const OLD_BUILD = "pkg-e2e-old";

const BAR_TITLE = "New version available";
const BAR_BODY =
  "The site was updated while you had it open. You can keep working, but some pages may not load or save correctly until you reload.";
const DIALOG_TITLE = "Reload to keep going";
const DIALOG_BODY =
  "The site was updated while you had this page open, and this version can't load your data any more. Reload to switch to the new version.";
const DIALOG_CAPTION =
  "Anything you haven't saved on this page will be lost. You'll come back to this same page.";
const UPDATED_TOAST = "Updated to the latest version.";

async function setOverride(page: Page, running: string | null) {
  const res = await page.request.post("/api/test/build-override", { data: { running } });
  expect(res.status()).toBe(204);
}

async function openFixture(page: Page) {
  await page.goto("/build-version-fixture");
  await expect(page.getByTestId("build-version-fixture")).toBeVisible({ timeout: 60_000 });
  await waitForHydrated(page);
}

test.describe("build-version", () => {

  test.afterEach(async ({ page }) => {
    await setOverride(page, null);
  });

  test("build-version-bar-not-now-returns-on-route-change", async ({ page }) => {
    await setOverride(page, OLD_BUILD);
    await openFixture(page);

    const bar = page.getByTestId("uf-build-bar");
    await expect(bar).toBeVisible();
    await expect(bar).toContainText(BAR_TITLE);
    await expect(bar).toContainText(BAR_BODY);
    await expect(bar.getByTestId("uf-build-bar-reload")).toHaveText("Reload now");
    await expect(bar.getByTestId("uf-build-bar-dismiss")).toHaveText("Not now");
    await expect(page.getByTestId("uf-build-dialog")).toBeHidden();

    await bar.getByTestId("uf-build-bar-dismiss").getByRole("button").click();
    await expect(bar).toHaveCount(0);

    await page.getByTestId("build-version-fixture-nav").getByRole("link").click();
    await expect(page.getByTestId("workspace-search-destination")).toBeVisible();
    await expect(page.getByTestId("uf-build-bar")).toBeVisible();
    await expect(page.getByTestId("uf-build-bar")).toContainText(BAR_TITLE);
  });

  test("build-version-reload-shows-updated-toast", async ({ page }) => {
    await setOverride(page, OLD_BUILD);
    await openFixture(page);
    const bar = page.getByTestId("uf-build-bar");
    await expect(bar).toBeVisible();

    // The reloaded page runs the build the server names, so it matches.
    await setOverride(page, null);
    await Promise.all([
      page.waitForEvent("load"),
      bar.getByTestId("uf-build-bar-reload").getByRole("button").click(),
    ]);
    await expect(page.getByTestId("build-version-fixture")).toBeVisible({ timeout: 60_000 });
    await waitForHydrated(page);

    await expect(page.getByText(UPDATED_TOAST)).toBeVisible();
    await expect(page.getByTestId("uf-build-bar")).toHaveCount(0);
    expect(await page.evaluate(() => sessionStorage.getItem("uf:updated-from"))).toBeNull();
    // Gone after five seconds.
    await expect(page.getByText(UPDATED_TOAST)).toHaveCount(0, { timeout: 10_000 });
  });

  test("build-version-server-fn-failure-shows-dialog", async ({ page }) => {
    await setOverride(page, OLD_BUILD);
    await openFixture(page);

    await page.getByTestId("build-version-fixture-load").click();
    await expect(page.getByTestId("build-version-fixture-failed")).toBeVisible();

    const dialog = page.getByTestId("uf-build-dialog");
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText(DIALOG_TITLE);
    await expect(dialog).toContainText(DIALOG_BODY);
    await expect(dialog).toContainText(DIALOG_CAPTION);
    await expect(dialog.getByTestId("uf-build-dialog-dismiss")).toHaveText("Not now");
    await expect(dialog.getByTestId("uf-build-dialog-reload")).toHaveText("Reload now");
    await expect(page.getByTestId("uf-build-bar")).toHaveCount(0);

    await dialog.getByTestId("uf-build-dialog-dismiss").getByRole("button").click();
    await expect(page.getByTestId("uf-build-dialog")).toBeHidden();
    await expect(page.getByTestId("uf-build-bar")).toHaveCount(0);
  });

  test("build-version-no-override-stays-quiet", async ({ page }) => {
    const buildRequests: string[] = [];
    page.on("request", (req) => {
      const path = new URL(req.url()).pathname;
      if (path === "/_uf/build.json" || path === "/uf-sw.js") buildRequests.push(path);
    });
    await openFixture(page);

    await page.getByTestId("build-version-fixture-load").click();
    await expect(page.getByTestId("build-version-fixture-failed")).toBeVisible();
    await page.waitForTimeout(1_500);

    await expect(page.getByTestId("uf-build-bar")).toHaveCount(0);
    await expect(page.getByTestId("uf-build-dialog")).toBeHidden();
    const registrations = await page.evaluate(async () =>
      "serviceWorker" in navigator ? (await navigator.serviceWorker.getRegistrations()).length : 0,
    );
    expect(registrations).toBe(0);
    expect(buildRequests).toEqual([]);
  });

  test.describe("phone", () => {
    test.use({ viewport: { width: 390, height: 844 } });

    test("build-version-bar-phone-wraps", async ({ page }) => {
      await setOverride(page, OLD_BUILD);
      await openFixture(page);

      const bar = page.getByTestId("uf-build-bar");
      await expect(bar).toBeVisible();
      const body = await bar.getByText(BAR_BODY).boundingBox();
      const reload = await bar.getByTestId("uf-build-bar-reload").boundingBox();
      const dismiss = await bar.getByTestId("uf-build-bar-dismiss").boundingBox();
      const barBox = await bar.boundingBox();
      expect(body && reload && dismiss && barBox).toBeTruthy();
      // The text wraps onto several lines and the buttons sit under it.
      expect(body!.height).toBeGreaterThan(30);
      expect(reload!.y).toBeGreaterThanOrEqual(body!.y + body!.height - 1);
      expect(dismiss!.y).toBeGreaterThanOrEqual(body!.y + body!.height - 1);
      expect(barBox!.x + barBox!.width).toBeLessThanOrEqual(390);
    });
  });
});
