import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  page.on("console", (msg) => {
    let msgText = "";
    for (let i = 0; i < msg.args().length; ++i) {
      msgText += `${msg.args()[i]}`;
    }
    // eslint-disable-next-line no-console
    console.log(msgText);
  });
});

test("boa demo", async ({ page }) => {
  await page.goto("/", {
    // wait until all content is loaded
    waitUntil: "networkidle",
  });
  // wait for the code evaluate
  await page.waitForTimeout(2000);
  const output = page.getByTestId("output");
  const result = await output.innerHTML();
  console.log("eval result: ", result);
  await expect(result.match("Hello, World")?.length).toEqual(1);
});
