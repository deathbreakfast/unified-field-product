import type { Page } from "@playwright/test";
import { test, expect, waitForHydrated } from "./fixtures";

// Orbital renders both layouts with role="dialog". The drawer also carries
// its position class; the Dialog surface does not.
const DRAWER = ".orbital-overlay-drawer--position-bottom";
const DIALOG = '[role="dialog"]:not(.orbital-overlay-drawer)';

const WIDTHS = [
  { name: "wide", width: 1280, height: 800, drawer: false },
  { name: "narrow", width: 390, height: 844, drawer: true },
] as const;

async function openFixture(page: Page, width: number, height: number) {
  await page.setViewportSize({ width, height });
  await page.goto("/adaptive-sheet");
  await waitForHydrated(page);
  await expect(page.getByTestId("adaptive-sheet-fixture")).toBeVisible({
    timeout: 60_000,
  });
}

async function openSheet(page: Page) {
  await page.getByTestId("demo-sheet-trigger").click();
  await expect(page.getByTestId("demo-sheet")).toBeVisible();
}

async function confirms(page: Page): Promise<number> {
  return Number(await page.getByTestId("demo-sheet-confirms").innerText());
}

test.describe("pw-adaptive-sheet", () => {
  for (const w of WIDTHS) {
    test.describe(w.name, () => {
      test(`pw-adaptive-sheet-layout-${w.name}-happy`, async ({ page }) => {
        await openFixture(page, w.width, w.height);
        await expect(page.getByTestId("demo-sheet")).toHaveCount(0);
        await openSheet(page);
        await expect(page.getByTestId("demo-sheet")).toHaveCount(1);
        if (w.drawer) {
          await expect(page.locator(DRAWER)).toBeVisible();
          await expect(page.locator(DIALOG)).toHaveCount(0);
          const box = await page.locator(DRAWER).boundingBox();
          expect(box, "drawer bounding box").toBeTruthy();
          expect(box!.y + box!.height).toBeGreaterThanOrEqual(w.height - 2);
        } else {
          await expect(page.locator(DIALOG)).toBeVisible();
          await expect(page.locator(DRAWER)).toHaveCount(0);
        }
      });

      test(`pw-adaptive-sheet-confirm-then-cancel-${w.name}-happy`, async ({ page }) => {
        await openFixture(page, w.width, w.height);
        await openSheet(page);
        const before = await confirms(page);
        await page.getByTestId("demo-sheet-confirm").click();
        await expect.poll(() => confirms(page)).toBe(before + 1);
        await expect(page.getByTestId("demo-sheet-open-state")).toHaveText("open");

        // Cancel closes the sheet and leaves the confirm counter alone.
        await page.getByTestId("demo-sheet-cancel").click();
        await expect(page.getByTestId("demo-sheet")).not.toBeVisible();
        await expect(page.getByTestId("demo-sheet-open-state")).toHaveText("closed");
        expect(await confirms(page)).toBe(before + 1);

        // Reopen: Cancel again, still no confirm.
        await openSheet(page);
        await page.getByTestId("demo-sheet-cancel").click();
        await expect(page.getByTestId("demo-sheet-open-state")).toHaveText("closed");
        expect(await confirms(page)).toBe(before + 1);
      });

      test(`pw-adaptive-sheet-escape-${w.name}-happy`, async ({ page }) => {
        await openFixture(page, w.width, w.height);
        await openSheet(page);
        await page.keyboard.press("Escape");
        await expect(page.getByTestId("demo-sheet")).not.toBeVisible();
        await expect(page.getByTestId("demo-sheet-open-state")).toHaveText("closed");
      });

      test(`pw-adaptive-sheet-info-${w.name}-happy`, async ({ page }) => {
        await openFixture(page, w.width, w.height);
        await page.getByTestId("demo-sheet-info-trigger").click();
        await expect(page.getByTestId("demo-sheet-info")).toBeVisible();
        await page.getByTestId("demo-sheet-info-close").click();
        await expect(page.getByTestId("demo-sheet-info")).not.toBeVisible();
      });
    });
  }

  test("pw-adaptive-sheet-breakpoint-swap-keeps-state-happy", async ({ page }) => {
    await openFixture(page, 1280, 800);
    await openSheet(page);
    await expect(page.locator(DIALOG)).toBeVisible();
    await page.getByTestId("demo-sheet-input").locator("input").fill("keep-me");

    await page.setViewportSize({ width: 390, height: 844 });
    await expect(page.locator(DRAWER)).toBeVisible();
    await expect(page.locator(DIALOG)).toHaveCount(0);
    await expect(page.getByTestId("demo-sheet")).toHaveCount(1);
    await expect(page.getByTestId("demo-sheet-open-state")).toHaveText("open");
    await expect(page.getByTestId("demo-sheet-input").locator("input")).toHaveValue(
      "keep-me",
    );

    await page.setViewportSize({ width: 1280, height: 800 });
    await expect(page.locator(DIALOG)).toBeVisible();
    await expect(page.locator(DRAWER)).toHaveCount(0);
    await expect(page.getByTestId("demo-sheet-input").locator("input")).toHaveValue(
      "keep-me",
    );
  });

  test("pw-adaptive-sheet-size-small-shorter-than-full-happy", async ({ page }) => {
    await openFixture(page, 390, 844);

    // A closed drawer stays mounted, so pick each one by the sheet it holds.
    const drawerOf = (id: string) =>
      page.locator(DRAWER).filter({ has: page.getByTestId(id) });

    await openSheet(page);
    const fullBox = await drawerOf("demo-sheet").boundingBox();
    expect(fullBox, "full drawer box").toBeTruthy();
    await page.getByTestId("demo-sheet-cancel").click();
    await expect(page.getByTestId("demo-sheet")).not.toBeVisible();

    await page.getByTestId("demo-sheet-small-trigger").click();
    await expect(page.getByTestId("demo-sheet-small")).toBeVisible();
    const smallBox = await drawerOf("demo-sheet-small").boundingBox();
    expect(smallBox, "small drawer box").toBeTruthy();

    expect(smallBox!.height).toBeLessThan(fullBox!.height * 0.75);
  });
});
