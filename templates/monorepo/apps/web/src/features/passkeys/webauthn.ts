export type PasskeyErrorReason =
  "cancelled" | "already_registered" | "unsupported" | "failed"

export class PasskeyError extends Error {
  reason: PasskeyErrorReason
  constructor(reason: PasskeyErrorReason, message?: string) {
    super(message ?? reason)
    this.name = "PasskeyError"
    this.reason = reason
  }
}

export function passkeysSupported(): boolean {
  return (
    typeof window !== "undefined" &&
    "PublicKeyCredential" in window &&
    typeof navigator !== "undefined" &&
    Boolean(navigator.credentials)
  )
}

// --- base64url <-> ArrayBuffer, for browsers without the native JSON helpers ---

function base64urlToBuffer(value: string): ArrayBuffer {
  const padded = value.replace(/-/g, "+").replace(/_/g, "/")
  const padding = "=".repeat((4 - (padded.length % 4)) % 4)
  const binary = atob(padded + padding)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
  return bytes.buffer
}

function bufferToBase64url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer)
  let binary = ""
  for (const byte of bytes) binary += String.fromCharCode(byte)
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "")
}

// Declared locally so we don't cast to `any` at every call site when the DOM
// lib in this TypeScript version lacks the newer native static helpers.
interface PublicKeyCredentialStatic {
  parseCreationOptionsFromJSON?: (
    options: unknown
  ) => PublicKeyCredentialCreationOptions
  parseRequestOptionsFromJSON?: (
    options: unknown
  ) => PublicKeyCredentialRequestOptions
}

function creationOptionsFromJson(
  publicKey: Record<string, unknown>
): PublicKeyCredentialCreationOptions {
  const native = (PublicKeyCredential as unknown as PublicKeyCredentialStatic)
    .parseCreationOptionsFromJSON
  if (native) return native(publicKey)

  const user = publicKey.user as Record<string, unknown>
  return {
    ...publicKey,
    challenge: base64urlToBuffer(publicKey.challenge as string),
    user: { ...user, id: base64urlToBuffer(user.id as string) },
    excludeCredentials: (
      (publicKey.excludeCredentials as { id: string }[] | undefined) ?? []
    ).map((c) => ({ ...c, id: base64urlToBuffer(c.id) })),
  } as PublicKeyCredentialCreationOptions
}

function requestOptionsFromJson(
  publicKey: Record<string, unknown>
): PublicKeyCredentialRequestOptions {
  const native = (PublicKeyCredential as unknown as PublicKeyCredentialStatic)
    .parseRequestOptionsFromJSON
  if (native) return native(publicKey)

  return {
    ...publicKey,
    challenge: base64urlToBuffer(publicKey.challenge as string),
    allowCredentials: (
      (publicKey.allowCredentials as { id: string }[] | undefined) ?? []
    ).map((c) => ({ ...c, id: base64urlToBuffer(c.id) })),
  } as PublicKeyCredentialRequestOptions
}

function credentialToJson(
  credential: PublicKeyCredential,
  isRegistration: boolean
) {
  if (typeof credential.toJSON === "function") return credential.toJSON()

  const response = credential.response
  const base = {
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    clientExtensionResults: credential.getClientExtensionResults(),
  }
  if (isRegistration) {
    const r = response as AuthenticatorAttestationResponse
    return {
      ...base,
      response: {
        clientDataJSON: bufferToBase64url(r.clientDataJSON),
        attestationObject: bufferToBase64url(r.attestationObject),
        transports: r.getTransports?.() ?? [],
      },
    }
  }
  const r = response as AuthenticatorAssertionResponse
  return {
    ...base,
    response: {
      clientDataJSON: bufferToBase64url(r.clientDataJSON),
      authenticatorData: bufferToBase64url(r.authenticatorData),
      signature: bufferToBase64url(r.signature),
      userHandle: r.userHandle ? bufferToBase64url(r.userHandle) : null,
    },
  }
}

function mapDomException(err: unknown): PasskeyError {
  if (err instanceof PasskeyError) return err
  if (err instanceof DOMException) {
    if (err.name === "NotAllowedError" || err.name === "AbortError")
      return new PasskeyError("cancelled", err.message)
    if (err.name === "InvalidStateError")
      return new PasskeyError(
        "already_registered",
        "This device already has a passkey for this account."
      )
    if (err.name === "SecurityError")
      return new PasskeyError(
        "failed",
        "Passkeys only work when the app is served from its expected origin."
      )
    if (err.name === "NotSupportedError")
      return new PasskeyError("unsupported", err.message)
  }
  return new PasskeyError(
    "failed",
    err instanceof Error ? err.message : "Passkey action failed"
  )
}

export async function createPasskey(options: unknown): Promise<unknown> {
  if (!passkeysSupported()) throw new PasskeyError("unsupported")
  const publicKey = (options as { publicKey: Record<string, unknown> })
    .publicKey
  try {
    const credential = await navigator.credentials.create({
      publicKey: creationOptionsFromJson(publicKey),
    })
    if (!credential) throw new PasskeyError("cancelled")
    return credentialToJson(credential as PublicKeyCredential, true)
  } catch (err) {
    throw mapDomException(err)
  }
}

export async function getPasskey(options: unknown): Promise<unknown> {
  if (!passkeysSupported()) throw new PasskeyError("unsupported")
  const publicKey = (options as { publicKey: Record<string, unknown> })
    .publicKey
  try {
    const credential = await navigator.credentials.get({
      publicKey: requestOptionsFromJson(publicKey),
    })
    if (!credential) throw new PasskeyError("cancelled")
    return credentialToJson(credential as PublicKeyCredential, false)
  } catch (err) {
    throw mapDomException(err)
  }
}

// exported for tests
export const __test__ = {
  base64urlToBuffer,
  bufferToBase64url,
  mapDomException,
}
