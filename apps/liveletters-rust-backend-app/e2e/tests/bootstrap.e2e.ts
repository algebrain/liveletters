import { test, expect } from "../helpers/fixtures.js";

async function waitForShellLayout(page: Parameters<typeof test>[0]["page"]) {
  await page.waitForFunction(() => {
    const topNav = document.querySelector(".ll-top-nav");
    const sidebar = document.querySelector(".ll-sidebar");
    const main = document.querySelector(".ll-main");

    if (!topNav || !sidebar || !main) {
      return false;
    }

    const topNavRect = topNav.getBoundingClientRect();
    const sidebarRect = sidebar.getBoundingClientRect();
    const mainRect = main.getBoundingClientRect();
    const topNavStyle = window.getComputedStyle(topNav);
    const mainStyle = window.getComputedStyle(main);

    return (
      topNavRect.height >= 44 &&
      sidebarRect.width >= 260 &&
      mainRect.left > sidebarRect.right - 4 &&
      topNavStyle.display === "flex" &&
      mainStyle.overflowY === "auto"
    );
  });
}

test.describe("Setup pending (initial setup)", () => {
  // setupCompleted: false — по умолчанию
  test("shows initial setup form", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 2 })).toBeVisible({
      timeout: 10000,
    });

    const headingText = await page
      .getByRole("heading", { level: 2 })
      .textContent();
    expect(headingText).toContain("Initial setup");

    // Проверяем наличие основных полей
    await expect(page.getByRole("textbox", { name: "Nickname" })).toBeVisible();
    await expect(page.getByRole("textbox", { name: "Email" })).toBeVisible();
    await expect(
      page.getByRole("textbox", { name: "SMTP host" }),
    ).toBeVisible();
    await expect(
      page.getByRole("textbox", { name: "IMAP host" }),
    ).toBeVisible();

    // Проверяем что ввод работает
    await page.getByRole("textbox", { name: "Nickname" }).fill("test-user");
    await expect(page.getByRole("textbox", { name: "Nickname" })).toHaveValue(
      "test-user",
    );
  });
});

test.describe("Setup completed", () => {
  test.use({ setupCompleted: true });

  test("shows three-panel layout with sidebar", async ({ page }) => {
    await page.goto("/");
    await waitForShellLayout(page);

    // Проверяем sidebar (используем role button чтобы избежать совпадений с заголовками)
    await expect(page.getByRole("button", { name: "Home" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Feed" })).toBeVisible();
    await expect(page.getByText(/подписки/i)).toBeVisible();

    // Проверяем навбар
    await expect(page.getByTitle("Написать пост")).toBeVisible();
    await expect(page.getByTitle("Добавить подписку")).toBeVisible();
    await expect(page.getByTitle("Настройки")).toBeVisible();

    // Проверяем контент — лента с фейковыми постами
    await expect(page.getByText("Первый тестовый пост")).toBeVisible();

    await page.screenshot({
      path: "screenshots/telegram-layout.png",
      fullPage: true,
    });
  });

  test("feed page screenshot", async ({ page }) => {
    await page.goto("/");
    await waitForShellLayout(page);
    await page.screenshot({ path: "screenshots/feed-page.png", fullPage: true });
  });
});
