import { test, expect, seedAuth, waitForHydrated } from "./fixtures";

async function openHelpMenu(page: import("@playwright/test").Page) {
  await page
    .getByTestId("app-bar-help")
    .getByRole("button", { name: /^Help$/i })
    .click({ force: true });
  await expect(page.getByTestId("help-menu-panel")).toBeVisible();
}

async function completeVisibleTour(page: import("@playwright/test").Page) {
  const footer = page.locator('[data-testid="spotlight-footer"]:visible');
  const next = footer.getByTestId("spotlight-tour-next");
  await expect(footer).toBeVisible({ timeout: 60_000 });
  for (let i = 0; i < 16; i++) {
    if ((await footer.count()) === 0) {
      break;
    }
    await next.click({ force: true });
    try {
      await expect(footer).toHaveCount(0, { timeout: 2_000 });
      break;
    } catch {
      /* more steps */
    }
  }
  await expect(footer).toHaveCount(0, { timeout: 30_000 });
}

test.describe("help-spotlight", () => {
  test("help-spotlight-once-anon", async ({ page }) => {
    await seedAuth(page, "anonymous", { help_tour: true });
    await page.goto("/coming-soon", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-tour-player")).toBeAttached();
    await expect(page.getByTestId("help-step-coming-soon")).toBeVisible({ timeout: 60_000 });
    await completeVisibleTour(page);

    await page.goto("/coming-soon", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-coming-soon")).toHaveCount(0);
    await expect(page.locator('[data-testid="spotlight-footer"]:visible')).toHaveCount(0);
  });

  test("help-spotlight-once-authed", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-apps-search")).toBeVisible({ timeout: 60_000 });
    await completeVisibleTour(page);

    await page.reload({ waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-apps-search")).toHaveCount(0);
  });

  test("help-spotlight-replay-current-route", async ({ page }) => {
    await seedAuth(page, "anonymous", { help_tour: true });
    await page.goto("/coming-soon", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-coming-soon")).toBeVisible({ timeout: 60_000 });
    await completeVisibleTour(page);
    await expect(page.getByTestId("help-step-coming-soon")).toHaveCount(0);

    await openHelpMenu(page);
    await page.getByTestId("help-menu-replay-tour").click({ force: true });
    await expect(page.getByTestId("help-step-coming-soon")).toBeVisible({ timeout: 60_000 });
  });

  test("help-spotlight-replay-does-not-affect-other-route", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/welcome", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-welcome-featured")).toBeVisible({ timeout: 60_000 });
    await completeVisibleTour(page);

    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-apps-search")).toBeVisible({ timeout: 60_000 });
    await completeVisibleTour(page);

    await openHelpMenu(page);
    await page.getByTestId("help-menu-replay-tour").click({ force: true });
    await expect(page.getByTestId("help-step-apps-search")).toBeVisible({ timeout: 60_000 });
    await completeVisibleTour(page);

    await page.goto("/welcome", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-welcome-featured")).toHaveCount(0);
  });

  test("help-spotlight-mobile-viewport", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await seedAuth(page, "anonymous", { help_tour: true });
    await page.goto("/coming-soon", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-coming-soon")).toBeVisible({ timeout: 60_000 });
    await expect(page.locator('[data-testid="spotlight-footer"]:visible')).toBeVisible();
    await completeVisibleTour(page);

    await openHelpMenu(page);
    await expect(page.getByTestId("help-menu-report-bug")).toBeVisible();
  });

  test("help-spotlight-unanchored-centered", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await seedAuth(page, "anonymous", { help_tour: true });
    await page.goto("/coming-soon", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);

    const step = page.getByTestId("help-step-coming-soon");
    await expect(step).toBeVisible({ timeout: 60_000 });
    await expect(page.locator('[data-testid="spotlight-header"]:visible')).toHaveText(
      "Coming Soon Intro",
    );

    const panel = page.locator(".orbital-popover-shell.orbital-spotlight").filter({
      has: step,
    });
    await expect(panel).toBeVisible();
    const box = await panel.boundingBox();
    expect(box).toBeTruthy();
    const vp = page.viewportSize();
    expect(vp).toBeTruthy();
    if (!box || !vp) {
      return;
    }
    const centerX = box.x + box.width / 2;
    const centerY = box.y + box.height / 2;
    // Viewport-centered (no `spotlight` id) — allow layout chrome slack.
    expect(Math.abs(centerX - vp.width / 2)).toBeLessThan(vp.width * 0.2);
    expect(Math.abs(centerY - vp.height / 2)).toBeLessThan(vp.height * 0.25);
  });
  test("help-spotlight-apps-application-cards", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-apps-search")).toBeVisible({ timeout: 60_000 });
    const footer = page.locator('[data-testid="spotlight-footer"]:visible');
    await footer.getByTestId("spotlight-tour-next").click({ force: true });
    await expect(page.getByTestId("help-step-apps-application-cards")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.locator("#apps-first-application-card")).toBeVisible();
  });

  test("help-spotlight-app-detail-more-information", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/apps/welcome", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-app-overview-more-information")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.locator("#app-overview-more-information")).toBeVisible();
  });

  test("help-spotlight-skips-signin-gate", async ({ page }) => {
    await seedAuth(page, "anonymous", { help_tour: true });
    await page.goto("/welcome", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached();
    await expect(page.getByText("Sign in required")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByTestId("help-step-welcome-featured")).toHaveCount(0);
    await expect(page.locator('[data-testid="spotlight-footer"]:visible')).toHaveCount(0);

    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached();
    await expect(page.getByTestId("help-step-apps-search")).toHaveCount(0);
    await expect(page.locator('[data-testid="spotlight-footer"]:visible')).toHaveCount(0);
  });

  test("help-spotlight-skips-permission-gate", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/gate/permission", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("permission-required-empty-state")).toBeAttached();
    await expect(page.getByText("Permission required")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByTestId("help-step-gate-permission")).toHaveCount(0);
    await expect(page.locator('[data-testid="spotlight-footer"]:visible')).toHaveCount(0);
  });

  test("help-spotlight-welcome-anchored-featured", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/welcome", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-welcome-featured")).toBeVisible({ timeout: 60_000 });
    await expect(page.locator("#welcome-featured-card")).toBeVisible();
  });
});

