import { useMutation, useQueryClient } from "@tanstack/react-query"
import { api, keys, mutations, queries } from "@/lib/client"
import { expiryDays } from "./schemas"
import type { ExpiryOption, Scope } from "./schemas"

export const apiKeyKeys = {
  all: keys.apiKeys,
}

export const apiKeysQueryOptions = queries.apiKeys.list()

export function useCreateApiKey() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (body: { name: string; scope: Scope; expiry: ExpiryOption }) =>
      api.apiKeys.create({
        name: body.name,
        scope: body.scope,
        expires_in_days: expiryDays(body.expiry),
      }),
    onSuccess: () => qc.invalidateQueries({ queryKey: apiKeyKeys.all }),
  })
}

export function useRevokeApiKey() {
  const qc = useQueryClient()
  return useMutation({
    ...mutations.apiKeys.revoke(),
    onSuccess: () => qc.invalidateQueries({ queryKey: apiKeyKeys.all }),
  })
}
