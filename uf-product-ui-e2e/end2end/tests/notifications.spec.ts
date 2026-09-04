import {
  test,
  expect,
  seedAuth,
  waitForHydrated,
  readBellBadgeCount,
  openBellDropdown,
  measureBellDropdownWidth,
} from "./fixtures";

test.describe("pw-notifications-gate", () => {
  test("pw-notifications-unauth-gated-sad", async ({ page }) => {
    await seedAuth(page, "anonymous");
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached();
    await expect(page.getByRole("dialog")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByText("Sign in required")).toBeVisible();
    await expect(page.getByTestId("notifications-inbox-page")).toHaveCount(0);
  });
});

test.describe("pw-notifications-bell", () => {
  test("pw-notifications-bell-dropdown-empty-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { notifications: [] });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("notification-bell")).toBeVisible({
      timeout: 60_000,
    });
    await openBellDropdown(page);
    await expect(page.getByText("No unread notifications.")).toBeVisible({
      timeout: 30_000,
    });
  });

  test("pw-notifications-bell-dropdown-width-happy", async ({ page }) => {
    const longTitle =
      "BellWidthProbeTitleThatWouldStretchTheDropdownWithoutAStableMinMaxBand";
    const longMessage =
      "BellWidthProbeMessageThatWouldStretchTheDropdownWithoutAStableMinMaxBand ".repeat(
        6,
      );

    await seedAuth(page, "authenticated_verified", { notifications: [] });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await openBellDropdown(page);
    const emptyWidth = await measureBellDropdownWidth(page);
    expect(emptyWidth).toBeGreaterThanOrEqual(360);
    expect(emptyWidth).toBeLessThanOrEqual(400);

    await page.keyboard.press("Escape");
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: longTitle, message: longMessage, url: "/" },
      ],
    });
    await page.reload({ waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await openBellDropdown(page);
    const populatedWidth = await measureBellDropdownWidth(page);
    expect(populatedWidth).toBeGreaterThanOrEqual(360);
    expect(populatedWidth).toBeLessThanOrEqual(400);
    expect(Math.abs(emptyWidth - populatedWidth)).toBeLessThanOrEqual(1);
  });

  test("pw-notifications-bell-dropdown-items-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "BellRowOne", message: "in dropdown", url: "/" },
        { title: "BellRowTwo", message: "also in dropdown", url: "/" },
      ],
    });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    expect(await readBellBadgeCount(page)).toBeGreaterThanOrEqual(2);
    await openBellDropdown(page);
    await expect(page.getByText("BellRowOne")).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText("BellRowTwo")).toBeVisible();
  });

  test("pw-notifications-bell-item-safe-link-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        {
          title: "BellSafeNav",
          message: "click me",
          url: "/notifications",
        },
      ],
    });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    const before = await readBellBadgeCount(page);
    await openBellDropdown(page);
    await page.getByText("BellSafeNav").click();
    await expect(page).toHaveURL(/\/notifications\/?$/, { timeout: 30_000 });
    await expect
      .poll(async () => readBellBadgeCount(page), { timeout: 30_000 })
      .toBeLessThan(before);
  });

  test("pw-notifications-bell-item-unsafe-url-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        {
          title: "BellUnsafe",
          message: "stay put",
          url: "https://evil.example/x",
        },
      ],
    });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await openBellDropdown(page);
    await page.getByText("BellUnsafe").click();
    await expect(page).toHaveURL(/\/(?:notifications\/?)?$/, { timeout: 30_000 });
    await expect(page).not.toHaveURL(/evil\.example/);
  });

  test("pw-notifications-bell-infinite-scroll-happy", async ({ page }) => {
    const notifications = Array.from({ length: 15 }, (_, i) => ({
      title: `BellPage-${String(i).padStart(2, "0")}`,
      message: `row ${i}`,
      url: "/",
    }));
    await seedAuth(page, "authenticated_verified", { notifications });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await openBellDropdown(page);
    const countBellPageTitles = async () => {
      let n = 0;
      for (let i = 0; i < 15; i++) {
        const title = `BellPage-${String(i).padStart(2, "0")}`;
        if ((await page.getByText(title, { exact: true }).count()) > 0) {
          n += 1;
        }
      }
      return n;
    };
    await expect
      .poll(countBellPageTitles, { timeout: 60_000 })
      .toBe(10);
    for (let i = 0; i < 12; i++) {
      await page.evaluate(() => {
        const nodes = Array.from(document.querySelectorAll("*"));
        for (const el of nodes) {
          if (el.scrollHeight > el.clientHeight + 20 && el.clientHeight > 80) {
            el.scrollTop = el.scrollHeight;
          }
        }
      });
      if ((await countBellPageTitles()) >= 15) {
        break;
      }
      await page.waitForTimeout(400);
    }
    await expect.poll(countBellPageTitles, { timeout: 60_000 }).toBe(15);
  });

  test("pw-notifications-photon-push-badge-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "BeforePush", message: "baseline", url: "/" }],
    });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("notification-bell")).toBeVisible({
      timeout: 60_000,
    });
    const before = await readBellBadgeCount(page);
    expect(before).toBeGreaterThanOrEqual(1);

    await seedAuth(page, "authenticated_verified", {
      append: true,
      notifications: [
        {
          title: "LivePushRow",
          message: "Photon should refetch badge",
          url: "/",
        },
      ],
    });

    await expect
      .poll(async () => readBellBadgeCount(page), {
        message: "Photon /ws/notifications should bump unread badge without reload",
        timeout: 45_000,
      })
      .toBeGreaterThan(before);
  });

  test("pw-notifications-photon-ws-blocked-sad", async ({ page }) => {
    // HTTP `page.route` does not intercept WebSockets — use routeWebSocket.
    await page.routeWebSocket(/\/ws\/notifications/, (ws) => {
      ws.close();
    });
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "WsBlockedBase", message: "no live", url: "/" }],
    });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    const before = await readBellBadgeCount(page);
    // Wipe + mark-read fallback leaves only this seed's unread row(s).
    expect(before).toBe(1);

    await seedAuth(page, "authenticated_verified", {
      append: true,
      notifications: [{ title: "WsBlockedPush", message: "should not live-refetch", url: "/" }],
    });

    // Without WS, badge must not bump from Photon refetch (poll stays flat).
    await page.waitForTimeout(3_000);
    expect(await readBellBadgeCount(page)).toBe(before);
  });
});

