import { Link, useRouter } from "@tanstack/react-router"
import { HugeiconsIcon } from "@hugeicons/react"
import {
  Alert02Icon,
  LockIcon,
  SearchRemoveIcon,
} from "@hugeicons/core-free-icons"
import { Button, buttonVariants } from "@/components/ui/button"
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { ApiError } from "@/lib/http"

export function ErrorScreen({
  error,
  reset,
}: {
  error: unknown
  reset?: () => void
}) {
  const router = useRouter()
  const forbidden = error instanceof ApiError && error.status === 403
  const missing = error instanceof ApiError && error.status === 404
  if (missing) return <NotFoundScreen />
  return (
    <Empty className="min-h-[60vh]">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <HugeiconsIcon icon={forbidden ? LockIcon : Alert02Icon} />
        </EmptyMedia>
        <EmptyTitle>
          {forbidden ? "Access denied" : "Something went wrong"}
        </EmptyTitle>
        <EmptyDescription>
          {error instanceof Error ? error.message : "Unexpected error"}
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <div className="flex gap-2">
          {!forbidden && (
            <Button
              onClick={() => {
                reset?.()
                void router.invalidate()
              }}
            >
              Try again
            </Button>
          )}
          <Link to="/" className={buttonVariants({ variant: "outline" })}>
            Go home
          </Link>
        </div>
      </EmptyContent>
    </Empty>
  )
}

export function NotFoundScreen() {
  return (
    <Empty className="min-h-[60vh]">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <HugeiconsIcon icon={SearchRemoveIcon} />
        </EmptyMedia>
        <EmptyTitle>Not found</EmptyTitle>
        <EmptyDescription>
          The page or record you are looking for does not exist.
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <Link to="/" className={buttonVariants()}>
          Go home
        </Link>
      </EmptyContent>
    </Empty>
  )
}
