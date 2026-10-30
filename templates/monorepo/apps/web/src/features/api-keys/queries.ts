import {
  queryOptions,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query"
import { apiKeysApi } from "./api"

export const apiKeyKeys = {
  all: ["api-keys"] as const,
}

export const apiKeysQueryOptions = queryOptions({
  queryKey: apiKeyKeys.all,
  queryFn: apiKeysApi.list,
})

export function useCreateApiKey() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: apiKeysApi.create,
    onSuccess: () => qc.invalidateQueries({ queryKey: apiKeyKeys.all }),
  })
}

export function useRevokeApiKey() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: apiKeysApi.revoke,
    onSuccess: () => qc.invalidateQueries({ queryKey: apiKeyKeys.all }),
  })
}
