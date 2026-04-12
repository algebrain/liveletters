import { test } from "../helpers/fixtures.js";

test.describe("Editor page", () => {
  test.use({ setupCompleted: true });

  test("screenshot: editor page", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(1500);

    await page.getByTitle("Написать пост").click();
    await page.waitForTimeout(500);

    await page.screenshot({
      path: "screenshots/editor-page.png",
      fullPage: true,
    });
  });
});
