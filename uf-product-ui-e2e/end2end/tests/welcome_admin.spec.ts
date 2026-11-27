import type { Page } from "@playwright/test";
import { test, expect, seedAuth, waitForHydrated } from "./fixtures";

/** Name the e2e seed gives the verified user's profile. */
const ADMIN_NAME = "Ines Okoro";

async function openAdmin(page: Page, opts: { admin: boolean; viewer?: string }) {
  await seedAuth(page, "authenticated_verified", {
    welcome_admin: opts.admin,
    usage_viewer: opts.viewer,
  });
  await page.goto("/welcome/admin");
  await waitForHydrated(page);
  await expect(page.getByTestId("welcome-admin-page")).toBeVisible({ timeout: 60_000 });
}

async function waitForEditor(page: Page) {
  await expect(page.getByTestId("featured-admin-loading")).toHaveCount(0, { timeout: 60_000 });
  await expect(page.getByTestId("featured-admin-list")).toBeVisible({ timeout: 60_000 });
}

/** Pick `name` in the Add an app picker and add it to the draft. */
async function addApp(page: Page, name: string) {
  const combobox = page.getByTestId("featured-admin-add-picker").getByRole("combobox");
  await combobox.click();
  await combobox.fill(name);
  await page.getByRole("option", { name, exact: true }).click();
  await page.getByTestId("add-featured-app").click();
}

async function publishDraft(page: Page) {
  await page.getByTestId("featured-admin-publish").click();
  const dialog = page.getByTestId("featured-admin-publish-dialog");
  await expect(dialog).toBeVisible();
  await dialog.getByTestId("featured-admin-publish-confirm").click();
  await expect(dialog).toBeHidden({ timeout: 30_000 });
}

async function rowOrder(page: Page): Promise<string[]> {
  return page
    .locator("[data-testid^='featured-admin-row-']")
    .evaluateAll((els) => els.map((el) => el.getAttribute("data-testid") ?? ""));
}