test.describe("help-report", () => {
  async function openReportDialog(
    page: import("@playwright/test").Page,
    menuTestId: string,
    dialogTestId: string,
  ) {
    await openHelpMenu(page);
    await page.getByTestId(menuTestId).click({ force: true });
    const dialog = page.getByTestId(dialogTestId);
    await expect(dialog).toBeVisible();
    return dialog;
  }

  test("help-report-bug-intro-and-form", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    if ((await page.getByTestId("help-step-apps-search").count()) > 0) {
      await completeVisibleTour(page);
    }

    const dialog = await openReportDialog(
      page,
      "help-menu-report-bug",
      "help-report-dialog-bug",
    );
    const openGithub = dialog.getByTestId("help-report-open-github").getByRole("link");
    await expect(openGithub).toBeVisible({ timeout: 30_000 });
    await expect(openGithub).toHaveAttribute("href", /\/issues\/new\?labels=bug/);
    const noAccount = dialog.getByTestId("help-report-no-account");
    await expect(noAccount).toBeEnabled();
    await noAccount.click();
    await expect(dialog.getByTestId("help-report-submit")).toBeVisible({
      timeout: 30_000,
    });
    await expect(dialog.getByPlaceholder("Short summary")).toBeVisible();
  });

  test("help-report-feature-intro-and-form", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    if ((await page.getByTestId("help-step-apps-search").count()) > 0) {
      await completeVisibleTour(page);
    }

    const dialog = await openReportDialog(
      page,
      "help-menu-request-feature",
      "help-report-dialog-feature",
    );
    const openGithub = dialog.getByTestId("help-report-open-github").getByRole("link");
    await expect(openGithub).toBeVisible({ timeout: 30_000 });
    await expect(openGithub).toHaveAttribute(
      "href",
      /\/issues\/new\?labels=enhancement/,
    );
    const noAccount = dialog.getByTestId("help-report-no-account");
    await expect(noAccount).toBeEnabled();
    await noAccount.click();
    await expect(dialog.getByTestId("help-report-submit")).toBeVisible({
      timeout: 30_000,
    });
    await expect(dialog.getByPlaceholder("Short summary")).toBeVisible();
  });

  test("help-report-security-private-copy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    if ((await page.getByTestId("help-step-apps-search").count()) > 0) {
      await completeVisibleTour(page);
    }

    const dialog = await openReportDialog(
      page,
      "help-menu-report-security",
      "help-report-dialog-security",
    );
    await expect(dialog.getByText(/Do not use public issues/i)).toBeVisible();
    const openGithub = dialog.getByTestId("help-report-open-github").getByRole("link");
    await expect(openGithub).toBeVisible({ timeout: 30_000 });
    await expect(openGithub).toHaveAttribute("href", /\/security\/advisories\/new/);
    await expect(openGithub).not.toHaveAttribute("href", /\/issues/);
    const noAccount = dialog.getByTestId("help-report-no-account");
    await expect(noAccount).toBeEnabled();
    await noAccount.click();
    await expect(dialog.getByTestId("help-report-submit")).toBeVisible({
      timeout: 30_000,
    });
    await expect(dialog.getByPlaceholder("Short summary")).toBeVisible();
  });

  test("help-report-missing-repository", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    // `/coming-soon` is host-only (no uf_app! repository) — sad path for TM-report-sad.
    await page.goto("/coming-soon", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    if ((await page.getByTestId("help-step-coming-soon").count()) > 0) {
      await completeVisibleTour(page);
    }

    const dialog = await openReportDialog(
      page,
      "help-menu-report-bug",
      "help-report-dialog-bug",
    );
    await expect(dialog.getByTestId("help-report-repo-missing")).toBeVisible({
      timeout: 30_000,
    });
    await expect(dialog.getByTestId("help-report-open-github")).toHaveCount(0);
    await expect(dialog.getByTestId("help-report-no-account")).toBeDisabled();
  });
});
