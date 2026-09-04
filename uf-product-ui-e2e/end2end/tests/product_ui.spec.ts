import { test, expect, seedAuth, waitForHydrated } from "./fixtures";

test.describe("pw-shell", () => {
  test("pw-shell-layout-default-closed-happy", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("unified-field-shell-layout")).toBeVisible({
      timeout: 60_000,
    });
    await waitForHydrated(page);
    await expect(page.getByTestId("shell-chrome-home")).toBeVisible();
    // Sidebar starts closed (Auto); left nav exists in DOM but drawer should not be forced open.
    await expect(page.getByTestId("shell-chrome-left-nav")).toBeAttached();
  });

  test("pw-shell-sidebar-toggle-happy", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("unified-field-shell-layout")).toBeVisible({
      timeout: 60_000,
    });
    await waitForHydrated(page);
    await page.getByRole("button", { name: /expand navigation/i }).click();
    await expect(page.getByTestId("shell-chrome-left-nav")).toBeVisible();
  });
});

test.describe("pw-app-bar", () => {
  test("pw-app-bar-default-utilities-happy", async ({ page }) => {
    await page.goto("/");
    await waitForHydrated(page);
    await expect(page.getByTestId("app-bar-trailing")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("app-bar-help")).toBeVisible();
    await expect(page.getByTestId("app-bar-apps")).toBeVisible();
    await expect(page.getByTestId("app-bar-apps").getByRole("button")).toBeEnabled();
    await expect(page.getByTestId("app-bar-appearance")).toBeVisible();
  });

  test("pw-app-bar-utilities-override-happy", async ({ page }) => {
    await page.goto("/utilities-override");
    await waitForHydrated(page);
    await expect(page.getByTestId("utilities-override-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("custom-utilities-marker")).toBeVisible();
    await expect(page.getByTestId("app-bar-help")).toHaveCount(0);
    await expect(page.getByTestId("app-bar-apps")).toHaveCount(0);
    await expect(page.getByTestId("app-bar-appearance")).toHaveCount(0);
  });

  test("pw-app-bar-slots-visible-happy", async ({ page }) => {
    await page.goto("/");
    await waitForHydrated(page);
    await expect(page.getByTestId("app-bar-search")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("app-bar-trailing")).toBeVisible();
    await expect(page.getByTestId("app-bar-user-menu")).toBeVisible();
  });

  test("pw-app-bar-mobile-visible-at-top-happy", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/scroll-chrome");
    await waitForHydrated(page);
    await expect(page.getByTestId("shell-chrome-scroll-fixture")).toBeVisible({
      timeout: 60_000,
    });

    const appBar = page.getByTestId("app-bar");
    const wrapper = page.getByTestId("hide-on-scroll");
    await expect(appBar).toBeVisible();
    await expect(wrapper).not.toHaveAttribute("data-app-bar-scroll-hidden", "true");
    const box = await appBar.boundingBox();
    expect(box).toBeTruthy();
    expect(box!.y).toBeGreaterThanOrEqual(-1);
  });

  test("pw-app-bar-mobile-hide-on-scroll-down-happy", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/scroll-chrome");
    await waitForHydrated(page);

    const scrollport = page.locator(".orbital-layout__page-scroll").first();
    const wrapper = page.getByTestId("hide-on-scroll");
    const appBar = page.getByTestId("app-bar");

    await scrollport.evaluate((el) => {
      el.scrollTop = 400;
      el.dispatchEvent(new Event("scroll", { bubbles: true }));
    });

    await expect
      .poll(async () => scrollport.evaluate((el) => el.scrollTop))
      .toBeGreaterThanOrEqual(400);

    await expect(wrapper).toHaveAttribute("data-app-bar-scroll-hidden", "true", {
      timeout: 10_000,
    });

    await expect
      .poll(async () => {
        const barBox = await appBar.boundingBox();
        const portBox = await scrollport.boundingBox();
        if (!barBox || !portBox) return false;
        return barBox.y + barBox.height <= portBox.y + 1;
      })
      .toBe(true);
  });

  test("pw-app-bar-mobile-show-on-scroll-up-happy", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/scroll-chrome");
    await waitForHydrated(page);

    const scrollport = page.locator(".orbital-layout__page-scroll").first();
    const wrapper = page.getByTestId("hide-on-scroll");
    const appBar = page.getByTestId("app-bar");

    await scrollport.evaluate((el) => {
      el.scrollTop = 400;
      el.dispatchEvent(new Event("scroll", { bubbles: true }));
    });
    await expect(wrapper).toHaveAttribute("data-app-bar-scroll-hidden", "true", {
      timeout: 10_000,
    });

    await scrollport.evaluate((el) => {
      el.scrollTop = 200;
      el.dispatchEvent(new Event("scroll", { bubbles: true }));
    });

    await expect(wrapper).not.toHaveAttribute("data-app-bar-scroll-hidden", "true", {
      timeout: 10_000,
    });
    await expect
      .poll(async () => {
        const barBox = await appBar.boundingBox();
        const portBox = await scrollport.boundingBox();
        if (!barBox || !portBox) return false;
        return Math.abs(barBox.y - portBox.y) <= 2;
      })
      .toBe(true);
  });

  test("pw-app-bar-mobile-hide-keeps-bar-mounted-happy", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/scroll-chrome");
    await waitForHydrated(page);

    const scrollport = page.locator(".orbital-layout__page-scroll").first();
    await scrollport.evaluate((el) => {
      el.scrollTop = 400;
      el.dispatchEvent(new Event("scroll", { bubbles: true }));
    });

    await expect(page.getByTestId("hide-on-scroll")).toHaveAttribute(
      "data-app-bar-scroll-hidden",
      "true",
      { timeout: 10_000 },
    );
    await expect(page.getByTestId("app-bar")).toHaveCount(1);
    await expect(page.getByTestId("app-bar-trailing-compact")).toBeAttached();
    await expect(page.getByTestId("app-bar-user-menu")).toBeAttached();
  });

  test("pw-app-bar-desktop-sticky-no-hide-happy", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/scroll-chrome");
    await waitForHydrated(page);

    const scrollport = page.locator(".orbital-layout__page-scroll").first();
    const wrapper = page.getByTestId("hide-on-scroll");
    const appBar = page.getByTestId("app-bar");

    await scrollport.evaluate((el) => {
      el.scrollTop = 400;
      el.dispatchEvent(new Event("scroll", { bubbles: true }));
    });
    await expect
      .poll(async () => scrollport.evaluate((el) => el.scrollTop))
      .toBeGreaterThanOrEqual(400);

    await expect(wrapper).not.toHaveAttribute("data-app-bar-scroll-hidden", "true");
    const box = await appBar.boundingBox();
    expect(box).toBeTruthy();
    expect(box!.y).toBeGreaterThanOrEqual(-1);
  });
});