test.describe("welcome-admin", () => {
  test.describe.configure({ timeout: 180_000 });

  test("welcome-admin-denied-no-controls-sad", async ({ page }) => {
    await openAdmin(page, { admin: false });
    const denied = page.getByTestId("welcome-admin-denied");
    await expect(denied).toBeVisible({ timeout: 60_000 });
    await expect(denied).toContainText("Only site admins can change featured apps");
    await expect(denied).toContainText("You need the Welcome admin permission.");
    await expect(page.getByTestId("featured-admin-back")).toBeVisible();
    await expect(page.getByTestId("featured-admin-ask-access")).toBeVisible();
    await expect(page.getByTestId("featured-admin-list")).toHaveCount(0);
    await expect(page.getByTestId("featured-admin-add-picker")).toHaveCount(0);
    await expect(page.getByTestId("add-featured-app")).toHaveCount(0);
    await expect(page.getByTestId("featured-admin-history-link")).toHaveCount(0);
    await expect(page.getByTestId("welcome-admin-page")).not.toContainText("permission_denied");

    await page.getByTestId("featured-admin-back").click();
    await expect(page).toHaveURL(/\/welcome$/);
  });

  test("welcome-admin-add-draft-publish-toast-happy", async ({ page }) => {
    await openAdmin(page, { admin: true, viewer: "e2e-welcome-admin-publish" });
    await waitForEditor(page);
    await expect(page.getByTestId("featured-admin-empty")).toContainText(
      "Welcome hides the Featured card until you add an app.",
    );
    await expect(page.getByTestId("featured-admin-last-published")).toContainText(
      "Not published yet",
    );

    // Editing the draft must not remount the page.
    await page.getByTestId("welcome-admin-page").evaluate((el) => {
      (el as HTMLElement & { __probe?: boolean }).__probe = true;
    });

    await addApp(page, "Apps");
    await expect(page.getByTestId("featured-admin-row-apps")).toBeVisible();
    await expect(page.getByTestId("featured-admin-row-apps")).toContainText("Added");
    await expect(page.getByTestId("featured-admin-preview-tile-apps")).toBeVisible();
    const bar = page.getByTestId("featured-admin-draft-bar");
    await expect(bar).toContainText("Draft");
    await expect(bar).toContainText("Added Apps");
    await expect(bar).toContainText("Members still see the old list.");

    const probe = await page
      .getByTestId("welcome-admin-page")
      .evaluate((el) => (el as HTMLElement & { __probe?: boolean }).__probe === true);
    expect(probe).toBe(true);

    await page.getByTestId("featured-admin-publish").click();
    const dialog = page.getByTestId("featured-admin-publish-dialog");
    await expect(dialog).toContainText("Publish featured apps?");
    await expect(dialog).toContainText("Every member sees Apps, in that order");
    await expect(dialog).toContainText("You can undo this from History.");
    await dialog.getByTestId("featured-admin-publish-confirm").click();

    await expect(page.getByText("Featured apps published.")).toBeVisible({ timeout: 30_000 });
    await expect(page.getByRole("button", { name: "Undo" })).toBeVisible();
    await expect(bar).toHaveCount(0);
    await expect(page.getByTestId("featured-admin-last-published")).toContainText(
      `Last published by ${ADMIN_NAME}`,
      { timeout: 30_000 },
    );
    await expect(
      page.getByTestId("featured-admin-last-published").locator("time"),
    ).toHaveAttribute("datetime", /\d{4}-\d{2}-\d{2}T/);

    await page.goto("/welcome");
    await waitForHydrated(page);
    await expect(
      page.getByTestId("featured-apps-card").getByTestId("app-link-apps"),
    ).toBeVisible({ timeout: 60_000 });
  });

  test("welcome-admin-move-remove-discard-happy", async ({ page }) => {
    await openAdmin(page, { admin: true, viewer: "e2e-welcome-admin-move" });
    await waitForEditor(page);
    await addApp(page, "Apps");
    await addApp(page, "Welcome");
    await publishDraft(page);
    await expect(page.getByTestId("featured-admin-draft-bar")).toHaveCount(0, { timeout: 30_000 });
    expect(await rowOrder(page)).toEqual([
      "featured-admin-row-apps",
      "featured-admin-row-welcome",
    ]);
    await expect(
      page.getByTestId("featured-admin-row-apps").getByTestId("featured-admin-move-up"),
    ).toBeDisabled();

    await page
      .getByTestId("featured-admin-row-apps")
      .getByTestId("featured-admin-move-down")
      .click();
    await expect.poll(() => rowOrder(page)).toEqual([
      "featured-admin-row-welcome",
      "featured-admin-row-apps",
    ]);
    await expect(page.getByTestId("featured-admin-draft-bar")).toContainText("Moved");

    await page.getByTestId("featured-admin-discard").click();
    await expect.poll(() => rowOrder(page)).toEqual([
      "featured-admin-row-apps",
      "featured-admin-row-welcome",
    ]);
    await expect(page.getByTestId("featured-admin-draft-bar")).toHaveCount(0);

    await page
      .getByTestId("featured-admin-row-welcome")
      .getByTestId("featured-admin-remove")
      .click();
    await expect(page.getByTestId("featured-admin-row-welcome")).toHaveCount(0);
    await expect(page.getByTestId("featured-admin-draft-bar")).toContainText("Removed Welcome");
    await page.getByTestId("featured-admin-discard").click();
    await expect(page.getByTestId("featured-admin-row-welcome")).toBeVisible();
  });

  test("welcome-admin-history-restore-happy", async ({ page }) => {
    await openAdmin(page, { admin: true, viewer: "e2e-welcome-admin-history" });
    await waitForEditor(page);
    await addApp(page, "Apps");
    await publishDraft(page);
    await expect(page.getByTestId("featured-admin-last-published")).toContainText(ADMIN_NAME, {
      timeout: 30_000,
    });

    await page.getByTestId("featured-admin-history-link").click();
    const drawer = page.getByTestId("featured-admin-history-drawer");
    const items = drawer.getByTestId("featured-admin-history-item");
    await expect(items).toHaveCount(1, { timeout: 30_000 });
    await expect(items.first()).toContainText(ADMIN_NAME);
    await expect(items.first()).toContainText("Added Apps");
    await expect(items.first().locator("time")).toHaveAttribute("datetime", /T/);

    await items.first().getByTestId("featured-admin-restore").click();
    await expect(page.getByText("Featured apps restored.")).toBeVisible({ timeout: 30_000 });
    await expect(items).toHaveCount(2, { timeout: 30_000 });
    await expect(items.first()).toContainText("Restored an earlier list");
    await page.keyboard.press("Escape");
    await expect(page.getByTestId("featured-admin-row-apps")).toHaveCount(0, { timeout: 30_000 });
    await expect(page.getByTestId("featured-admin-empty")).toBeVisible();
  });

  test("welcome-admin-toast-undo-happy", async ({ page }) => {
    await openAdmin(page, { admin: true, viewer: "e2e-welcome-admin-undo" });
    await waitForEditor(page);
    await addApp(page, "Apps");
    await publishDraft(page);
    await page.getByRole("button", { name: "Undo" }).click();
    await expect(page.getByText("Change undone.")).toBeVisible({ timeout: 30_000 });
    await expect(page.getByTestId("featured-admin-row-apps")).toHaveCount(0, { timeout: 30_000 });
    await expect(page.getByTestId("featured-admin-empty")).toBeVisible();
  });

  test("welcome-admin-conflict-two-sessions-sad", async ({ browser }) => {
    const ctxA = await browser.newContext();
    const ctxB = await browser.newContext();
    try {
      const a = await ctxA.newPage();
      const b = await ctxB.newPage();
      // Both drafts start from the same (empty) published list.
      await openAdmin(a, { admin: true, viewer: "e2e-welcome-admin-conflict-a" });
      await openAdmin(b, { admin: true, viewer: "e2e-welcome-admin-conflict-b" });
      await waitForEditor(a);
      await waitForEditor(b);

      await addApp(a, "Apps");
      await publishDraft(a);
      await expect(a.getByText("Featured apps published.")).toBeVisible({ timeout: 30_000 });

      await addApp(b, "Welcome");
      await publishDraft(b);
      const conflict = b.getByTestId("featured-admin-conflict");
      await expect(conflict).toContainText(
        `${ADMIN_NAME} published a change while you were editing. Reload to see it; your draft is kept.`,
        { timeout: 30_000 },
      );
      await expect(b.getByTestId("featured-admin-row-welcome")).toBeVisible();

      await b.getByTestId("featured-admin-reload").click();
      await expect(conflict).toHaveCount(0, { timeout: 30_000 });
      await expect(b.getByTestId("featured-admin-row-welcome")).toBeVisible();
      await expect(b.getByTestId("featured-admin-draft-bar")).toBeVisible();

      await publishDraft(b);
      await expect(b.getByText("Featured apps published.")).toBeVisible({ timeout: 30_000 });
    } finally {
      await ctxA.close();
      await ctxB.close();
    }
  });

  test("welcome-admin-phone-stacks-preview-happy", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await openAdmin(page, { admin: true, viewer: "e2e-welcome-admin-phone" });
    await waitForEditor(page);
    await addApp(page, "Apps");
    const list = await page.getByTestId("featured-admin-list").boundingBox();
    const preview = await page.getByTestId("featured-admin-preview").boundingBox();
    expect(list).toBeTruthy();
    expect(preview).toBeTruthy();
    if (list && preview) {
      expect(preview.y).toBeGreaterThan(list.y + list.height - 1);
    }
  });
});
