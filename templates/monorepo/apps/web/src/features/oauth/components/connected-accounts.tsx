import { useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { HugeiconsIcon } from "@hugeicons/react"
import { GithubIcon, GoogleIcon } from "@hugeicons/core-free-icons"
import { ConfirmDialog } from "@/components/common/confirm-dialog"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"
import { toast } from "@/components/ui/toast"
import { oauthApi } from "@/features/oauth/api"
import {
  identitiesQueryOptions,
  providersQueryOptions,
  useUnlinkProvider,
} from "@/features/oauth/queries"
import type {
  Identity,
  OAuthProvider,
  Provider,
} from "@/features/oauth/schemas"
import { ApiError } from "@/lib/http"

const icons: Record<OAuthProvider, typeof GoogleIcon> = {
  google: GoogleIcon,
  github: GithubIcon,
}

export function ConnectedAccounts() {
  const providers = useQuery(providersQueryOptions)
  const identities = useQuery(identitiesQueryOptions)
  const unlink = useUnlinkProvider()
  const [pendingUnlink, setPendingUnlink] = useState<OAuthProvider>()

  const known = new Map<
    OAuthProvider,
    { provider: Provider; identity?: Identity }
  >()
  for (const p of providers.data ?? []) known.set(p.provider, { provider: p })
  for (const id of identities.data ?? []) {
    const existing = known.get(id.provider)
    if (existing) existing.identity = id
    else
      known.set(id.provider, {
        provider: { provider: id.provider, name: id.provider, login_url: "" },
        identity: id,
      })
  }
  const rows = [...known.values()]

  if (providers.isLoading || identities.isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Connected accounts</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-2">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
        </CardContent>
      </Card>
    )
  }

  if (rows.length === 0) return null

  const connect = async (provider: OAuthProvider) => {
    try {
      const { authorize_url } = await oauthApi.link(provider)
      window.location.assign(authorize_url)
    } catch (error) {
      if (error instanceof ApiError)
        toast.add({ type: "error", title: error.message })
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Connected accounts</CardTitle>
        <CardDescription>
          Sign in with Google or GitHub instead of a password.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        {rows.map(({ provider, identity }) => (
          <div
            key={provider.provider}
            className="flex items-center gap-3 border px-3 py-2"
          >
            <HugeiconsIcon icon={icons[provider.provider]} className="size-5" />
            <div className="flex min-w-0 flex-1 flex-col text-sm">
              <span className="font-medium capitalize">
                {provider.name || provider.provider}
              </span>
              {identity?.email && (
                <span className="truncate text-xs text-muted-foreground">
                  {identity.email}
                </span>
              )}
            </div>
            {identity ? (
              <>
                <Badge variant="secondary">Connected</Badge>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setPendingUnlink(provider.provider)}
                >
                  Disconnect
                </Button>
              </>
            ) : (
              <Button
                variant="outline"
                size="sm"
                onClick={() => void connect(provider.provider)}
              >
                Connect
              </Button>
            )}
          </div>
        ))}
      </CardContent>
      {pendingUnlink && (
        <ConfirmDialog
          open
          onOpenChange={(open) => !open && setPendingUnlink(undefined)}
          title={`Disconnect ${pendingUnlink}?`}
          description="You will no longer be able to sign in with this provider."
          confirmLabel="Disconnect"
          pending={unlink.isPending}
          onConfirm={() =>
            unlink.mutate(pendingUnlink, {
              onSuccess: () => {
                toast.add({ type: "success", title: "Disconnected" })
                setPendingUnlink(undefined)
              },
              onError: (error) => {
                if (error instanceof ApiError)
                  toast.add({ type: "error", title: error.message })
                setPendingUnlink(undefined)
              },
            })
          }
        />
      )}
    </Card>
  )
}
