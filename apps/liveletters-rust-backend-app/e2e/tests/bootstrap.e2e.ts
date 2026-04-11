import { test, expect } from "../helpers/fixtures.js";

test.describe("Initial Setup", () => {
  test("должен открыть initial setup, заполнить настройки и сохранить", async ({
    page,
  }) => {
    // 1. Открываем приложение — должен быть initial setup
    await page.goto("/");

    // Ждём когда frontend инициализируется и adapter будет установлен
    // Кнопка disabled пока adapter === nil
    // Подождём пока кнопка станет enabled (adapter установлен и форма submittable)
    const saveButton = page.getByRole("button", { name: /Save/i });

    // Сначала проверим что страница загрузилась
    await expect(page.getByRole("heading", { level: 2 })).toBeVisible({
      timeout: 10000,
    });

    const headingText = await page
      .getByRole("heading", { level: 2 })
      .textContent();
    console.log(`Initial setup heading: "${headingText}"`);

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

    // 3. Заполняем минимальные настройки
    await page.getByRole("textbox", { name: "Nickname" }).fill("e2e-test-user");
    await page.getByRole("textbox", { name: "Email" }).fill("e2e@test.local");
    await page
      .getByRole("textbox", { name: "SMTP host" })
      .fill("smtp.test.local");
    await page.getByRole("textbox", { name: "SMTP port" }).fill("587");
    await page
      .getByRole("textbox", { name: "SMTP username" })
      .fill("smtp-user");
    await page
      .getByRole("textbox", { name: "SMTP password" })
      .fill("smtp-pass");
    await page
      .getByRole("textbox", { name: "IMAP host" })
      .fill("imap.test.local");
    await page.getByRole("textbox", { name: "IMAP port" }).fill("143");
    await page
      .getByRole("textbox", { name: "IMAP username" })
      .fill("imap-user");
    await page
      .getByRole("textbox", { name: "IMAP password" })
      .fill("imap-pass");

    // 4. Ждём пока кнопка станет enabled (adapter установлен + форма valid)
    await expect(saveButton).toBeEnabled({ timeout: 10000 });
    await saveButton.click();

    // 5. После сохранения — переход на feed (initial setup меняется на "Home feed")
    await expect(
      page.getByRole("heading", { level: 2, name: "Home feed" }),
    ).toBeVisible({
      timeout: 10000,
    });

    console.log("Initial setup успешно пройден — перейдено на feed");
  });
});
