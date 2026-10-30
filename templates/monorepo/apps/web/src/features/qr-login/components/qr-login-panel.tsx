import { useEffect, useState } from "react"
import { useRouter } from "@tanstack/react-router"
import { useQueryClient } from "@tanstack/react-query"
import { HugeiconsIcon } from "@hugeicons/react"
import { QrCodeIcon } from "@hugeicons/core-free-icons"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Spinner } from "@/components/ui/spinner"
import { completeSignIn } from "@/features/auth/session-actions"
import { useQrLogin } from "@/features/qr-login/use-qr-login"

function qrImageSrc(svg: string) {
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`
}

function Countdown({ expiresAt }: { expiresAt: string }) {
  const [secondsLeft, setSecondsLeft] = useState(() =>
    Math.max(0, Math.round((new Date(expiresAt).getTime() - Date.now()) / 1000))
  )
  useEffect(() => {
    const id = setInterval(() => {
      setSecondsLeft(
        Math.max(
          0,
          Math.round((new Date(expiresAt).getTime() - Date.now()) / 1000)
        )
      )
    }, 1000)
    return () => clearInterval(id)
  }, [expiresAt])
  const m = Math.floor(secondsLeft / 60)
  const s = secondsLeft % 60
  return (
    <span className="tabular-nums">
      {m}:{s.toString().padStart(2, "0")}
    </span>
  )
}

function QrLoginBody({
  redirect,
  onDone,
}: {
  redirect?: string
  onDone: () => void
}) {
  const qc = useQueryClient()
  const router = useRouter()

  const { state, start } = useQrLogin((token) => {
    void completeSignIn(qc, router, token, redirect).then(onDone)
  })

  useEffect(() => {
    void start()
    // one session per time the dialog opens
  }, [])

  if (state.phase === "idle" || state.phase === "creating") {
    return (
      <div className="flex flex-col items-center gap-3 py-8" role="status">
        <Spinner className="size-6" />
        <p className="text-sm text-muted-foreground">Preparing your code…</p>
      </div>
    )
  }

  if (state.phase === "error") {
    return (
      <div className="flex flex-col items-center gap-4 py-6">
        <p className="text-center text-sm text-destructive" role="alert">
          {state.errorMessage}
        </p>
        <Button variant="outline" onClick={() => void start()}>
          Try again
        </Button>
      </div>
    )
  }

  if (state.phase === "rejected") {
    return (
      <div className="flex flex-col items-center gap-4 py-6">
        <p className="text-sm" role="status">
          The sign-in was declined on your phone.
        </p>
        <Button variant="outline" onClick={() => void start()}>
          Try again
        </Button>
      </div>
    )
  }

  if (state.phase === "expired" || state.phase === "consumed") {
    return (
      <div className="flex flex-col items-center gap-4 py-6">
        <p className="text-sm" role="status">
          {state.phase === "expired"
            ? "This QR code expired."
            : "This code was already used."}
        </p>
        <Button variant="outline" onClick={() => void start()}>
          New QR code
        </Button>
      </div>
    )
  }

  if (state.phase === "approved") {
    return (
      <div className="flex flex-col items-center gap-3 py-8" role="status">
        <Spinner className="size-6" />
        <p className="text-sm text-muted-foreground">Signing you in…</p>
      </div>
    )
  }

  const session = state.session
  if (!session) return null

  return (
    <div className="flex flex-col items-center gap-4 py-2">
      <img
        alt="QR code to sign in"
        src={qrImageSrc(session.qr_svg)}
        className="size-48 border p-2"
      />
      <div className="flex flex-col items-center gap-1">
        <span className="font-heading text-3xl font-semibold tracking-[0.3em]">
          {session.verification_code}
        </span>
        <p
          className="text-center text-sm text-muted-foreground"
          aria-live="polite"
        >
          {state.phase === "scanned"
            ? "Confirm on your phone"
            : "On your phone, scan this code and enter the number above"}
        </p>
      </div>
      <p className="text-xs text-muted-foreground">
        Expires in <Countdown expiresAt={session.expires_at} />
      </p>
    </div>
  )
}

/** "Sign in with QR code" on the login page, opening a compact dialog. */
export function QrLoginDialog({ redirect }: { redirect?: string }) {
  const [open, setOpen] = useState(false)
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <Button type="button" variant="outline" onClick={() => setOpen(true)}>
        <HugeiconsIcon icon={QrCodeIcon} data-icon="inline-start" />
        Sign in with QR code
      </Button>
      <DialogContent className="sm:max-w-xs">
        <DialogHeader>
          <DialogTitle>Scan to sign in</DialogTitle>
          <DialogDescription>
            Use an already signed-in device to approve this sign-in.
          </DialogDescription>
        </DialogHeader>
        {open && (
          <QrLoginBody redirect={redirect} onDone={() => setOpen(false)} />
        )}
      </DialogContent>
    </Dialog>
  )
}
