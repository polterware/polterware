async (page) => {
  const root = "plans/sparky-next/status/w3-converge/";
  const widths = [[800, 560], [1280, 800], [1920, 1080]];
  const snap = async (name) => {
    await page.screenshot({
      path: `${root}after-${name}-${page.viewportSize().width}.png`,
      animations: "disabled",
      scale: "css",
    });
  };
  const goto = async (query) => {
    await page.goto("http://127.0.0.1:5179/?" + query);
    await page.locator(".window-title-bar, .sparky-titlebar").first().waitFor();
    await page.waitForTimeout(400);
  };
  for (const [width, height] of widths) {
    await page.setViewportSize({ width, height });
    await goto("app=polterware");
    await snap("polterware");
    await goto("app=sparky");
    await snap("sparky");
  }
  await page.setViewportSize({ width: 1280, height: 800 });
  await goto("app=sparky");
  await page.getByRole("button", { name: "Open Commander" }).click();
  await page.getByRole("dialog").waitFor();
  await snap("sparky-commander");
  return "ok";
}
