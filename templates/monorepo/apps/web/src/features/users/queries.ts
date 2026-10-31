import {
  keepPreviousData,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query"
import { statsKeys } from "@/features/stats/queries"
import { api, keys, mutations, queries } from "@/lib/client"
import type { UpdateUserDto } from "@/lib/client"
import type { ListParams, User } from "./schemas"

export const userKeys = {
  all: keys.users,
  me: queries.users.me().queryKey,
  lists: () => [...keys.users, "list"] as const,
  detail: (id: string) => queries.users.getOne(id).queryKey,
}

export const meQueryOptions = queries.users.me()

export const usersListQueryOptions = (params: ListParams) => ({
  ...queries.users.list(params),
  placeholderData: keepPreviousData,
})

export const userDetailQueryOptions = (id: string) => queries.users.getOne(id)

export function useUpdateMe() {
  const qc = useQueryClient()
  return useMutation({
    ...mutations.users.updateMe(),
    onSuccess: (user) => qc.setQueryData(userKeys.me, user),
  })
}

export function useUpdateUser(id: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (body: UpdateUserDto) => api.users.update(id, body),
    onSuccess: (user: User) => {
      qc.setQueryData(userKeys.detail(id), user)
      const me = qc.getQueryData(userKeys.me)
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
    ...mutations.avatars.upload(),
    onSuccess: (user) => applyMeUpdate(qc, user),
  })
  const remove = useMutation({
    ...mutations.avatars.remove(),
    onSuccess: (user) => applyMeUpdate(qc, user),
  })
  return { upload, remove }
}

export function useDeleteUser() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => api.users.remove(id),
    onSuccess: (_data, id) => {
      qc.removeQueries({ queryKey: userKeys.detail(id) })
      void qc.invalidateQueries({ queryKey: userKeys.lists() })
      void qc.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}
