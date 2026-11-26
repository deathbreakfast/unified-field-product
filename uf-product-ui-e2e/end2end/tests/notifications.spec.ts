import {
  test,
  expect,
  seedAuth,
  waitForHydrated,
  readBellBadgeCount,
  openBellDropdown,
  measureBellDropdownWidth,
  type SeedNotification,
} from "./fixtures";
import type { Page } from "@playwright/test";

/** Seed row with an age, for day groups and time labels. */
type AgedNotification = SeedNotification & { age_secs?: number };

/** Fixture kinds registered by the e2e host (`seed.rs`). */
const KIND_ASSIGNED = "welcome.e2e_assigned"; // Welcome · Assigned to you · asks for action
const KIND_DIGEST = "apps.e2e_digest"; // Apps · Weekly digest

/** One inbox row by title. */
function inboxRow(page: Page, title: string) {
  return page
    .getByTestId("notifications-inbox-page")
    .getByTestId("notification-item")
    .filter({ hasText: title });
}

/** An inbox tab by its leading label ("All", "Unread", "Waiting on you"). */
function inboxTab(page: Page, label: string) {
  return page
    .getByTestId("notifications-inbox-page")
    .getByRole("tab", { name: new RegExp(`^${label}`) });
}

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
    await expect(page.getByText("You're all caught up")).toBeVisible({
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

  test("pw-notifications-tab-counts-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "CountAsk", message: "a", kind: KIND_ASSIGNED },
        { title: "CountDigest", message: "b", kind: KIND_DIGEST },
        { title: "CountPlain", message: "c" },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(inboxRow(page, "CountAsk")).toBeVisible({ timeout: 60_000 });
    await expect(inboxTab(page, "All")).toHaveText("All (3)", { timeout: 30_000 });
    await expect(inboxTab(page, "Unread")).toHaveText("Unread (3)");
    await expect(inboxTab(page, "Waiting on you")).toHaveText("Waiting on you (1)");

    // A single Mark read updates the Unread count without reloading the list.
    await inboxRow(page, "CountPlain").getByRole("button", { name: "Mark read", exact: true }).click();
    await expect(inboxTab(page, "Unread")).toHaveText("Unread (2)", { timeout: 30_000 });
    await expect(inboxTab(page, "All")).toHaveText("All (3)");
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
    await inboxTab(page, "Unread").click();
    await expect(page.getByText("You're all caught up")).toBeVisible({ timeout: 30_000 });
    await expect(inboxTab(page, "Unread")).toHaveText("Unread (0)");
  });

  test("pw-notifications-mark-all-when-none-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "OnlyThenRead", message: "mark-all empty unread" }],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("OnlyThenRead")).toBeVisible({ timeout: 60_000 });
    await page.getByRole("button", { name: "Mark all read", exact: true }).click();
    await inboxTab(page, "Unread").click();
    await expect(page.getByText("You're all caught up")).toBeVisible({ timeout: 30_000 });
    await page.getByRole("button", { name: "Mark all read", exact: true }).click();
    await expect(page.getByText("You're all caught up")).toBeVisible({ timeout: 30_000 });
  });

  test("pw-notifications-tabs-all-unread-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "FilterKeep", message: "unread" },
        { title: "FilterMark", message: "will read" },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(inboxRow(page, "FilterKeep")).toBeVisible({ timeout: 60_000 });
    const tabList = await page.getByTestId("notifications-inbox-page").getByRole("tablist").elementHandle();

    await inboxRow(page, "FilterMark").getByRole("button", { name: "Mark read", exact: true }).click();
    await expect(
      inboxRow(page, "FilterMark").getByRole("button", { name: "Mark unread", exact: true }),
    ).toBeVisible({ timeout: 30_000 });

    await inboxTab(page, "Unread").click();
    await expect(inboxTab(page, "Unread")).toHaveAttribute("aria-selected", "true");
    await expect(inboxRow(page, "FilterKeep")).toBeVisible({ timeout: 30_000 });
    await expect(inboxRow(page, "FilterMark")).toHaveCount(0);

    await inboxTab(page, "All").click();
    await expect(inboxRow(page, "FilterKeep")).toBeVisible({ timeout: 30_000 });
    await expect(inboxRow(page, "FilterMark")).toBeVisible();

    // Switching tabs reloads only the list: the tab list is the same element.
    expect(await tabList!.evaluate((el) => el.isConnected)).toBe(true);
  });

  test("pw-notifications-waiting-on-you-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "WaitAsk", message: "please review", kind: KIND_ASSIGNED },
        { title: "WaitDigest", message: "fyi", kind: KIND_DIGEST },
        { title: "WaitPlain", message: "unregistered kind" },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(inboxRow(page, "WaitDigest")).toBeVisible({ timeout: 60_000 });
    await inboxTab(page, "Waiting on you").click();
    await expect(inboxRow(page, "WaitAsk")).toBeVisible({ timeout: 30_000 });
    await expect(inboxRow(page, "WaitDigest")).toHaveCount(0);
    await expect(inboxRow(page, "WaitPlain")).toHaveCount(0);
  });

  test("pw-notifications-waiting-on-you-empty-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "OnlyDigest", message: "fyi", kind: KIND_DIGEST }],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(inboxRow(page, "OnlyDigest")).toBeVisible({ timeout: 60_000 });
    await inboxTab(page, "Waiting on you").click();
    await expect(page.getByText("Nothing is waiting on you")).toBeVisible({ timeout: 30_000 });
  });

  test("pw-notifications-app-filter-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "AppWelcomeRow", message: "from welcome", kind: KIND_ASSIGNED },
        { title: "AppAppsRow", message: "from apps", kind: KIND_DIGEST },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(inboxRow(page, "AppWelcomeRow")).toBeVisible({ timeout: 60_000 });
    const select = page.locator("#notifications-app-filter");
    await expect(select.locator("option", { hasText: /^Apps$/ })).toHaveCount(1, { timeout: 30_000 });
    await select.selectOption("apps");
    await expect(inboxRow(page, "AppWelcomeRow")).toHaveCount(0, { timeout: 30_000 });
    await expect(inboxRow(page, "AppAppsRow")).toBeVisible();

    await select.selectOption("");
    await expect(inboxRow(page, "AppWelcomeRow")).toBeVisible({ timeout: 30_000 });
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
    await expect(inboxRow(page, "AlphaPing")).toBeVisible({ timeout: 60_000 });
    await expect(inboxRow(page, "BetaPing")).toBeVisible();

    const search = page.getByPlaceholder("Search notifications");
    await search.fill("AlphaPing");
    await expect(inboxRow(page, "AlphaPing")).toBeVisible({ timeout: 30_000 });
    await expect(inboxRow(page, "BetaPing")).toHaveCount(0);

    await search.fill("zz-no-match");
    await expect(page.getByText("No notifications match")).toBeVisible({ timeout: 30_000 });
    await page.getByRole("button", { name: "Clear filters", exact: true }).click();
    await expect(search).toHaveValue("");
    await expect(inboxRow(page, "BetaPing")).toBeVisible({ timeout: 30_000 });
  });

  test("pw-notifications-mark-read-failure-toast-sad", async ({ page }) => {
    const title = `FailRead-${Date.now()}`;
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title, message: "server says no" }],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(inboxRow(page, title)).toBeVisible({ timeout: 60_000 });

    await page.route(/\/api\/mark_notification_read/, (route) =>
      route.fulfill({ status: 500, body: "stubbed failure" }),
    );
    await inboxRow(page, title).getByRole("button", { name: "Mark read", exact: true }).click();
    await expect(page.getByText(`Couldn't mark “${title}” as read`)).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByText("It's still unread.")).toBeVisible();
    // The row flips back.
    await expect(
      inboxRow(page, title).getByRole("button", { name: "Mark read", exact: true }),
    ).toBeVisible({ timeout: 30_000 });
    await expect(inboxRow(page, title)).toHaveAttribute("data-read", "false");

    await page.unroute(/\/api\/mark_notification_read/);
    await page.getByRole("button", { name: "Try again", exact: true }).click();
    await expect(
      inboxRow(page, title).getByRole("button", { name: "Mark unread", exact: true }),
    ).toBeVisible({ timeout: 30_000 });
  });

  test("pw-notifications-load-error-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "LoadErrorProbe", message: "comes back" }],
    });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    // Navigate client-side so the list loads from the browser, where the stub applies.
    await page.route(/\/api\/get_inbox_rows_page/, (route) =>
      route.fulfill({ status: 500, body: "stubbed failure" }),
    );
    await openBellDropdown(page);
    await page.getByRole("menuitem", { name: "Open inbox" }).click();
    await expect(page.getByTestId("notifications-load-error")).toBeVisible({ timeout: 60_000 });
    await expect(
      page.getByText(
        "Couldn't load your notifications. Nothing was lost; this is a problem on our side.",
      ),
    ).toBeVisible();
    await expect(page.getByTestId("notifications-end")).toHaveCount(0);

    await page.unroute(/\/api\/get_inbox_rows_page/);
    await page
      .getByTestId("notifications-load-error")
      .getByRole("button", { name: "Try again", exact: true })
      .click();
    await expect(inboxRow(page, "LoadErrorProbe")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByTestId("notifications-load-error")).toHaveCount(0);
  });

  test("pw-notifications-row-actions-aligned-happy", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "AlignShort", message: "x", url: "/", kind: KIND_DIGEST },
        {
          title: "AlignLongTitleThatWrapsFurtherThanTheOther",
          message: "a longer message to make this row taller than the first one",
          url: "/",
          kind: KIND_ASSIGNED,
        },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(inboxRow(page, "AlignShort")).toBeVisible({ timeout: 60_000 });
    // Different labels on the first button ("Mark unread" vs "Mark read").
    await inboxRow(page, "AlignShort").getByRole("button", { name: "Mark read", exact: true }).click();
    await expect(
      inboxRow(page, "AlignShort").getByRole("button", { name: "Mark unread", exact: true }),
    ).toBeVisible({ timeout: 30_000 });

    // Measure the buttons themselves (by role) in each row.
    const rows = ["AlignShort", "AlignLongTitleThatWrapsFurtherThanTheOther"].map((t) =>
      inboxRow(page, t),
    );
    const openButtons = rows.map((r) => r.getByRole("button", { name: "Open", exact: true }));
    const moreButtons = rows.map((r) => r.getByRole("button", { name: "More actions" }));
    for (const b of [...openButtons, ...moreButtons]) {
      await expect(b).toBeVisible({ timeout: 30_000 });
    }
    const [o1, o2] = await Promise.all(openButtons.map((b) => b.boundingBox()));
    const [m1, m2] = await Promise.all(moreButtons.map((b) => b.boundingBox()));
    expect(Math.abs(o1!.x - o2!.x)).toBeLessThanOrEqual(1);
    expect(Math.abs(m1!.x - m2!.x)).toBeLessThanOrEqual(1);
  });

  test("pw-notifications-row-menu-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "MenuRow", message: "menu", kind: KIND_ASSIGNED }],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(inboxRow(page, "MenuRow")).toBeVisible({ timeout: 60_000 });
    await inboxRow(page, "MenuRow").getByTestId("notification-more").click();
    await expect(
      page.getByRole("menuitem", { name: "Change how I get “Assigned to you”" }),
    ).toBeVisible({ timeout: 30_000 });
    await page.getByRole("menuitem", { name: "Mark read" }).click();
    await expect(inboxRow(page, "MenuRow")).toHaveAttribute("data-read", "true", {
      timeout: 30_000,
    });
  });

  test("pw-notifications-phone-menu-only-happy", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "PhoneRow", message: "tap me", url: "/" }],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    const row = inboxRow(page, "PhoneRow");
    await expect(row).toBeVisible({ timeout: 60_000 });
    await expect(row.getByTestId("notification-more")).toBeVisible();
    await expect(row.getByTestId("notification-open")).toHaveCount(0);
    await expect(row.getByRole("button", { name: "Mark read", exact: true })).toHaveCount(0);
    await row.getByText("PhoneRow", { exact: true }).click();
    await expect(page).toHaveURL(/\/$/, { timeout: 30_000 });
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

