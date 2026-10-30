import { useRef, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { HugeiconsIcon } from "@hugeicons/react"
import { CloudUploadIcon, Files01Icon } from "@hugeicons/core-free-icons"
import { ErrorScreen } from "@/components/common/screens"
import { ConfirmDialog } from "@/components/common/confirm-dialog"
import { PageHeader } from "@/components/layout/page-header"
import { Button } from "@/components/ui/button"
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { Skeleton } from "@/components/ui/skeleton"
import { Spinner } from "@/components/ui/spinner"
import { toast } from "@/components/ui/toast"
import { MediaCard } from "@/features/media/components/media-card"
import { MEDIA_ALLOWED_TYPES, MEDIA_MAX_BYTES } from "@/features/media/limits"
import {
  mediaListQueryOptions,
  useDeleteMedia,
  useUploadMedia,
} from "@/features/media/queries"
import type { Media, MediaListParams } from "@/features/media/schemas"
import { ApiError } from "@/lib/http"
import { cn } from "@/lib/utils"

export function MediaLibrary({
  params,
  onPageChange,
}: {
  params: MediaListParams
  onPageChange: (page: number) => void
}) {
  const { data, isPending, isError, error, refetch } = useQuery(
    mediaListQueryOptions(params)
  )
  const upload = useUploadMedia()
  const remove = useDeleteMedia()
  const inputRef = useRef<HTMLInputElement>(null)
  const [dragging, setDragging] = useState(false)
  const [pendingDelete, setPendingDelete] = useState<Media>()

  const total = data?.total ?? 0
  const lastPage = Math.max(1, Math.ceil(total / params.per_page))
  const from = total === 0 ? 0 : (params.page - 1) * params.per_page + 1
  const to = Math.min(params.page * params.per_page, total)

  const send = (files: File[]) => {
    if (files.length === 0 || upload.isPending) return
    upload.mutate(files, {
      // newest first: show the result of the upload from the first page
      onSuccess: (stored) => {
        if (stored.length > 0 && params.page !== 1) onPageChange(1)
      },
    })
  }

  if (isError && !data) {
    return <ErrorScreen error={error} reset={() => void refetch()} />
  }

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        title="Media"
        description={
          data ? `${total.toLocaleString()} files` : "Your private files"
        }
        actions={
          <Button
            disabled={upload.isPending}
            onClick={() => inputRef.current?.click()}
          >
            {upload.isPending ? (
              <Spinner />
            ) : (
              <HugeiconsIcon icon={CloudUploadIcon} data-icon="inline-start" />
            )}
            {upload.isPending ? "Uploading…" : "Upload"}
          </Button>
        }
      />

      <input
        ref={inputRef}
        type="file"
        multiple
        accept={MEDIA_ALLOWED_TYPES.join(",")}
        className="sr-only"
        data-testid="media-input"
        onChange={(e) => {
          send(Array.from(e.target.files ?? []))
          e.target.value = ""
        }}
      />

      <div
        className={cn(
          "flex flex-col items-center gap-1 border border-dashed p-6 text-center text-sm text-muted-foreground transition-colors",
          dragging && "border-primary bg-primary/5 text-foreground"
        )}
        onDragOver={(e) => {
          e.preventDefault()
          setDragging(true)
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={(e) => {
          e.preventDefault()
          setDragging(false)
          send(Array.from(e.dataTransfer.files))
        }}
      >
        <span>Drag and drop files here, or use Upload.</span>
        <span className="text-xs">
          Images, PDF, MP4/WebM video, MP3/OGG audio, up to{" "}
          {MEDIA_MAX_BYTES / (1024 * 1024)} MiB each.
        </span>
      </div>

      {isPending ? (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }, (_, i) => (
            <Skeleton key={i} className="h-64 w-full rounded-none" />
          ))}
        </div>
      ) : data.items.length === 0 ? (
        <Empty className="border">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <HugeiconsIcon icon={Files01Icon} />
            </EmptyMedia>
            <EmptyTitle>No files yet</EmptyTitle>
            <EmptyDescription>
              Upload something to see it here. Only you can see your files.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {data.items.map((item) => (
            <MediaCard key={item.id} item={item} onDelete={setPendingDelete} />
          ))}
        </div>
      )}

      <div className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
        <span aria-live="polite">
          {total > 0 ? `${from}–${to} of ${total}` : ""}
        </span>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={params.page <= 1}
            onClick={() => onPageChange(params.page - 1)}
          >
            Previous
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={params.page >= lastPage}
            onClick={() => onPageChange(params.page + 1)}
          >
            Next
          </Button>
        </div>
      </div>

      {pendingDelete && (
        <ConfirmDialog
          open
          onOpenChange={(open) => !open && setPendingDelete(undefined)}
          title="Delete this file?"
          description={`${pendingDelete.original_filename ?? "This file"} will be permanently deleted.`}
          confirmLabel="Delete"
          pending={remove.isPending}
          onConfirm={() =>
            remove.mutate(pendingDelete, {
              onSuccess: () => {
                toast.add({ type: "success", title: "File deleted" })
                // deleting the last file on a later page: step back a page
                if (data && data.items.length === 1 && params.page > 1)
                  onPageChange(params.page - 1)
                setPendingDelete(undefined)
              },
              onError: (err) => {
                if (err instanceof ApiError)
                  toast.add({ type: "error", title: err.message })
                setPendingDelete(undefined)
              },
            })
          }
        />
      )}
    </div>
  )
}
