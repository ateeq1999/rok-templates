import { useState } from "react"
import { useMutation } from "@tanstack/react-query"
import { format } from "date-fns"
import { HugeiconsIcon } from "@hugeicons/react"
import {
  Delete02Icon,
  Download01Icon,
  File01Icon,
  MusicNote01Icon,
  Pdf01Icon,
  PlayIcon,
  Video01Icon,
} from "@hugeicons/core-free-icons"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"
import { Spinner } from "@/components/ui/spinner"
import { toast } from "@/components/ui/toast"
import { mediaApi } from "@/features/media/api"
import { formatBytes, mediaKind } from "@/features/media/limits"
import { downloadName } from "@/features/media/schemas"
import type { Media } from "@/features/media/schemas"
import { useBlobUrl } from "@/features/media/use-blob-url"
import { downloadBlob } from "@/lib/clipboard"
import { ApiError } from "@/lib/http"

const PREVIEW_BOX = "flex h-40 items-center justify-center bg-muted"

/**
 * Images preview by themselves. Video and audio can be large, so their bytes
 * are only fetched once the user asks. Everything else (PDF, ...) gets a
 * Download button that fetches the blob on demand.
 */
function Preview({ item }: { item: Media }) {
  const kind = mediaKind(item.content_type)
  const [requested, setRequested] = useState(false)
  const blob = useBlobUrl(item, kind === "image" || requested)
  const label = downloadName(item)

  if (kind === "image") {
    if (blob.isError)
      return (
        <div className={PREVIEW_BOX}>
          <span className="text-xs text-muted-foreground">
            Could not load the preview.
          </span>
        </div>
      )
    if (!blob.url) return <Skeleton className="h-40 w-full rounded-none" />
    return (
      <img
        src={blob.url}
        alt={label}
        className="h-40 w-full bg-muted object-contain"
      />
    )
  }

  if (kind === "video" || kind === "audio") {
    if (blob.url) {
      return kind === "video" ? (
        <video src={blob.url} controls className="h-40 w-full bg-black" />
      ) : (
        <div className={PREVIEW_BOX}>
          <audio src={blob.url} controls className="w-11/12" />
        </div>
      )
    }
    return (
      <div className={PREVIEW_BOX}>
        <Button
          variant="outline"
          size="sm"
          disabled={blob.isLoading}
          onClick={() => setRequested(true)}
        >
          {blob.isLoading ? (
            <Spinner />
          ) : (
            <HugeiconsIcon icon={PlayIcon} data-icon="inline-start" />
          )}
          {kind === "video" ? "Load video" : "Load audio"}
        </Button>
        {blob.isError && (
          <span className="ml-2 text-xs text-destructive">Failed to load.</span>
        )}
      </div>
    )
  }

  return (
    <div className={PREVIEW_BOX}>
      <HugeiconsIcon
        icon={item.content_type === "application/pdf" ? Pdf01Icon : File01Icon}
        className="size-10 text-muted-foreground"
      />
    </div>
  )
}

export function MediaCard({
  item,
  onDelete,
}: {
  item: Media
  onDelete: (item: Media) => void
}) {
  const name = downloadName(item)
  const kind = mediaKind(item.content_type)

  const download = useMutation({
    mutationFn: async () => {
      const blob = await mediaApi.content(item)
      downloadBlob(blob, name)
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Download failed",
        description: error instanceof ApiError ? error.message : undefined,
      })
    },
  })

  return (
    <Card size="sm" className="overflow-hidden pt-0">
      <Preview item={item} />
      <CardContent className="flex flex-col gap-2 pt-3">
        <div className="min-w-0">
          <p className="truncate text-sm font-medium" title={name}>
            {name}
          </p>
          <p className="flex items-center gap-1 text-xs text-muted-foreground">
            {kind === "video" && (
              <HugeiconsIcon icon={Video01Icon} className="size-3" />
            )}
            {kind === "audio" && (
              <HugeiconsIcon icon={MusicNote01Icon} className="size-3" />
            )}
            {item.content_type} · {formatBytes(item.size_bytes)} ·{" "}
            {format(new Date(item.created_at), "PP")}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={download.isPending}
            onClick={() => download.mutate()}
          >
            {download.isPending ? (
              <Spinner />
            ) : (
              <HugeiconsIcon icon={Download01Icon} data-icon="inline-start" />
            )}
            Download
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            className="ml-auto"
            aria-label={`Delete ${name}`}
            onClick={() => onDelete(item)}
          >
            <HugeiconsIcon icon={Delete02Icon} />
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