test.describe("pw-search-picker", () => {
  test("pw-search-picker-select-happy", async ({ page }) => {
    await page.goto("/");
    await waitForHydrated(page);
    await expect(page.getByTestId("shell-chrome-search")).toBeVisible({
      timeout: 60_000,
    });
    const search = page.getByTestId("shell-chrome-search");
    const combobox = search.getByRole("combobox");
    await combobox.click();
    await combobox.fill("Alpha");
    await expect(page.getByRole("option", { name: /Beacon Alpha/i })).toBeVisible({
      timeout: 30_000,
    });
    await page.getByRole("option", { name: /Beacon Alpha/i }).click();
    await expect(page.getByTestId("search-selected")).toHaveText("Beacon Alpha");
  });

  test("pw-search-picker-no-match-sad", async ({ page }) => {
    await page.goto("/");
    await waitForHydrated(page);
    await expect(page.getByTestId("shell-chrome-search")).toBeVisible({
      timeout: 60_000,
    });
    const search = page.getByTestId("shell-chrome-search");
    const combobox = search.getByRole("combobox");
    await combobox.click();
    await combobox.fill("zz-no-match");
    await expect(page.getByRole("option")).toHaveCount(0);
  });
});

test.describe("pw-pages", () => {
  test("pw-coming-soon-page-happy", async ({ page }) => {
    await page.goto("/coming-soon");
    await waitForHydrated(page);
    await expect(page.getByTestId("unified-field-coming-soon-page")).toBeVisible({
      timeout: 60_000,
    });
  });

  test("pw-not-found-page-happy", async ({ page }) => {
    await page.goto("/404");
    await waitForHydrated(page);
    await expect(page.getByTestId("unified-field-not-found-page")).toBeVisible({
      timeout: 60_000,
    });
  });
});

