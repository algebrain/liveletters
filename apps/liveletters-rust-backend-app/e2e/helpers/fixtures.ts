import { test as base, expect, type Page } from "@playwright/test";
import { TAURI_MOCK_SCRIPT } from "../helpers/tauri-mock.js";

export const test = base.extend<{ page: Page }>({
  page: async ({ page }, use) => {
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
    await page.addInitScript(TAURI_MOCK_SCRIPT);
    await use(page);
  },
});

export { expect };
