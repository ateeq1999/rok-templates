import { useEffect, useRef, useState } from "react"
import { Link, createFileRoute } from "@tanstack/react-router"
import { useMutation } from "@tanstack/react-query"
import { z } from "zod"
import { formatDistanceToNow } from "date-fns"
import { HugeiconsIcon } from "@hugeicons/react"
import { CheckmarkCircle02Icon, QrCodeIcon } from "@hugeicons/core-free-icons"
import { Button, buttonVariants } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import {
  InputOTP,
  InputOTPGroup,
  InputOTPSlot,
} from "@/components/ui/input-otp"
import { Spinner } from "@/components/ui/spinner"
import { requireMe } from "@/features/auth/session-actions"
import { qrLoginApi } from "@/features/qr-login/api"
import type { QrScan } from "@/features/qr-login/schemas"
import { describeUserAgent } from "@/lib/user-agent"
import { ApiError } from "@/lib/http"

export const Route = createFileRoute("/_app/qr-login")({
  validateSearch: z.object({
    session: z.string().optional().catch(undefined),
  }),
  beforeLoad: ({ context, location }) => requireMe(context, location),
  component: QrLoginPage,
})

type Outcome =
  | { kind: "loading" }
  | { kind: "scan"; details: QrScan }
  | { kind: "approved" }
  | { kind: "declined" }
  | { kind: "cancelled" }
  | { kind: "error"; message: string }

function QrLoginPage() {
  const { session } = Route.useSearch()

  if (!session) {
    return (
      <Empty className="min-h-[50vh]">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <HugeiconsIcon icon={QrCodeIcon} />
          </EmptyMedia>
          <EmptyTitle>Nothing to approve</EmptyTitle>
          <EmptyDescription>
            Open this page by scanning a QR code shown on another device.
          </EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
          <Link to="/profile" className={buttonVariants()}>
            Go home
          </Link>
        </EmptyContent>
      </Empty>
    )
  }

  return <ApprovePanel sessionId={session} />
}

function ApprovePanel({ sessionId }: { sessionId: string }) {
  const [outcome, setOutcome] = useState<Outcome>({ kind: "loading" })
  const [code, setCode] = useState("")
  const [codeError, setCodeError] = useState<string>()
  const started = useRef(false)

  useEffect(() => {
    if (started.current) return
    started.current = true
    qrLoginApi
      .scan(sessionId)
      .then((details) => setOutcome({ kind: "scan", details }))
      .catch((error: unknown) => {
        if (error instanceof ApiError) {
          if (error.status === 400) {
            setOutcome({
              kind: "error",
              message:
                "This QR code expired. Ask the other device for a new one.",
            })
            return
          }
          if (error.status === 409) {
            setOutcome({
              kind: "error",
              message: "This QR code was already used by another account.",
            })
            return
          }
          if (error.status === 404) {
            setOutcome({
              kind: "error",
              message: "This QR code is not valid.",
            })
            return
          }
        }
        setOutcome({ kind: "error", message: "Something went wrong." })
      })
  }, [sessionId])

  const approve = useMutation({
    mutationFn: () => qrLoginApi.approve(sessionId, code),
    onSuccess: () => setOutcome({ kind: "approved" }),
    onError: (error) => {
      if (error instanceof ApiError && error.status === 400) {
        if (error.message.includes("too many wrong codes")) {
          setOutcome({ kind: "cancelled" })
          return
        }
        setCodeError(error.message)
        return
      }
      if (error instanceof ApiError && error.status === 404) {
        setOutcome({
          kind: "error",
          message: "This request is no longer valid.",
        })
        return
      }
      setCodeError("Something went wrong. Please try again.")
    },
  })

  const reject = useMutation({
    mutationFn: () => qrLoginApi.reject(sessionId),
    onSuccess: () => setOutcome({ kind: "declined" }),
  })

  if (outcome.kind === "loading") {
    return (
      <div className="flex justify-center py-12" role="status">
        <Spinner className="size-6" />
      </div>
    )
  }

  if (outcome.kind === "approved") {
    return (
      <Card className="mx-auto max-w-sm">
        <CardHeader className="items-center text-center">
          <HugeiconsIcon
            icon={CheckmarkCircle02Icon}
            className="mb-2 size-8 text-emerald-500"
          />
          <CardTitle>Approved</CardTitle>
          <CardDescription>You can close this page.</CardDescription>
        </CardHeader>
        <CardContent className="flex justify-center">
          <Link
            to="/profile"
            className={buttonVariants({ variant: "outline" })}
          >
            Go home
          </Link>
        </CardContent>
      </Card>
    )
  }

  if (outcome.kind === "declined") {
    return (
      <Card className="mx-auto max-w-sm">
        <CardHeader className="text-center">
          <CardTitle>Sign-in declined</CardTitle>
        </CardHeader>
      </Card>
    )
  }

  if (outcome.kind === "cancelled") {
    return (
      <Card className="mx-auto max-w-sm">
        <CardHeader className="text-center">
          <CardTitle>This sign-in was cancelled</CardTitle>
          <CardDescription>Too many wrong codes were entered.</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  if (outcome.kind === "error") {
    return (
      <Card className="mx-auto max-w-sm">
        <CardHeader className="text-center">
          <CardTitle>Cannot approve this sign-in</CardTitle>
          <CardDescription>{outcome.message}</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  const { details } = outcome
  return (
    <Card className="mx-auto max-w-sm">
      <CardHeader>
        <CardTitle>Approve this sign-in?</CardTitle>
        <CardDescription>
          Someone is trying to sign in using this account. This grants full
          access to it.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <dl className="grid grid-cols-[6rem_1fr] gap-y-1 text-sm">
          <dt className="text-muted-foreground">Device</dt>
          <dd>
            {describeUserAgent(details.requester_agent) ?? "Unknown device"}
          </dd>
          <dt className="text-muted-foreground">Location</dt>
          <dd>{details.requester_ip ?? "unknown"}</dd>
          <dt className="text-muted-foreground">Requested</dt>
          <dd>
            {formatDistanceToNow(new Date(details.requested_at), {
              addSuffix: true,
            })}
          </dd>
        </dl>
        <div className="flex flex-col items-center gap-2">
          <p className="text-sm">Enter the code shown on the other device</p>
          <InputOTP
            maxLength={4}
            value={code}
            onChange={(v) => {
              setCode(v)
              setCodeError(undefined)
            }}
          >
            <InputOTPGroup>
              <InputOTPSlot index={0} />
              <InputOTPSlot index={1} />
              <InputOTPSlot index={2} />
              <InputOTPSlot index={3} />
            </InputOTPGroup>
          </InputOTP>
          {codeError && (
            <p className="text-xs text-destructive" role="alert">
              {codeError}
            </p>
          )}
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            className="flex-1"
            disabled={reject.isPending}
            onClick={() => reject.mutate()}
          >
            {reject.isPending && <Spinner />}
            Reject
          </Button>
          <Button
            className="flex-1"
            disabled={code.length !== 4 || approve.isPending}
            onClick={() => approve.mutate()}
          >
            {approve.isPending && <Spinner />}
            Approve
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
