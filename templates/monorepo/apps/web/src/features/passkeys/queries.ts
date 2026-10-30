import {
  queryOptions,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query"
import { passkeysApi } from "./api"

export const passkeyKeys = {
  all: ["passkeys"] as const,
}

export const passkeysQueryOptions = queryOptions({
  queryKey: passkeyKeys.all,
  queryFn: passkeysApi.list,
})

export function useRemovePasskey() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: passkeysApi.remove,
    onSuccess: () => qc.invalidateQueries({ queryKey: passkeyKeys.all }),
  })
}
