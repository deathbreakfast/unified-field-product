import { test, expect, seedAuth, waitForHydrated } from "./fixtures";

test.describe("pw-workspace-search", () => {
  test("pw-workspace-search-desktop-select-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto("/");
    await waitForHydrated(page);

    const search = page.getByTestId("app-bar-search");
    await expect(search).toBeVisible();
    await search.getByRole("combobox").fill("WorkspaceBeaconAlpha");
    const hit = page.getByTestId("workspace-search-hit").filter({
      hasText: "WorkspaceBeaconAlpha",
    });
    await expect(hit).toBeVisible({ timeout: 10_000 });
    await hit.click();
    await expect(page).toHaveURL(/\/workspace-search-hit/);
    await expect(page.getByTestId("workspace-search-destination")).toBeVisible();
  });

  test("pw-workspace-search-desktop-no-match-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto("/");
    await waitForHydrated(page);

    const search = page.getByTestId("app-bar-search");
    await search.getByRole("combobox").fill("zz-no-workspace-match");
    await expect(
      page.getByTestId("workspace-search-hit").filter({
        hasText: "WorkspaceBeaconAlpha",
      }),
    ).toHaveCount(0, { timeout: 10_000 });
    await expect(page).toHaveURL(/\/$/);
  });

  test("pw-workspace-search-mobile-select-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/");
    await waitForHydrated(page);

    await expect(page.getByTestId("app-bar-search")).toHaveCount(0);
    await page.getByTestId("app-bar-search-mobile-trigger").click();
    const dialog = page.getByTestId("workspace-search-dialog");
    await expect(dialog).toBeVisible();
    await dialog.getByRole("searchbox").fill("WorkspaceBeaconAlpha");
    const hit = dialog.getByTestId("workspace-search-hit").filter({
      hasText: "WorkspaceBeaconAlpha",
    });
    await expect(hit).toBeVisible({ timeout: 10_000 });
    await hit.click();
    await expect(page).toHaveURL(/\/workspace-search-hit/);
    await expect(page.getByTestId("workspace-search-destination")).toBeVisible();
  });

  test("pw-workspace-search-mobile-no-match-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/");
    await waitForHydrated(page);

    await page.getByTestId("app-bar-search-mobile-trigger").click();
    const dialog = page.getByTestId("workspace-search-dialog");
    await dialog.getByRole("searchbox").fill("zz-no-workspace-match");
    await expect(
      dialog.getByTestId("workspace-search-hit").filter({
        hasText: "WorkspaceBeaconAlpha",
      }),
    ).toHaveCount(0, { timeout: 10_000 });
    await expect(page).toHaveURL(/\/$/);
  });

  test("pw-workspace-search-unauth-sad", async ({ page }) => {
    await seedAuth(page, "anonymous");
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto("/");
    await waitForHydrated(page);

    const search = page.getByTestId("app-bar-search");
    const combobox = search.getByRole("combobox");
    await expect(combobox).toBeDisabled();
    await expect(combobox).toHaveAttribute("placeholder", "Sign in to search");
  });
});
