import {
  queryOptions,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query"
import { oauthApi } from "./api"

export const oauthKeys = {
  providers: ["oauth", "providers"] as const,
  identities: ["oauth", "identities"] as const,
}

export const providersQueryOptions = queryOptions({
  queryKey: oauthKeys.providers,
  queryFn: oauthApi.providers,
  staleTime: 10 * 60 * 1000,
  retry: false,
})

export const identitiesQueryOptions = queryOptions({
  queryKey: oauthKeys.identities,
  queryFn: oauthApi.identities,
})

export function useUnlinkProvider() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: oauthApi.unlink,
    onSuccess: () => qc.invalidateQueries({ queryKey: oauthKeys.identities }),
  })
}
