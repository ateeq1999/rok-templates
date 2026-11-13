import { useQuery } from "@tanstack/react-query"
import { buttonVariants } from "@/components/ui/button"
import { signInProvidersQueryOptions } from "../queries"
import { providerLabel } from "../provider-label"
import { signInStartUrl } from "../start-url"

/**
 * "Continue with <provider>" for each provider the API is configured with,
 * each a link that starts its sign-in. Nothing at all while none is
 * configured (or the list hasn't loaded).
 */
export function ContinueWithButtons() {
  const { data: providers = [] } = useQuery(signInProvidersQueryOptions)
  if (providers.length === 0) return null

  return (
    <div className="flex flex-col gap-2">
      {providers.map(({ name }) => (
        // A link, not a button: it leaves the App for the provider.
        <a
          key={name}
          href={signInStartUrl(name)}
          className={buttonVariants({ variant: "outline" })}
        >
          Continue with {providerLabel(name)}
        </a>
      ))}
    </div>
  )
}
