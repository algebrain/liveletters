import { test, expect } from "../helpers/fixtures.js";

test.describe("Initial Setup", () => {
  test("должен отобразить initial setup с формой настроек", async ({
    page,
  }) => {
    // 1. Открываем приложение — должен быть initial setup
    await page.goto("/");

    // Проверяем что страница загрузилась
    await expect(page.getByRole("heading", { level: 2 })).toBeVisible({
      timeout: 10000,
    });

    const headingText = await page
      .getByRole("heading", { level: 2 })
      .textContent();
    console.log(`Initial setup heading: "${headingText}"`);
    expect(headingText).toContain("Initial setup");

    // 2. Проверяем наличие основных полей (ищем по label)
    await expect(page.getByRole("textbox", { name: "Nickname" })).toBeVisible();
    await expect(page.getByRole("textbox", { name: "Email" })).toBeVisible();
    await expect(
      page.getByRole("textbox", { name: "SMTP host" }),
    ).toBeVisible();
    await expect(
      page.getByRole("textbox", { name: "IMAP host" }),
    ).toBeVisible();

    console.log("Все основные поля формы присутствуют");

    // 3. Заполняем поля — проверяем что ввод работает
    await page.getByRole("textbox", { name: "Nickname" }).fill("e2e-test-user");
    await expect(page.getByRole("textbox", { name: "Nickname" })).toHaveValue(
      "e2e-test-user",
    );

    console.log("Initial setup тест пройден");
  });
});
