import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { cleanup, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"
import { signInProvidersQueryOptions } from "../queries"
import { ContinueWithButtons } from "./continue-with-buttons"

afterEach(cleanup)

/** The buttons, with the API having answered `providers` already. */
function renderWith(providers: { name: string }[]) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: Infinity } },
  })
  qc.setQueryData(signInProvidersQueryOptions.queryKey, providers)
  return render(
    <QueryClientProvider client={qc}>
      <ContinueWithButtons />
    </QueryClientProvider>
  )
}

describe("ContinueWithButtons", () => {
  it("shows one button per configured provider, each starting its sign-in", () => {
    renderWith([{ name: "google" }, { name: "acme-id" }])

    const google = screen.getByRole("link", { name: "Continue with Google" })
    expect(google.getAttribute("href")).toBe("/api/v1/auth/oidc/google/start")
    const acme = screen.getByRole("link", { name: "Continue with Acme Id" })
    expect(acme.getAttribute("href")).toBe("/api/v1/auth/oidc/acme-id/start")
    expect(screen.getAllByRole("link")).toHaveLength(2)
  })

  it("shows nothing when no provider is configured", () => {
    const { container } = renderWith([])

    expect(screen.queryByRole("link")).toBeNull()
    expect(container.textContent).toBe("")
  })
})
