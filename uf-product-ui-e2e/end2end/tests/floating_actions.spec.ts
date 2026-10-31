import { test, expect, waitForHydrated } from "./fixtures";

/**
 * Shell floating actions menu (FAM). `/` mounts `ShellFloatingActions` and the
 * host registers one fixture item; `/utilities-override` is a shell without the
 * slot. Each check runs on desktop light, desktop dark and phone.
 */
const VARIANTS = [
  { name: "desktop-light", viewport: { width: 1280, height: 800 }, colorScheme: "light" },
  { name: "desktop-dark", viewport: { width: 1280, height: 800 }, colorScheme: "dark" },
  { name: "phone", viewport: { width: 390, height: 844 }, colorScheme: "light" },
] as const;

test.describe("pw-shell-fam", () => {
  for (const v of VARIANTS) {
    test.describe(v.name, () => {
      test.use({ viewport: v.viewport, colorScheme: v.colorScheme });

      test(`pw-shell-fam-item-${v.name}-happy`, async ({ page }) => {
        await page.goto("/");
        await expect(page.getByTestId("unified-field-shell-layout")).toBeVisible({
          timeout: 60_000,
        });
        await waitForHydrated(page);

        const fam = page.getByRole("group", { name: "Page actions" });
        await expect(fam).toBeVisible();
        const trigger = fam.getByTestId("floating-actions-menu-trigger");
        await expect(trigger).toBeVisible();

        const item = fam.getByTestId("fam-e2e-fixture");
        await expect(item).toBeHidden();
        await trigger.click();
        await expect(fam.getByRole("menu")).toHaveAttribute("aria-expanded", "true");
        await expect(item).toBeVisible();
        await expect(item).toHaveAccessibleName("E2E fixture action");

        await item.click();
        await expect(page.getByText("E2E fixture action ran")).toBeVisible();
        await expect(fam.getByRole("menu")).toHaveAttribute("aria-expanded", "false");
        await expect(item).toBeHidden();
      });

      test(`pw-shell-fam-absent-${v.name}-sad`, async ({ page }) => {
        await page.goto("/utilities-override");
        await expect(page.getByTestId("utilities-override-page")).toBeVisible({
          timeout: 60_000,
        });
        await waitForHydrated(page);

        await expect(page.getByTestId("unified-field-shell-layout")).toBeVisible();
        await expect(page.getByRole("group", { name: "Page actions" })).toHaveCount(0);
        await expect(page.getByTestId("floating-actions-menu-trigger")).toHaveCount(0);
        await expect(page.getByTestId("fam-e2e-fixture")).toHaveCount(0);
      });
    });
  }
});
