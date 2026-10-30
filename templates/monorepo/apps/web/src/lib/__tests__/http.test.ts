import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { ApiError, apiUrl, http } from "../http"
import { toServerErrors } from "../forms"
import { session } from "../session"

function mockFetch(status: number, body?: unknown) {
  const fn = vi.fn(async () =>
    status === 204
      ? new Response(null, { status })
      : new Response(JSON.stringify(body), {
          status,
          headers: { "content-type": "application/json" },
        })
  )
  vi.stubGlobal("fetch", fn)
  return fn
}

const validation = {
  error: {
    code: "validation_failed",
    message: "validation failed",
    details: {
      email: [{ code: "email", message: "must be a valid email" }],
      password: [{ code: "length", message: "must be 8-128 characters" }],
    },
  },
}

beforeEach(() => session.clear())
afterEach(() => vi.unstubAllGlobals())

describe("http", () => {
  it("sends the bearer token and JSON content type", async () => {
    session.login("abc")
    const fetchMock = mockFetch(200, { ok: true })
    await http("/api/v1/x", { method: "POST", body: { a: 1 } })
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit]
    const headers = init.headers as Record<string, string>
    expect(headers.Authorization).toBe("Bearer abc")
    expect(headers["Content-Type"]).toBe("application/json")
    expect(init.body).toBe('{"a":1}')
  })

  it("sends a rawBody blob as-is, with its content type", async () => {
    const fetchMock = mockFetch(200, { ok: true })
    const blob = new Blob(["fake-bytes"], { type: "image/png" })
    await http("/api/v1/users/me/avatar", { method: "PUT", rawBody: blob })
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit]
    const headers = init.headers as Record<string, string>
    expect(headers["Content-Type"]).toBe("image/png")
    expect(init.body).toBe(blob)
  })

  it("merges custom headers, e.g. X-QR-Secret", async () => {
    const fetchMock = mockFetch(200, { status: "pending" })
    await http("/api/v1/auth/qr/sessions/x", {
      headers: { "X-QR-Secret": "shh" },
      skipExpire: true,
    })
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit]
    const headers = init.headers as Record<string, string>
    expect(headers["X-QR-Secret"]).toBe("shh")
  })

  it("apiUrl prefixes VITE_API_URL", () => {
    expect(apiUrl("/api/v1/avatars/x.jpg")).toBe("/api/v1/avatars/x.jpg")
  })

  it("returns undefined for 204", async () => {
    mockFetch(204)
    await expect(
      http("/api/v1/users/1", { method: "DELETE" })
    ).resolves.toBeUndefined()
  })

  it("parses the error contract into ApiError", async () => {
    mockFetch(422, validation)
    const err = await http("/x").catch((e: unknown) => e)
    expect(err).toBeInstanceOf(ApiError)
    expect((err as ApiError).status).toBe(422)
    expect((err as ApiError).code).toBe("validation_failed")
    expect((err as ApiError).details?.email[0].message).toBe(
      "must be a valid email"
    )
  })

  it("maps network failures to status 0", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Promise.reject(new TypeError("boom")))
    )
    const err = await http("/x").catch((e: unknown) => e)
    expect((err as ApiError).status).toBe(0)
  })

  it("expires the session once on 401, but not for skipExpire calls", async () => {
    session.login("abc")
    const onExpire = vi.fn()
    const off = session.onExpire(onExpire)
    mockFetch(401, { error: { code: "unauthorized", message: "nope" } })
    await Promise.allSettled([http("/a"), http("/b"), http("/c")])
    expect(onExpire).toHaveBeenCalledTimes(1)
    expect(session.token).toBeNull()

    session.login("def")
    await http("/login", { skipExpire: true }).catch(() => {})
    expect(session.token).toBe("def")
    off()
  })
})

describe("toServerErrors", () => {
  it("maps 422 details onto fields", () => {
    const err = new ApiError(422, "validation_failed", "x", {
      email: [{ code: "email", message: "must be a valid email" }],
    })
    expect(toServerErrors(err).fields).toEqual({
      email: "must be a valid email",
    })
  })

  it("maps 409 onto the conflict field", () => {
    const err = new ApiError(409, "conflict", "email is already registered")
    expect(toServerErrors(err, { conflictField: "email" }).fields).toEqual({
      email: "email is already registered",
    })
  })

  it("turns other errors into a form message", () => {
    expect(
      toServerErrors(new ApiError(400, "bad_request", "wrong password")).form
    ).toBe("wrong password")
    expect(
      toServerErrors(new ApiError(429, "too_many_requests", "x")).form
    ).toMatch(/minute/)
  })
})