test.describe("pw-auth-gates", () => {
  test("pw-auth-gate-unauthenticated-sad", async ({ page }) => {
    await seedAuth(page, "anonymous");
    await page.goto("/welcome");
    await waitForHydrated(page);
    // AccessGateDialog portals content; the testid wrapper stays an empty status node.
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached();
    await expect(page.getByRole("dialog")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByText("Sign in required")).toBeVisible();
  });

  test("pw-auth-gate-signin-click-happy", async ({ page }) => {
    await seedAuth(page, "anonymous");
    await page.goto("/welcome");
    await waitForHydrated(page);
    await expect(page.getByText("Sign in required")).toBeVisible({
      timeout: 60_000,
    });
    await page.getByRole("button", { name: "Sign In", exact: true }).click();
    await expect(page).toHaveURL(/\/welcome/);
    await expect(page.getByTestId("auth-dialog-root")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("harness-auth-dialog-signin")).toBeAttached();
  });

  test("pw-welcome-authenticated-happy", async ({ page }) => {
    // Unique viewer with no seeded events → validating empty usage cards.
    await seedAuth(page, "authenticated_verified", {
      usage_viewer: "e2e-welcome-empty",
    });
    await page.goto("/welcome");
    await waitForHydrated(page);
    await expect(page.getByTestId("auth-required-empty-state")).toHaveCount(0);
    await expect(page.getByTestId("e2e-auth-bootstrap")).toHaveAttribute(
      "data-auth",
      "authenticated_verified",
    );
    await expect(page.getByTestId("welcome-page")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByTestId("featured-apps-card")).toBeVisible();
    await expect(page.getByTestId("recent-apps-card")).toBeVisible();
    await expect(page.getByTestId("my-most-used-card")).toBeVisible();
    await expect(page.getByTestId("popular-apps-card")).toBeVisible();
    await expect(
      page.getByTestId("featured-apps-card").getByTestId("app-links-empty"),
    ).toContainText("No featured apps yet.");
    await expect(
      page.getByTestId("recent-apps-card").getByTestId("app-links-empty"),
    ).toContainText("Nothing yet. Visit an app to see it here.");
    await expect(
      page.getByTestId("my-most-used-card").getByTestId("app-links-empty"),
    ).toContainText("No usage yet. Open a few apps and check back.");
    // Popular is fleet-wide; do not assert empty here (other scenarios may have seeded).
    await expect(page.getByText("Useful links", { exact: true })).toHaveCount(0);
    await expect(page.getByText("Tips", { exact: true })).toHaveCount(0);
  });

  test("pw-welcome-usage-seeded-happy", async ({ page }) => {
    // Fixture/integ-style UI: seed injects Spectra rows (not emit→fetch E2E).
    const viewer = "e2e-welcome-seeded";
    const seed = await seedAuth(page, "authenticated_verified", {
      usage_viewer: viewer,
      page_views: [
        { app_id: "apps", age_secs: 40 },
        { app_id: "welcome", age_secs: 20 },
        { app_id: "apps", age_secs: 10 },
        // Other viewer — must not appear in Recent / My most used.
        { app_id: "help", viewer_key: "other-user", age_secs: 5 },
      ],
    });
    expect(seed.recent_preview ?? []).toEqual(
      expect.arrayContaining(["apps", "welcome"]),
    );
    expect(seed.recent_preview ?? []).not.toContain("help");

    await page.goto("/welcome");
    await waitForHydrated(page);
    await expect(page.getByTestId("welcome-page")).toBeVisible({ timeout: 60_000 });

    const recent = page.getByTestId("recent-apps-card");
    await expect(recent.getByTestId("app-links-list")).toBeVisible({ timeout: 60_000 });
    await expect(recent.getByTestId("app-link-welcome")).toBeVisible();
    await expect(recent.getByTestId("app-link-apps")).toBeVisible();
    await expect(recent.getByTestId("app-link-help")).toHaveCount(0);

    const mine = page.getByTestId("my-most-used-card");
    await expect(mine.getByTestId("app-link-apps")).toBeVisible();
    await expect(mine.getByTestId("app-link-help")).toHaveCount(0);

    const popular = page.getByTestId("popular-apps-card");
    await expect(popular.getByTestId("app-link-apps")).toBeVisible();
    // Fleet popular may include other viewers' apps.
    await expect(popular.getByTestId("app-link-help")).toBeVisible();
  });

  test("pw-welcome-usage-emit-fetch-happy", async ({ page }) => {
    const viewer = "e2e-welcome-emit-fetch";
    await seedAuth(page, "authenticated_verified", {
      usage_viewer: viewer,
    });
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("apps-index-page")).toBeVisible({
      timeout: 60_000,
    });
    // PageViewTracker records /apps after hydrate; then welcome reads Spectra.
    await page.goto("/welcome", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("welcome-page")).toBeVisible({ timeout: 60_000 });

    const recent = page.getByTestId("recent-apps-card");
    await expect(recent.getByTestId("app-link-apps")).toBeVisible({
      timeout: 60_000,
    });
    const mine = page.getByTestId("my-most-used-card");
    await expect(mine.getByTestId("app-link-apps")).toBeVisible({
      timeout: 60_000,
    });
  });

  test("pw-welcome-admin-unauthenticated-sad", async ({ page }) => {
    await seedAuth(page, "anonymous");
    await page.goto("/welcome/admin");
    await waitForHydrated(page);
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached();
    await expect(page.getByRole("dialog")).toBeVisible({ timeout: 60_000 });
  });

  test("pw-welcome-admin-denied-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.goto("/welcome/admin");
    await waitForHydrated(page);
    await expect(page.getByTestId("welcome-admin-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("welcome-admin-denied")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("welcome-admin-denied")).toContainText(
      "WelcomeAdmin permission is required",
    );
  });

  test("pw-welcome-admin-featured-crud-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      usage_viewer: "e2e-welcome-admin-crud",
      welcome_admin: true,
    });
    await page.goto("/welcome/admin");
    await waitForHydrated(page);
    await expect(page.getByTestId("welcome-admin-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("welcome-admin-denied")).toHaveCount(0);

    await page.locator('select').selectOption("apps");
    await page.getByTestId("add-featured-app").getByRole("button").click();
    await expect(page.getByTestId("featured-admin-row-apps")).toBeVisible({
      timeout: 60_000,
    });

    await page.locator('select').selectOption("welcome");
    await page.getByTestId("add-featured-app").getByRole("button").click();
    await expect(page.getByTestId("featured-admin-row-welcome")).toBeVisible({
      timeout: 60_000,
    });

    await page.goto("/welcome");
    await waitForHydrated(page);
    const featured = page.getByTestId("featured-apps-card");
    await expect(featured.getByTestId("app-link-apps")).toBeVisible({
      timeout: 60_000,
    });
    await expect(featured.getByTestId("app-link-welcome")).toBeVisible();

    await page.goto("/welcome/admin");
    await waitForHydrated(page);
    const appsRow = page.getByTestId("featured-admin-row-apps");
    await appsRow.getByTestId("featured-admin-move-down").getByRole("button").click();
    // After move down, welcome should be first in the admin list DOM order.
    const rows = page.locator("[data-testid^='featured-admin-row-']");
    await expect(rows.nth(0)).toHaveAttribute(
      "data-testid",
      "featured-admin-row-welcome",
      { timeout: 60_000 },
    );
    await expect(rows.nth(1)).toHaveAttribute(
      "data-testid",
      "featured-admin-row-apps",
    );

    await page.goto("/welcome");
    await waitForHydrated(page);
    const featuredAfter = page.getByTestId("featured-apps-card");
    const featuredLinks = featuredAfter.locator("[data-testid^='app-link-']");
    await expect(featuredLinks.nth(0)).toHaveAttribute(
      "data-testid",
      "app-link-welcome",
      { timeout: 60_000 },
    );
    await expect(featuredLinks.nth(1)).toHaveAttribute(
      "data-testid",
      "app-link-apps",
    );

    await page.goto("/welcome/admin");
    await waitForHydrated(page);
    await page
      .getByTestId("featured-admin-row-apps")
      .getByTestId("featured-admin-remove")
      .getByRole("button")
      .click();
    await page
      .getByTestId("featured-admin-row-welcome")
      .getByTestId("featured-admin-remove")
      .getByRole("button")
      .click();
    await expect(page.getByTestId("featured-admin-row-apps")).toHaveCount(0, {
      timeout: 60_000,
    });
    await expect(page.getByTestId("featured-admin-row-welcome")).toHaveCount(0);

    await page.goto("/welcome");
    await waitForHydrated(page);
    await expect(
      page.getByTestId("featured-apps-card").getByTestId("app-links-empty"),
    ).toContainText("No featured apps yet.");
  });

  test("pw-auth-gate-unverified-email-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_unverified");
    await page.goto("/gate/email");
    await waitForHydrated(page);
    await expect(
      page.getByTestId("email-verification-required-empty-state"),
    ).toBeAttached();
    await expect(page.getByRole("dialog")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByText("Email verification required")).toBeVisible();
  });

  test("pw-auth-gate-permission-denied-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.goto("/gate/permission");
    await waitForHydrated(page);
    await expect(page.getByTestId("permission-required-empty-state")).toBeAttached();
    await expect(page.getByRole("dialog")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByText("Permission required")).toBeVisible();
  });

  test("pw-auth-gate-permission-allow-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { permission_allow: true });
    await page.goto("/gate/permission-allow");
    await waitForHydrated(page);
    await expect(page.getByTestId("gate-permission-allow-content")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("permission-required-empty-state")).toHaveCount(0);
  });

  test("pw-auth-gate-permission-request-redirect-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.goto("/gate/permission");
    await waitForHydrated(page);
    await expect(page.getByTestId("permission-required-empty-state")).toBeAttached();
    await page.getByRole("button", { name: "Request Permission" }).click();
    await expect(page).toHaveURL(/\/permission\/permissions\/?$/, { timeout: 60_000 });
  });
});

