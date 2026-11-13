import { useQuery } from "@tanstack/react-query"
import { formatDistanceToNow } from "date-fns"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Spinner } from "@/components/ui/spinner"
import { toast } from "@/components/ui/toast"
import { ApiError } from "@/lib/client"
import {
  identitiesQueryOptions,
  signInProvidersQueryOptions,
  useLinkIdentity,
  useUnlinkIdentity,
} from "../queries"
import { providerLabel } from "../provider-label"

/**
 * The account's Linked identities: each with Unlink, and Link for each
 * configured provider not linked yet. Hidden while no provider is
 * configured. The API refuses to unlink the account's last way in, and says
 * why.
 */
export function LinkedIdentitiesCard() {
  const { data: providers = [] } = useQuery(signInProvidersQueryOptions)
  if (providers.length === 0) return null
  return <LinkedIdentities providers={providers.map(({ name }) => name)} />
}

function LinkedIdentities({ providers }: { providers: string[] }) {
  const { data: identities, isPending } = useQuery(identitiesQueryOptions)
  const link = useLinkIdentity()
  const unlink = useUnlinkIdentity()
  const linked = new Set(identities?.map((identity) => identity.provider))
  const showError = (error: unknown) => {
    if (error instanceof ApiError)
      toast.add({ type: "error", title: error.message })
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Linked accounts</CardTitle>
        <CardDescription>
          Sign in with another account instead of your password.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {isPending && <Spinner />}
        {identities?.map((identity) => (
          <div
            key={identity.id}
            className="flex items-center justify-between gap-3 text-sm"
          >
            <div className="flex flex-col">
              <span className="font-medium">
                {providerLabel(identity.provider)}
              </span>
              <span className="text-xs text-muted-foreground">
                {identity.email ?? "No email"} · linked{" "}
                {formatDistanceToNow(new Date(identity.created_at), {
                  addSuffix: true,
                })}
              </span>
            </div>
            <Button
              variant="outline"
              size="sm"
              disabled={unlink.isPending}
              onClick={() =>
                unlink.mutate(identity.id, {
                  onSuccess: () =>
                    toast.add({
                      type: "success",
                      title: `${providerLabel(identity.provider)} is unlinked`,
                    }),
                  onError: showError,
                })
              }
            >
              Unlink
            </Button>
          </div>
        ))}
        <div className="flex flex-wrap gap-2">
          {providers
            .filter((provider) => !linked.has(provider))
            .map((provider) => (
              <Button
                key={provider}
                variant="outline"
                disabled={link.isPending}
                onClick={() => link.mutate(provider, { onError: showError })}
              >
                {link.isPending && link.variables === provider && <Spinner />}
                Link {providerLabel(provider)}
              </Button>
            ))}
        </div>
      </CardContent>
    </Card>
  )
}
