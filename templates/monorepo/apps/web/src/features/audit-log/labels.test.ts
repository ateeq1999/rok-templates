import { describe, expect, it } from "vitest"
import { actionMeta, detailEntries, shortId } from "./labels"

describe("actionMeta (action label mapping)", () => {
  it("maps the four known actions to readable labels and badges", () => {
    expect(actionMeta("user.created_by_admin")).toEqual({
      label: "User created",
      variant: "default",
    })
    expect(actionMeta("user.role_changed")).toEqual({
      label: "Role changed",
      variant: "secondary",
    })
    expect(actionMeta("user.active_status_changed")).toEqual({
      label: "Status changed",
      variant: "outline",
    })
    expect(actionMeta("user.deleted")).toEqual({
      label: "User deleted",
      variant: "destructive",
    })
  })

  it("renders an unknown action as its raw string", () => {
    expect(actionMeta("user.something_new")).toEqual({
      label: "user.something_new",
      variant: "outline",
    })
  })
})

describe("detailEntries", () => {
  it("flattens any object without assuming its keys", () => {
    expect(
      detailEntries({ old_role: "user", new_role: "admin", count: 2, ok: true })
    ).toEqual([
      { key: "old role", value: "user" },
      { key: "new role", value: "admin" },
      { key: "count", value: "2" },
      { key: "ok", value: "true" },
    ])
  })

  it("handles null, nested values and long values compactly", () => {
    const rows = detailEntries({ a: null, b: { x: 1 }, c: "z".repeat(200) })
    expect(rows[0]).toEqual({ key: "a", value: "—" })
    expect(rows[1]).toEqual({ key: "b", value: '{"x":1}' })
    expect(rows[2].value).toBe("z".repeat(200))
    const nested = detailEntries({ big: { s: "y".repeat(200) } })
    expect(nested[0].value.length).toBeLessThanOrEqual(80)
    expect(nested[0].value.endsWith("…")).toBe(true)
  })

  it("yields no rows for null or non-object details", () => {
    expect(detailEntries(null)).toEqual([])
    expect(detailEntries(undefined)).toEqual([])
    expect(detailEntries("text")).toEqual([])
    expect(detailEntries([1, 2])).toEqual([])
  })
})

describe("shortId", () => {
  it("shortens a uuid for display", () => {
    expect(shortId("3f2a9c81-aaaa-bbbb-cccc-1234567890ab")).toBe("3f2a9c81")
    expect(shortId("abc")).toBe("abc")
  })
})