test.describe("pw-notifications-inbox", () => {
  test("pw-notifications-inbox-bell-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        {
          title: "E2E inbox ping",
          message: "Seeded via System send_notification",
          url: "/notifications",
        },
      ],
    });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("e2e-auth-bootstrap")).toHaveAttribute(
      "data-auth",
      "authenticated_verified",
    );
    await expect(page.getByTestId("notification-bell")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("notification-bell-container")).toBeVisible();

    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("notifications-inbox-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("nav-notifications-inbox")).toBeAttached();
    await expect(page.getByText("E2E inbox ping")).toBeVisible({ timeout: 60_000 });
  });

  test("pw-notifications-inbox-min-width-happy", async ({ page }) => {
    await page.setViewportSize({ width: 320, height: 800 });
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "InboxWidthProbe", message: "min width floor" }],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("notifications-inbox-page")).toBeVisible({
      timeout: 60_000,
    });
    const box = await page.getByTestId("notifications-inbox-page").boundingBox();
    expect(box).toBeTruthy();
    expect(box!.width).toBeGreaterThanOrEqual(360);
  });

  test("pw-notifications-stats-grid-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "StatOne", message: "a" },
        { title: "StatTwo", message: "b" },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("StatOne")).toBeVisible({ timeout: 60_000 });
    await expect(page.locator("span").filter({ hasText: /^Unread$/ })).toBeVisible();
    await expect(page.locator("span").filter({ hasText: /^Total$/ })).toBeVisible();
    await expect(page.locator("span").filter({ hasText: /^Today$/ })).toBeVisible();
    await expect(page.getByText("2").first()).toBeVisible({ timeout: 30_000 });
  });

  test("pw-notifications-mark-read-happy", async ({ page }) => {
    const title = `MarkMeRead-${Date.now()}`;
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title, message: "toggle target" }],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText(title, { exact: true })).toBeVisible({ timeout: 60_000 });
    // Prefer the unread toggle; avoid ancestor filters that match sibling cards.
    await expect(page.getByRole("button", { name: "Mark read", exact: true }).first()).toBeVisible({
      timeout: 30_000,
    });
    await page.getByRole("button", { name: "Mark read", exact: true }).first().click();
    await expect(page.getByRole("button", { name: "Mark unread", exact: true }).first()).toBeVisible({
      timeout: 30_000,
    });
  });

  test("pw-notifications-mark-unread-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "Toggle unread", message: "round trip" }],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("Toggle unread")).toBeVisible({ timeout: 60_000 });
    await page.getByRole("button", { name: "Mark read", exact: true }).first().click();
    await expect(page.getByRole("button", { name: "Mark unread", exact: true }).first()).toBeVisible({
      timeout: 30_000,
    });
    await page.getByRole("button", { name: "Mark unread", exact: true }).first().click();
    await expect(page.getByRole("button", { name: "Mark read", exact: true }).first()).toBeVisible({
      timeout: 30_000,
    });
  });

  test("pw-notifications-mark-all-read-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "BulkA", message: "1" },
        { title: "BulkB", message: "2" },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("BulkA")).toBeVisible({ timeout: 60_000 });
    await page.getByRole("button", { name: "Mark all read", exact: true }).click();
    await expect(page.getByRole("button", { name: "Mark unread", exact: true }).first()).toBeVisible({
      timeout: 30_000,
    });
    await page.getByRole("button", { name: "Unread", exact: true }).click();
    await expect(page.getByText("No notifications")).toBeVisible({ timeout: 30_000 });
  });

  test("pw-notifications-mark-all-when-none-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "OnlyThenRead", message: "mark-all empty unread" }],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("OnlyThenRead")).toBeVisible({ timeout: 60_000 });
    await page.getByRole("button", { name: "Mark all read", exact: true }).click();
    await page.getByRole("button", { name: "Unread", exact: true }).click();
    await expect(page.getByText("You're all caught up")).toBeVisible({ timeout: 30_000 });
    await page.getByRole("button", { name: "Mark all read", exact: true }).click();
    await expect(page.getByText("You're all caught up")).toBeVisible({ timeout: 30_000 });
  });

  test("pw-notifications-filter-all-unread-read-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "FilterKeep", message: "unread" },
        { title: "FilterMark", message: "will read" },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("FilterKeep")).toBeVisible({ timeout: 60_000 });

    await page
      .locator("div")
      .filter({ hasText: /^FilterMark/ })
      .getByRole("button", { name: "Mark read", exact: true })
      .first()
      .click();
    await expect(page.getByRole("button", { name: "Mark unread", exact: true }).first()).toBeVisible({
      timeout: 30_000,
    });

    await page.getByRole("button", { name: "Unread", exact: true }).click();
    await expect(page.getByText("FilterKeep")).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText("FilterMark")).toHaveCount(0);

    await page.getByRole("button", { name: "Read", exact: true }).click();
    await expect(page.getByText("FilterMark")).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText("FilterKeep")).toHaveCount(0);

    await page.getByRole("button", { name: "All", exact: true }).click();
    await expect(page.getByText("FilterKeep")).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText("FilterMark")).toBeVisible();
  });

  test("pw-notifications-filter-search-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "AlphaPing", message: "keep me" },
        { title: "BetaPing", message: "filter me out" },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("AlphaPing")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByText("BetaPing")).toBeVisible();

    await page.getByPlaceholder("Search notifications...").fill("AlphaPing");
    await expect(page.getByText("AlphaPing")).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText("BetaPing")).toHaveCount(0);

    await page.getByPlaceholder("Search notifications...").fill("zz-no-match");
    await expect(page.getByText("No notifications")).toBeVisible({
      timeout: 30_000,
    });
  });

  test("pw-notifications-safe-url-nav-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        {
          title: "SafeInboxLink",
          message: "go home",
          url: "/",
        },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("SafeInboxLink")).toBeVisible({ timeout: 60_000 });
    await page.getByText("SafeInboxLink").click();
    await expect(page).toHaveURL(/\/$/, { timeout: 30_000 });
    await expect(page.getByTestId("shell-chrome-home")).toBeVisible({
      timeout: 30_000,
    });
  });

  test("pw-notifications-unsafe-url-fallback-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        {
          title: "UnsafeLink",
          message: "should stay on inbox",
          url: "https://evil.example/x",
        },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("UnsafeLink")).toBeVisible({ timeout: 60_000 });
    await page.getByText("UnsafeLink").click();
    await expect(page).toHaveURL(/\/notifications\/?$/, { timeout: 30_000 });
  });

  test("pw-notifications-inbox-pagination-happy", async ({ page }) => {
    const notifications = Array.from({ length: 25 }, (_, i) => ({
      title: `InboxPage-${String(i).padStart(2, "0")}`,
      message: `page row ${i}`,
      url: "/notifications",
    }));
    await seedAuth(page, "authenticated_verified", { notifications });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    const countInboxPageTitles = async () => {
      let n = 0;
      for (let i = 0; i < 25; i++) {
        const title = `InboxPage-${String(i).padStart(2, "0")}`;
        if ((await page.getByText(title, { exact: true }).count()) > 0) {
          n += 1;
        }
      }
      return n;
    };
    await expect
      .poll(countInboxPageTitles, { timeout: 60_000 })
      .toBe(20);
    for (let i = 0; i < 15; i++) {
      await page.evaluate(() => {
        const root = document.querySelector('[data-testid="notifications-inbox-page"]');
        if (root) {
          root.scrollTop = root.scrollHeight;
        }
        window.scrollTo(0, document.body.scrollHeight);
        for (const el of Array.from(document.querySelectorAll("*"))) {
          const style = window.getComputedStyle(el);
          if (
            (style.overflowY === "auto" || style.overflowY === "scroll") &&
            el.scrollHeight > el.clientHeight + 20
          ) {
            el.scrollTop = el.scrollHeight;
          }
        }
      });
      if ((await countInboxPageTitles()) >= 25) {
        break;
      }
      await page.waitForTimeout(500);
    }
    await expect.poll(countInboxPageTitles, { timeout: 90_000 }).toBe(25);
  });
});

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
  test("help-spotlight-once-authed", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-tour-player")).toBeAttached();
    await expect(page.getByTestId("help-step-notifications-bell")).toBeVisible({
      timeout: 60_000,
    });
    await completeVisibleTour(page);

    await page.reload({ waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-notifications-bell")).toHaveCount(0);
    await expect(page.getByTestId("help-step-notifications-inbox")).toHaveCount(0);
  });

  test("help-spotlight-replay-current-route", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-notifications-bell")).toBeVisible({
      timeout: 60_000,
    });
    await completeVisibleTour(page);
    await expect(page.getByTestId("help-step-notifications-bell")).toHaveCount(0);

    await openHelpMenu(page);
    await page.getByTestId("help-menu-replay-tour").click({ force: true });
    await expect(page.getByTestId("help-step-notifications-bell")).toBeVisible({
      timeout: 60_000,
    });
  });

  test("help-spotlight-skips-signin-gate", async ({ page }) => {
    await seedAuth(page, "anonymous", { help_tour: true });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached();
    await expect(page.getByText("Sign in required")).toBeVisible();
    await expect(page.getByTestId("help-step-notifications-bell")).toHaveCount(0);
    await expect(page.getByTestId("help-step-notifications-inbox")).toHaveCount(0);
    await expect(page.locator('[data-testid="spotlight-footer"]:visible')).toHaveCount(0);
  });

  test("help-spotlight-home-skips-bell", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", { help_tour: true });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByTestId("help-step-notifications-bell")).toHaveCount(0);
    await expect(page.locator('[data-testid="spotlight-footer"]:visible')).toHaveCount(0);
  });
});
