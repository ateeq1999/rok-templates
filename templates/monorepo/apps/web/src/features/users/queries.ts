import {
  keepPreviousData,
  queryOptions,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query"
import { statsKeys } from "@/features/stats/queries"
import { usersApi } from "./api"
import type { ListParams, User } from "./schemas"

export const userKeys = {
  all: ["users"] as const,
  me: ["users", "me"] as const,
  lists: () => ["users", "list"] as const,
  list: (p: ListParams) => ["users", "list", p] as const,
  detail: (id: string) => ["users", "detail", id] as const,
}

export const meQueryOptions = queryOptions({
  queryKey: userKeys.me,
  queryFn: usersApi.me,
})

export const usersListQueryOptions = (params: ListParams) =>
  queryOptions({
    queryKey: userKeys.list(params),
    queryFn: ({ signal }) => usersApi.list(params, signal),
    placeholderData: keepPreviousData,
  })

export const userDetailQueryOptions = (id: string) =>
  queryOptions({
    queryKey: userKeys.detail(id),
    queryFn: () => usersApi.get(id),
  })

export function useUpdateMe() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: usersApi.updateMe,
    onSuccess: (user) => qc.setQueryData(userKeys.me, user),
  })
}

export function useCreateUser() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: usersApi.create,
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: userKeys.lists() })
      void qc.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}

export function useUpdateUser(id: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (body: Parameters<typeof usersApi.update>[1]) =>
      usersApi.update(id, body),
    onSuccess: (user: User) => {
      qc.setQueryData(userKeys.detail(id), user)
      const me = qc.getQueryData<User>(userKeys.me)
      if (me?.id === id) qc.setQueryData(userKeys.me, user)
      void qc.invalidateQueries({ queryKey: userKeys.lists() })
      void qc.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}

function applyMeUpdate(qc: ReturnType<typeof useQueryClient>, user: User) {
  qc.setQueryData(userKeys.me, user)
  qc.setQueryData(userKeys.detail(user.id), user)
  void qc.invalidateQueries({ queryKey: userKeys.lists() })
}

export function useAvatarMutations() {
  const qc = useQueryClient()
  const upload = useMutation({
    mutationFn: usersApi.uploadAvatar,
    onSuccess: (user) => applyMeUpdate(qc, user),
  })
  const remove = useMutation({
    mutationFn: usersApi.removeAvatar,
    onSuccess: (user) => applyMeUpdate(qc, user),
  })
  return { upload, remove }
}

export function useDeleteUser() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: usersApi.remove,
    onSuccess: (_data, id) => {
      qc.removeQueries({ queryKey: userKeys.detail(id) })
      void qc.invalidateQueries({ queryKey: userKeys.lists() })
      void qc.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}
