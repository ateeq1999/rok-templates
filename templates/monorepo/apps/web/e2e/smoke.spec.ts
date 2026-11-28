import { expect, test } from "@playwright/test"

// `e2e/global-setup.ts` sets these: the account, or why there is no API.
const skip = process.env.E2E_SKIP
const email = process.env.E2E_EMAIL ?? ""
const password = process.env.E2E_PASSWORD ?? ""

test("signs in with the dev seed's account, lands on home and signs out", async ({
  page,
}) => {
  test.skip(Boolean(skip), skip)

  await page.goto("/login")
  await page.getByLabel("Email").fill(email)
  await page.getByLabel("Password", { exact: true }).fill(password)
  await page.getByRole("button", { name: "Sign in", exact: true }).click()

  // An ordinary user's home is their profile.
  await expect(page).toHaveURL(/\/profile$/)

  await page.getByRole("button", { name: new RegExp(escape(email)) }).click()
  await page.getByRole("menuitem", { name: "Sign out" }).click()

  await expect(page).toHaveURL(/\/login$/)
  await expect(
    page.getByRole("button", { name: "Sign in", exact: true })
  ).toBeVisible()
})

function escape(text: string) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
}