test.describe("pw-apps", () => {
  test("pw-apps-index-list-filter-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("apps-index-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("app-card-apps")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("app-card-welcome")).toBeVisible();

    const search = page.getByPlaceholder("Search apps");
    await search.fill("Welcome");
    await expect(page.getByTestId("app-card-welcome")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("app-card-apps")).toHaveCount(0);
  });

  test("pw-apps-index-filter-empty-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("apps-index-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("app-card-apps")).toBeVisible({
      timeout: 60_000,
    });

    await page.getByPlaceholder("Search apps").fill("zz-no-such-product-app");
    await expect(page.getByTestId("app-card-apps")).toHaveCount(0, {
      timeout: 30_000,
    });
    await expect(page.getByTestId("app-card-welcome")).toHaveCount(0);
    await expect(page.getByText("No apps registered")).toBeVisible();
  });

  test("pw-apps-detail-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    // Prefer index → Open over hard-nav to `/apps/apps`: full SSR of the detail
    // page can panic the e2e host (orbital StyleRegistry dispose mid-request).
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("app-card-apps")).toBeVisible({
      timeout: 60_000,
    });
    await page.getByTestId("app-card-apps").getByRole("link", { name: /open/i }).click();
    await expect(page.getByTestId("app-detail-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByText("App not found.")).toHaveCount(0);
  });

  test("pw-apps-detail-overview-links-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("app-card-apps")).toBeVisible({
      timeout: 60_000,
    });
    await page.getByTestId("app-card-apps").getByRole("link", { name: /open/i }).click();
    await expect(page.getByTestId("app-detail-page")).toBeVisible({
      timeout: 60_000,
    });

    const github = page.getByTestId("app-overview-github");
    await expect(github).toBeVisible({ timeout: 30_000 });
    await expect(github).toHaveAttribute(
      "href",
      "https://github.com/unified-field-dev/unified-field-product",
    );
    const [githubTab] = await Promise.all([
      page.waitForEvent("popup"),
      github.click(),
    ]);
    await expect(githubTab).toHaveURL(
      "https://github.com/unified-field-dev/unified-field-product",
      { timeout: 60_000 },
    );
    await githubTab.close();

    const docs = page.getByTestId("app-overview-docs");
    await expect(docs).toBeVisible();
    await expect(docs).toHaveAttribute("href", "https://docs.rs/uf-apps");
    const [docsTab] = await Promise.all([
      page.waitForEvent("popup"),
      docs.click(),
    ]);
    await expect(docsTab).toHaveURL(/https:\/\/docs\.rs\/uf-apps\/?/, {
      timeout: 60_000,
    });
    await docsTab.close();
  });

  test("pw-apps-detail-overview-links-absent-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.goto("/apps", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("app-card-welcome")).toBeVisible({
      timeout: 60_000,
    });
    await page
      .getByTestId("app-card-welcome")
      .getByRole("link", { name: /open/i })
      .click();
    await expect(page.getByTestId("app-detail-page")).toBeVisible({
      timeout: 60_000,
    });
    // Welcome registers a repository for Help filing; docs.rs still absent (no crate_name).
    await expect(page.getByTestId("app-overview-github")).toBeVisible();
    await expect(page.getByTestId("app-overview-docs")).toHaveCount(0);
  });

  test("pw-apps-detail-unknown-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.goto("/apps/zz-no-such-app", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("app-detail-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByText("App not found.")).toBeVisible({
      timeout: 30_000,
    });
  });
});

