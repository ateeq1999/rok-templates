import { useMutation, useQueryClient } from "@tanstack/react-query"
import { api, mutations, queries } from "@/lib/client"

/** The providers people may sign in with: none, and the Apps hide the buttons. */
export const signInProvidersQueryOptions = {
  ...queries.auth.listSignInProviders(),
  // The configured providers change only when the API restarts.
  staleTime: 5 * 60 * 1000,
}

/** The signed-in account's Linked identities. */
export const identitiesQueryOptions = queries.auth.listIdentities()

/**
 * Links an identity at a provider: the API answers the provider's URL, and
 * the browser goes there. The provider sends it back to `/auth/oidc`.
 */
export function useLinkIdentity() {
  return useMutation({
    mutationFn: (provider: string) => api.auth.linkIdentity(provider),
    onSuccess: ({ url }) => window.location.assign(url),
  })
}

/** Unlinks one of the account's identities; the API keeps its last way in. */
export function useUnlinkIdentity() {
  const qc = useQueryClient()
  return useMutation({
    ...mutations.auth.unlinkIdentity(),
    onSuccess: () =>
      qc.invalidateQueries({ queryKey: identitiesQueryOptions.queryKey }),
  })
}
