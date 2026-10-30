import { useQuery } from "@tanstack/react-query"
import { HugeiconsIcon } from "@hugeicons/react"
import { GithubIcon, GoogleIcon } from "@hugeicons/core-free-icons"
import { Button } from "@/components/ui/button"
import { apiUrl } from "@/lib/http"
import { providersQueryOptions } from "@/features/oauth/queries"
import type { OAuthProvider } from "@/features/oauth/schemas"

const icons: Record<OAuthProvider, typeof GoogleIcon> = {
  google: GoogleIcon,
  github: GithubIcon,
}

/**
 * One outline button per enabled provider, each a real full-page link (never
 * fetch or the router Link) so the browser follows the 303 to the provider.
 */
export function SocialButtons({
  redirect,
  mode,
}: {
  redirect?: string
  mode: "signin" | "signup"
}) {
  const { data: providers } = useQuery(providersQueryOptions)

  if (!providers || providers.length === 0) return null

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-3 text-xs text-muted-foreground">
        <span className="h-px flex-1 bg-border" />
        or
        <span className="h-px flex-1 bg-border" />
      </div>
      <div className="flex flex-col gap-2">
        {providers.map((p) => (
          <Button
            key={p.provider}
            variant="outline"
            nativeButton={false}
            render={<a href={loginHref(p.login_url, redirect)} />}
          >
            <HugeiconsIcon icon={icons[p.provider]} data-icon="inline-start" />
            {mode === "signup" ? "Sign up" : "Continue"} with {p.name}
          </Button>
        ))}
      </div>
    </div>
  )
}

function loginHref(loginUrl: string, redirect: string | undefined) {
  const qs = redirect ? `?redirect=${encodeURIComponent(redirect)}` : ""
  return apiUrl(`${loginUrl}${qs}`)
}