test.describe("pw-apps-launcher", () => {
  test("pw-apps-launcher-open-empty-happy", async ({ page }) => {
    await page.goto("/");
    await waitForHydrated(page);
    await page.getByTestId("app-bar-apps").getByRole("button").click();
    await expect(page.getByTestId("apps-launcher-dialog")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("apps-launcher-empty-prompt")).toBeVisible();
    await expect(page.locator("[data-testid^='apps-launcher-result-']")).toHaveCount(0);
  });

  test("pw-apps-launcher-typeahead-filter-happy", async ({ page }) => {
    await page.goto("/");
    await waitForHydrated(page);
    await page.getByTestId("app-bar-apps").getByRole("button").click();
    await expect(page.getByTestId("apps-launcher-dialog")).toBeVisible({
      timeout: 30_000,
    });
    const search = page
      .getByTestId("apps-launcher-search")
      .getByPlaceholder("Search apps");
    await search.fill("Welcome");
    await expect(page.getByTestId("apps-launcher-result-welcome")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("apps-launcher-result-apps")).toHaveCount(0);
  });

  test("pw-apps-launcher-filter-empty-sad", async ({ page }) => {
    await page.goto("/");
    await waitForHydrated(page);
    await page.getByTestId("app-bar-apps").getByRole("button").click();
    await expect(page.getByTestId("apps-launcher-dialog")).toBeVisible({
      timeout: 30_000,
    });
    await page
      .getByTestId("apps-launcher-search")
      .getByPlaceholder("Search apps")
      .fill("zz-no-such-product-app");
    await expect(page.getByTestId("apps-launcher-empty-no-match")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.locator("[data-testid^='apps-launcher-result-']")).toHaveCount(0);
  });

  test("pw-apps-launcher-select-navigate-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified");
    await page.goto("/");
    await waitForHydrated(page);
    await page.getByTestId("app-bar-apps").getByRole("button").click();
    await expect(page.getByTestId("apps-launcher-dialog")).toBeVisible({
      timeout: 30_000,
    });
    await page
      .getByTestId("apps-launcher-search")
      .getByPlaceholder("Search apps")
      .fill("Welcome");
    await expect(page.getByTestId("apps-launcher-result-welcome")).toBeVisible({
      timeout: 30_000,
    });
    await page.getByTestId("apps-launcher-result-welcome").click();
    await expect(page).toHaveURL(/\/welcome\/?$/, { timeout: 60_000 });
    await expect(page.getByTestId("apps-launcher-dialog")).toBeHidden();
  });

  test("pw-apps-launcher-dismiss-esc-happy", async ({ page }) => {
    await page.goto("/");
    await waitForHydrated(page);
    await page.getByTestId("app-bar-apps").getByRole("button").click();
    await expect(page.getByTestId("apps-launcher-dialog")).toBeVisible({
      timeout: 30_000,
    });
    await page.keyboard.press("Escape");
    // Dialog keeps the surface in the DOM when closed; assert not visible.
    await expect(page.getByTestId("apps-launcher-dialog")).toBeHidden({
      timeout: 30_000,
    });
    await expect(page.getByTestId("apps-launcher-empty-prompt")).toBeHidden();
    await expect(page).toHaveURL(/\/$/);
  });

  test("pw-apps-launcher-mobile-open-empty-happy", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/");
    await waitForHydrated(page);
    await page.getByTestId("app-bar-apps").getByRole("button").click();
    await expect(page.getByTestId("apps-launcher-dialog")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("apps-launcher-empty-prompt")).toBeVisible();
    await expect(page.locator("[data-testid^='apps-launcher-result-']")).toHaveCount(0);
  });

  test("pw-apps-launcher-mobile-select-navigate-happy", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await seedAuth(page, "authenticated_verified");
    await page.goto("/");
    await waitForHydrated(page);
    await page.getByTestId("app-bar-apps").getByRole("button").click();
    await expect(page.getByTestId("apps-launcher-dialog")).toBeVisible({
      timeout: 30_000,
    });
    await page
      .getByTestId("apps-launcher-search")
      .getByPlaceholder("Search apps")
      .fill("Welcome");
    await expect(page.getByTestId("apps-launcher-result-welcome")).toBeVisible({
      timeout: 30_000,
    });
    await page.getByTestId("apps-launcher-result-welcome").click();
    await expect(page).toHaveURL(/\/welcome\/?$/, { timeout: 60_000 });
    await expect(page.getByTestId("apps-launcher-dialog")).toBeHidden();
  });
});

test.describe("auth-and-notifications-routes", () => {
  test("pw-auth-index-redirects-to-signin", async ({ page }) => {
    await page.goto("/auth", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page).toHaveURL(/\/auth\/signin\/?/, { timeout: 60_000 });
    await expect(page.getByText("Coming Soon", { exact: true })).toHaveCount(0);
  });
});
