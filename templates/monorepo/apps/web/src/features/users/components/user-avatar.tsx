import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { avatarSrc } from "@/features/users/avatar"
import { initials } from "@/features/users/schemas"
import type { User } from "@/features/users/schemas"

export function UserAvatar({
  user,
  size = "default",
  className,
}: {
  user: Pick<User, "display_name" | "email" | "avatar_url">
  size?: "default" | "sm" | "lg"
  className?: string
}) {
  return (
    <Avatar size={size} className={className}>
      {user.avatar_url && (
        <AvatarImage src={avatarSrc(user.avatar_url)} alt="" />
      )}
      <AvatarFallback>{initials(user)}</AvatarFallback>
    </Avatar>
  )
}
