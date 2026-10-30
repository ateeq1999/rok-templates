/**
 * Mirror of the server's default upload limits, in one place. The server is
 * authoritative (MEDIA_MAX_UPLOAD_BYTES / MEDIA_ALLOWED_CONTENT_TYPES can be
 * changed by whoever deploys it) and its message is always what gets shown when
 * it rejects a file; these only save a doomed upload.
 */
export const MEDIA_MAX_BYTES = 8 * 1024 * 1024

export const MEDIA_ALLOWED_TYPES = [
  "image/jpeg",
  "image/png",
  "image/gif",
  "image/webp",
  "application/pdf",
  "video/mp4",
  "video/webm",
  "audio/mpeg",
  "audio/ogg",
] as const

export const MEDIA_MAX_FILENAME = 255

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`
}

/**
 * Pre-check before uploading. Returns a message, or undefined when the file
 * may be sent. The browser-declared type is only a hint (the server sniffs the
 * bytes), so an empty type is let through for the server to decide.
 */
export function validateMediaFile(file: {
  size: number
  type: string
}): string | undefined {
  if (file.size === 0) return "The file is empty."
  if (file.size > MEDIA_MAX_BYTES)
    return `The file is too large: at most ${MEDIA_MAX_BYTES / (1024 * 1024)} MiB.`
  if (
    file.type &&
    !(MEDIA_ALLOWED_TYPES as readonly string[]).includes(file.type)
  )
    return `Files of type ${file.type} are not accepted.`
  return undefined
}

/** The server limits `filename` to 255 characters; keep the extension when trimming. */
export function safeUploadName(name: string): string {
  if (name.length <= MEDIA_MAX_FILENAME) return name
  const dot = name.lastIndexOf(".")
  const ext = dot > 0 && name.length - dot <= 16 ? name.slice(dot) : ""
  return name.slice(0, MEDIA_MAX_FILENAME - ext.length) + ext
}

export type MediaKind = "image" | "video" | "audio" | "other"

export function mediaKind(contentType: string): MediaKind {
  if (contentType.startsWith("image/")) return "image"
  if (contentType.startsWith("video/")) return "video"
  if (contentType.startsWith("audio/")) return "audio"
  return "other"
}
