import { test as base, expect, type Page } from "@playwright/test";
import { TAURI_MOCK_SCRIPT } from "../helpers/tauri-mock.js";

/**
 * Фикстура с параметризованным mock Tauri API.
 *
 * Использование:
 *   test.describe('Setup completed', () => {
 *     test.use({ setupCompleted: true });
 *     test('shows layout', async ({ page }) => { ... });
 *   });
 */
export const test = base.extend<{
  page: Page;
  setupCompleted: boolean;
}>({
  setupCompleted: [false, { option: true }],

  page: async ({ page, setupCompleted }, use) => {
    // Подставляем значение setupCompleted в mock-скрипт
    const mockScript = TAURI_MOCK_SCRIPT.replace(
      "let setupCompleted = false;",
      `let setupCompleted = ${setupCompleted};`,
    );

    // Перехватываем console.log для отладки
    page.on("console", (msg) => {
      if (msg.type() === "error" || msg.text().includes("[tauri")) {
        console.log(`[browser] ${msg.text()}`);
      }
    });

    page.on("pageerror", (err) => {
      console.error(`[browser error] ${err.message}`);
    });

    // Внедряем mock Tauri API ДО загрузки frontend-бандла
    await page.addInitScript(mockScript);
    await use(page);
  },
});

export { expect };
