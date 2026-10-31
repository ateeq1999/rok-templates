import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { ApiError, api, apiUrl } from "../client"
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

describe("the generated client, configured with the session", () => {
  it("sends the bearer token and JSON content type", async () => {
    session.login("abc")
    const fetchMock = mockFetch(200, { ok: true })
    await api.users.updateMe({ display_name: "Ada" })
    const [url, init] = fetchMock.mock.calls[0] as unknown as [
      string,
      RequestInit,
    ]
    const headers = init.headers as Record<string, string>
    expect(url).toBe("/api/v1/users/me")
    expect(init.method).toBe("PATCH")
    expect(headers.Authorization).toBe("Bearer abc")
    expect(headers["Content-Type"]).toBe("application/json")
    expect(init.body).toBe('{"display_name":"Ada"}')
  })

  it("sends a raw body blob as-is, with its content type", async () => {
    const fetchMock = mockFetch(200, { ok: true })
    const blob = new Blob(["fake-bytes"], { type: "image/png" })
    await api.avatars.upload(blob)
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit]
    const headers = init.headers as Record<string, string>
    expect(headers["Content-Type"]).toBe("image/png")
    expect(init.body).toBe(blob)
  })

  it("merges custom headers", async () => {
    const fetchMock = mockFetch(200, { status: "pending" })
    await api.users.me({ headers: { "X-Trace": "shh" } })
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit]
    const headers = init.headers as Record<string, string>
    expect(headers["X-Trace"]).toBe("shh")
  })

  it("apiUrl prefixes VITE_API_URL", () => {
    expect(apiUrl("/api/v1/avatars/x.jpg")).toBe("/api/v1/avatars/x.jpg")
  })

  it("returns undefined for 204", async () => {
    mockFetch(204)
    await expect(api.users.remove("1")).resolves.toBeUndefined()
  })

  it("parses the error contract into ApiError", async () => {
    mockFetch(422, validation)
    const err = await api.users.me().catch((e: unknown) => e)
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
    const err = await api.users.me().catch((e: unknown) => e)
    expect((err as ApiError).status).toBe(0)
  })

  it("expires the session once on 401, but not for keepSession calls", async () => {
    session.login("abc")
    const onExpire = vi.fn()
    const off = session.onExpire(onExpire)
    mockFetch(401, { error: { code: "unauthorized", message: "nope" } })
    await Promise.allSettled([
      api.users.me(),
      api.users.getOne("b"),
      api.apiKeys.list(),
    ])
    expect(onExpire).toHaveBeenCalledTimes(1)
    expect(session.token).toBeNull()

    session.login("def")
    await api.auth.setup({ keepSession: true }).catch(() => {})
    expect(session.token).toBe("def")
    off()
  })

  it("sends no token to a public operation, whose 401 keeps the session", async () => {
    session.login("abc")
    const onExpire = vi.fn()
    const off = session.onExpire(onExpire)
    const fetchMock = mockFetch(401, {
      error: { code: "unauthorized", message: "invalid email or password" },
    })
    await api.auth.login({ email: "a@b.co", password: "wrong" }).catch(() => {})
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit]
    expect((init.headers as Record<string, string>).Authorization).toBe(
      undefined
    )
    expect(onExpire).not.toHaveBeenCalled()
    expect(session.token).toBe("abc")
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
