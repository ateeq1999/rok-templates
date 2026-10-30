import { useEffect, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { mediaApi } from "./api"
import { mediaKeys } from "./queries"
import type { Media } from "./schemas"

/**
 * Loads a file's bytes with the Authorization header and exposes them as an
 * object URL for <img>/<video>/<audio>. The URL is revoked whenever the blob
 * changes and on unmount, so nothing leaks.
 */
export function useBlobUrl(item: Pick<Media, "url">, enabled = true) {
  const query = useQuery({
    queryKey: mediaKeys.blob(item.url),
    queryFn: ({ signal }) => mediaApi.content(item, signal),
    enabled,
    staleTime: Infinity,
    gcTime: 60_000,
  })
  const [objectUrl, setObjectUrl] = useState<string>()

  useEffect(() => {
    if (!query.data) {
      setObjectUrl(undefined)
      return
    }
    const url = URL.createObjectURL(query.data)
    setObjectUrl(url)
    return () => URL.revokeObjectURL(url)
  }, [query.data])

  return {
    url: objectUrl,
    isLoading: enabled && (query.isLoading || (!!query.data && !objectUrl)),
    isError: query.isError,
  }
}
