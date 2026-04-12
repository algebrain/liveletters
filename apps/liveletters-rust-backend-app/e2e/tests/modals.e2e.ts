import { test, expect } from "../helpers/fixtures.js";

test.describe("Modals (setup completed)", () => {
  test.use({ setupCompleted: true });

  test("screenshot: settings modal", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(1500);

    // Открываем модалку настроек через кнопку шестерёнки
    await page.getByTitle("Настройки").click();
    await page.waitForTimeout(500);

    await page.screenshot({
      path: "screenshots/settings-modal.png",
      fullPage: true,
    });
  });

  test("screenshot: add subscription modal", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(1500);

    // Открываем модалку добавления подписки
    await page.getByTitle("Добавить подписку").click();
    await page.waitForTimeout(500);

    await page.screenshot({
      path: "screenshots/add-subscription-modal.png",
      fullPage: true,
    });
  });
});
