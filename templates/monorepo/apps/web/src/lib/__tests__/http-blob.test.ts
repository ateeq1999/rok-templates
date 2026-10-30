import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { ApiError, http, httpBlob } from "../http"
import { toServerErrors } from "../forms"
import { session } from "../session"

function stubFetch(response: Response) {
  const fn = vi.fn(async () => response)
  vi.stubGlobal("fetch", fn)
  return fn
}

const json = (status: number, body: unknown) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  })

beforeEach(() => session.clear())
afterEach(() => vi.unstubAllGlobals())

describe("httpBlob", () => {
  it("sends the bearer token and returns the bytes", async () => {
    session.login("tok")
    const fetchMock = stubFetch(
      new Response(new TextEncoder().encode("pdf-bytes"), {
        status: 200,
        headers: { "content-type": "application/pdf" },
      })
    )
    const blob = await httpBlob("/api/v1/media/1/content")
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit]
    expect((init.headers as Record<string, string>).Authorization).toBe(
      "Bearer tok"
    )
    expect(await blob.text()).toBe("pdf-bytes")
  })

  it("maps errors like http() does", async () => {
    stubFetch(json(404, { error: { code: "not_found", message: "not found" } }))
    const err = await httpBlob("/api/v1/media/x/content").catch(
      (e: unknown) => e
    )
    expect(err).toBeInstanceOf(ApiError)
    expect((err as ApiError).status).toBe(404)
  })
})

describe("error messages", () => {
  it("shows the server message for 413 and 415", async () => {
    stubFetch(
      json(413, {
        error: {
          code: "payload_too_large",
          message: "the file is too large: at most 8 MiB",
        },
      })
    )
    const err = (await http("/x", { method: "POST" }).catch(
      (e: unknown) => e
    )) as ApiError
    expect(err.code).toBe("payload_too_large")
    expect(err.message).toBe("the file is too large: at most 8 MiB")
  })

  it("falls back to a readable message when a proxy sends no JSON body", async () => {
    stubFetch(new Response("Request Entity Too Large", { status: 413 }))
    const big = (await http("/x", { method: "POST" }).catch(
      (e: unknown) => e
    )) as ApiError
    expect(big.message).toBe("The file is too large.")

    stubFetch(new Response("", { status: 415 }))
    const type = (await http("/x", { method: "POST" }).catch(
      (e: unknown) => e
    )) as ApiError
    expect(type.message).toBe("That file type is not supported.")
  })
})

describe("401 handling", () => {
  it("does not expire the session for a request that carried no token", async () => {
    const onExpire = vi.fn()
    const off = session.onExpire(onExpire)
    stubFetch(json(401, { error: { code: "unauthorized", message: "no" } }))
    await http("/api/v1/users/me").catch(() => {})
    expect(onExpire).not.toHaveBeenCalled()
    off()
  })

  it("does not expire a newer session because of an old request's 401", async () => {
    session.login("old")
    const onExpire = vi.fn()
    const off = session.onExpire(onExpire)
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        session.login("new") // signed in again while the request was in flight
        return json(401, { error: { code: "unauthorized", message: "no" } })
      })
    )
    await http("/api/v1/users/me").catch(() => {})
    expect(onExpire).not.toHaveBeenCalled()
    expect(session.token).toBe("new")
    off()
  })

  it("expires the session for a 401 on the current token", async () => {
    session.login("current")
    const onExpire = vi.fn()
    const off = session.onExpire(onExpire)
    stubFetch(json(401, { error: { code: "unauthorized", message: "no" } }))
    await http("/api/v1/users/me").catch(() => {})
    expect(onExpire).toHaveBeenCalledTimes(1)
    off()
  })

  it("never expires on skipExpire calls, even with a token (2FA endpoints)", async () => {
    session.login("current")
    const onExpire = vi.fn()
    const off = session.onExpire(onExpire)
    stubFetch(
      json(401, {
        error: {
          code: "unauthorized",
          message: "that code is incorrect or has expired",
        },
      })
    )
    const err = (await http("/api/v1/auth/2fa/verify", {
      method: "POST",
      body: {},
      skipExpire: true,
    }).catch((e: unknown) => e)) as ApiError
    expect(err.status).toBe(401)
    expect(onExpire).not.toHaveBeenCalled()
    expect(session.token).toBe("current")
    off()
  })
})

describe("toServerErrors: 400 and 401 placement", () => {
  const weak = new ApiError(
    400,
    "bad_request",
    "This password is too easy to guess."
  )

  it("puts a password-policy 400 under the password field", () => {
    expect(toServerErrors(weak, { badRequestField: "password" })).toMatchObject(
      {
        fields: { password: "This password is too easy to guess." },
      }
    )
  })

  it("keeps a 400 form-level when no field is configured", () => {
    expect(toServerErrors(weak)).toMatchObject({
      fields: {},
      form: "This password is too easy to guess.",
    })
  })

  it("lets a function choose the field from the message", () => {
    const pick = (m: string) =>
      /current password/.test(m) ? "current_password" : "new_password"
    expect(
      toServerErrors(
        new ApiError(400, "bad_request", "current password is incorrect"),
        { badRequestField: pick }
      ).fields
    ).toEqual({ current_password: "current password is incorrect" })
    expect(toServerErrors(weak, { badRequestField: pick }).fields).toEqual({
      new_password: "This password is too easy to guess.",
    })
  })

  it("places a wrong-credential 401 on unauthorizedField", () => {
    expect(
      toServerErrors(
        new ApiError(401, "unauthorized", "invalid email or password"),
        { unauthorizedField: "current_password" }
      ).fields
    ).toEqual({ current_password: "invalid email or password" })
  })
})
