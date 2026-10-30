import { apiUrl } from "@/lib/http"

export const AVATAR_TYPES = [
  "image/png",
  "image/jpeg",
  "image/webp",
  "image/gif",
] as const

export const AVATAR_MAX_BYTES = 2 * 1024 * 1024

export function avatarSrc(url: string | null | undefined): string | undefined {
  return url ? apiUrl(url) : undefined
}

/** Returns an error message when the file cannot be uploaded, else undefined. */
export function validateAvatarFile(file: File): string | undefined {
  if (file.size === 0) return "That file is empty."
  if (!AVATAR_TYPES.includes(file.type as (typeof AVATAR_TYPES)[number]))
    return "That file is not a supported image. Use PNG, JPEG, WebP or GIF."
  if (file.size > AVATAR_MAX_BYTES)
    return "The image is too large: at most 2 MiB."
  return undefined
}
