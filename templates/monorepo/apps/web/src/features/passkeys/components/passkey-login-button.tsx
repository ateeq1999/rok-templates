import { useState } from "react"
import { useRouter } from "@tanstack/react-router"
import { useQueryClient } from "@tanstack/react-query"
import { HugeiconsIcon } from "@hugeicons/react"
import { FingerprintScanIcon } from "@hugeicons/core-free-icons"
import { Button } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"
import { toast } from "@/components/ui/toast"
import { completeSignIn } from "@/features/auth/session-actions"
import { passkeysApi } from "@/features/passkeys/api"
import {
  PasskeyError,
  getPasskey,
  passkeysSupported,
} from "@/features/passkeys/webauthn"
import { ApiError } from "@/lib/http"

/** "Sign in with a passkey" on the login page. Renders nothing unsupported. */
export function PasskeyLoginButton({ redirect }: { redirect?: string }) {
  const qc = useQueryClient()
  const router = useRouter()
  const [pending, setPending] = useState(false)

  if (!passkeysSupported()) return null

  const signIn = async () => {
    setPending(true)
    try {
      const { challenge_id, options } = await passkeysApi.loginBegin()
      const credential = await getPasskey(options)
      const token = await passkeysApi.loginFinish({
        challenge_id,
        credential,
      })
      await completeSignIn(qc, router, token.access_token, redirect)
    } catch (error) {
      if (error instanceof PasskeyError) {
        if (error.reason !== "cancelled")
          toast.add({ type: "error", title: error.message })
      } else if (error instanceof ApiError) {
        toast.add({
          type: "error",
          title:
            error.status === 401
              ? "Passkey sign-in failed. Try again or use another method."
              : error.message,
        })
      }
    } finally {
      setPending(false)
    }
  }

  return (
    <Button
      type="button"
      variant="outline"
      disabled={pending}
      onClick={() => void signIn()}
    >
      {pending ? (
        <Spinner />
      ) : (
        <HugeiconsIcon icon={FingerprintScanIcon} data-icon="inline-start" />
      )}
      Sign in with a passkey
    </Button>
  )
}