test.describe("pw-notifications-kinds-and-times", () => {
  test.use({ timezoneId: "America/Los_Angeles" });

  test("pw-notifications-bell-caption-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        {
          title: "BellKindRow",
          message: "with app, kind, sender and time",
          kind: KIND_ASSIGNED,
          sender_name: "Ada Sender",
        },
      ],
    });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await openBellDropdown(page);
    const bell = page.getByTestId("notification-bell-dropdown");
    await expect(bell.getByText("BellKindRow")).toBeVisible({ timeout: 30_000 });
    await expect(bell.getByTestId("notification-meta")).toHaveText(
      /^Ada Sender · Welcome · Assigned to you · (Just now|\d+ min ago)$/,
    );
    await expect(bell.locator("time")).toHaveAttribute("datetime", /Z$/);
    await expect(page.getByRole("menuitem", { name: "Open inbox" })).toBeVisible();
  });

  test("pw-notifications-bell-tabs-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "BellTabOne", message: "1", url: "/" },
        { title: "BellTabTwo", message: "2", url: "/" },
      ],
    });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await openBellDropdown(page);
    const bell = page.getByTestId("notification-bell-dropdown");
    await expect(bell.getByRole("tab", { name: /^Unread/ })).toHaveText("Unread (2)", {
      timeout: 30_000,
    });
    await expect(bell.getByText("BellTabOne")).toBeVisible();

    await bell.getByRole("button", { name: "Mark all read", exact: true }).click();
    await expect(bell.getByText("You're all caught up")).toBeVisible({ timeout: 30_000 });

    await bell.getByRole("tab", { name: "All" }).click();
    await expect(bell.getByText("BellTabOne")).toBeVisible({ timeout: 30_000 });
    await expect(bell.getByText("BellTabTwo")).toBeVisible();
  });

  test("pw-notifications-day-groups-happy", async ({ page }) => {
    const rows: AgedNotification[] = [
      { title: "DayToday", message: "now", age_secs: 0 },
      { title: "DayYesterday", message: "a day ago", age_secs: 86_400 },
      { title: "DayEarlier", message: "four days ago", age_secs: 4 * 86_400 },
    ];
    await seedAuth(page, "authenticated_verified", { notifications: rows });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(inboxRow(page, "DayEarlier")).toBeVisible({ timeout: 60_000 });

    const groups = page.getByTestId("notifications-day-group");
    await expect(groups).toHaveCount(3, { timeout: 30_000 });
    await expect(groups.nth(0)).toHaveAttribute("data-group", "Today");
    await expect(groups.nth(1)).toHaveAttribute("data-group", "Yesterday");
    await expect(groups.nth(2)).toHaveAttribute("data-group", "Earlier");
    await expect(groups.nth(0).getByText("DayToday")).toBeVisible();
    await expect(groups.nth(1).getByText("DayYesterday")).toBeVisible();
    await expect(groups.nth(2).getByText("DayEarlier")).toBeVisible();

    await expect(inboxRow(page, "DayYesterday").locator("time")).toHaveText(
      /^Yesterday, \d{1,2}:\d{2} (AM|PM)$/,
    );
    await expect(inboxRow(page, "DayEarlier").locator("time")).toHaveText(
      /^[A-Z][a-z]{2} \d{1,2}(, \d{4})?, \d{1,2}:\d{2} (AM|PM) P[SD]T$/,
    );
    await expect(page.getByTestId("notifications-end")).toHaveText(
      "That's everything from the last 90 days.",
    );
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

test.describe("pw-notifications-sender", () => {
  test("pw-notifications-sender-name-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "SenderNameProbe", message: "sent by a person", sender_name: "Ada Sender" },
      ],
    });
    // Open the bell on `/`: on the inbox page "Notifications" also names the page,
    // which trips openBellDropdown's exact-text wait.
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await openBellDropdown(page);
    const bell = page.getByTestId("notification-bell-dropdown");
    await expect(bell.getByText("SenderNameProbe")).toBeVisible({ timeout: 30_000 });
    await expect(bell.getByTestId("notification-sender")).toHaveText("Ada Sender");

    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("SenderNameProbe")).toBeVisible({ timeout: 60_000 });
    const inbox = page.getByTestId("notifications-inbox-page");
    await expect(inbox.getByTestId("notification-sender")).toHaveText("Ada Sender");
  });

  test("pw-notifications-sender-fallback-sad", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [{ title: "SystemSenderProbe", message: "no sender on this row" }],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("SystemSenderProbe")).toBeVisible({ timeout: 60_000 });
    const inbox = page.getByTestId("notifications-inbox-page");
    await expect(inbox.getByTestId("notification-sender")).toHaveText("Someone");
  });

  test("pw-notifications-sender-renamed-happy", async ({ page }) => {
    await seedAuth(page, "authenticated_verified", {
      notifications: [
        { title: "RenameProbeFirst", message: "before rename", sender_name: "Old Name" },
      ],
    });
    // Renaming the sender's profile changes the name on the earlier row too:
    // the name is looked up when the page loads, never stored on the row.
    await seedAuth(page, "authenticated_verified", {
      append: true,
      notifications: [
        { title: "RenameProbeSecond", message: "after rename", sender_name: "New Name" },
      ],
    });
    await page.goto("/notifications", { waitUntil: "domcontentloaded" });
    await waitForHydrated(page);
    await expect(page.getByText("RenameProbeFirst")).toBeVisible({ timeout: 60_000 });
    const senders = page.getByTestId("notifications-inbox-page").getByTestId("notification-sender");
    await expect(senders).toHaveCount(2);
    await expect(senders).toHaveText(["New Name", "New Name"]);
  });
});
