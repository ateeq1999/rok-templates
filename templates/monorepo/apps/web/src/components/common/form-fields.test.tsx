import { cleanup, render } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"
import { PasswordField, TextField } from "./form-fields"
import type { FieldLike } from "./form-fields"

afterEach(cleanup)

function fakeField(name: string): FieldLike {
  return {
    name,
    state: { value: "", meta: { isTouched: false, errors: [] } },
    form: { state: { isSubmitted: false } },
    handleBlur: vi.fn(),
    handleChange: vi.fn(),
  }
}

describe("form field DOM ids", () => {
  it("stay unique when two forms on one page use the same field name", () => {
    // e.g. the change-password card and the turn-off-2FA dialog both have
    // `current_password`; a shared id made the dialog's label focus the input behind it
    const { container } = render(
      <>
        <PasswordField
          field={fakeField("current_password")}
          label="Current password"
        />
        <PasswordField
          field={fakeField("current_password")}
          label="Current password"
        />
        <TextField field={fakeField("current_password")} label="Other" />
      </>
    )
    const inputs = [...container.querySelectorAll("input")]
    const ids = inputs.map((i) => i.id)
    expect(new Set(ids).size).toBe(ids.length)
    for (const input of inputs) {
      const label = container.querySelector(
        `label[for="${CSS.escape(input.id)}"]`
      )
      expect(label).not.toBeNull()
    }
  })

  it("keeps the field name as the form control name", () => {
    const { container } = render(
      <TextField field={fakeField("email")} label="Email" />
    )
    expect(container.querySelector("input")?.getAttribute("name")).toBe("email")
  })
})
