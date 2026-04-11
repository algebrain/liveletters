import { defineConfig } from "@playwright/test";
import path from "node:path";
import url from "node:url";

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
const backendRoot = path.resolve(__dirname, "..");
const frontendResources = path.join(
  backendRoot,
  "..",
  "liveletters-frontend-app",
  "resources",
);

const FRONTEND_PORT = 3456;

/**
 * E2E-контур для LiveLetters на Playwright.
 *
 * Архитектура:
 * - Playwright запускает WebKit (тот же движок, что и Tauri на Linux)
 * - Фронтенд ресурсы раздаются как статические файлы
 * - Tauri IPC API (window.__TAURI__) мокается через page.addInitScript
 *
 * Это проверяет реальную UI-логику и рендеринг.
 * Native-оболочка и IPC-мост тестируются отдельно через integration tests.
 */
export default defineConfig({
  testDir: "./tests",
  testMatch: "*.e2e.ts",
  fullyParallel: false,
  forbidOnly: false,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: process.env.WDIO_DIAGNOSTIC ? "line" : "list",
  timeout: 30000,

  use: {
    baseURL: `http://localhost:${FRONTEND_PORT}`,
    trace: "off",
    screenshot: "only-on-failure",
    video: "off",
  },

  projects: [
    {
      name: "webkit",
      use: {
        browserName: "webkit",
        viewport: { width: 1200, height: 800 },
        locale: "ru-RU",
      },
    },
  ],

  webServer: {
    command: `npx -y serve "${frontendResources}" --listen ${FRONTEND_PORT} --single`,
    url: `http://localhost:${FRONTEND_PORT}`,
    reuseExistingServer: true,
    timeout: 15000,
  },
});
