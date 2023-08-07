import { test, expect } from "@playwright/test";

const addr = "http://localhost:3000/";

test("homepage has title and subtitle", async ({ page }) => {
  await page.goto(addr);

  await expect(page).toHaveTitle("Home");

  await expect(page.locator("h1.logo-font")).toHaveText("conduit");
  await expect(page.locator("body > main > div > div.banner > div > p")).toHaveText("A place to share your knowledge.")
});

test("signup, logout and login works", async ({ page }) => {
  await page.goto(addr + "signup");

  const username = (Math.random() + 1).toString(36).substring(7);
  const password = (Math.random() + 1).toString(36).substring(7);

  // Create user
  await page.getByPlaceholder("Your Username").fill(username);
  await page.getByPlaceholder("Password").fill(password);
  await page.getByPlaceholder("Email").fill(username + "@" + username + ".com");
  await page.getByRole("button", { name: "Sign up" }).click()

  // Logout user
  await page.waitForURL(addr);
  const logout = page.locator("body > nav > div > ul > li:nth-child(5) > form > button");
  await expect(logout).toBeVisible();
  await logout.click();

  // Login user
  await page.waitForURL(addr + "login")
  await page.getByPlaceholder("Your Username").fill(username);
  await page.getByPlaceholder("Password").fill(password);
  await page.getByRole("button", { name: "Sign in" }).click()

  await page.waitForURL(addr)
  await expect(page.locator("a.nav-link > i.ion-person")).toBeVisible()
});
