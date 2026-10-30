import { StrictMode } from "react"
import { cleanup, render } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"
import { useCapturedToken } from "./use-captured-token"

afterEach(cleanup)

function Harness({
  token,
  strip,
  redeem,
}: {
  token: string | undefined
  strip: () => void
  redeem: (t: string) => void
}) {
  const captured = useCapturedToken(token, strip, redeem)
  return <div data-testid="token">{captured ?? "none"}</div>
}

describe("useCapturedToken", () => {
  it("redeems exactly once even under StrictMode's double effects", () => {
    const redeem = vi.fn()
    const strip = vi.fn()
    render(
      <StrictMode>
        <Harness token="tok-123" strip={strip} redeem={redeem} />
      </StrictMode>
    )
    expect(redeem).toHaveBeenCalledTimes(1)
    expect(redeem).toHaveBeenCalledWith("tok-123")
  })

  it("always strips the URL, even with no token", () => {
    const redeem = vi.fn()
    const strip = vi.fn()
    render(
      <StrictMode>
        <Harness token={undefined} strip={strip} redeem={redeem} />
      </StrictMode>
    )
    expect(strip).toHaveBeenCalled()
    expect(redeem).not.toHaveBeenCalled()
  })

  it("captures the initial token even if the caller's value changes later", () => {
    const redeem = vi.fn()
    const strip = vi.fn()
    const { getByTestId, rerender } = render(
      <Harness token="first" strip={strip} redeem={redeem} />
    )
    expect(getByTestId("token").textContent).toBe("first")
    rerender(<Harness token="second" strip={strip} redeem={redeem} />)
    expect(getByTestId("token").textContent).toBe("first")
    expect(redeem).toHaveBeenCalledTimes(1)
  })
})
