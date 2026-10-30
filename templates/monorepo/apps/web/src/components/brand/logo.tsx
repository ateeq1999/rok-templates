import { cn } from "@/lib/utils"

/** Three offset bars: a stack, built up layer by layer. */
export function LogoMark({ className, ...props }: React.ComponentProps<"svg">) {
  return (
    <svg
      viewBox="0 0 32 32"
      aria-hidden="true"
      className={cn("size-7 shrink-0", className)}
      {...props}
    >
      <rect width="32" height="32" className="fill-primary" />
      <rect x="7" y="8" width="18" height="4" fill="#fff" />
      <rect x="11" y="14" width="14" height="4" fill="#fff" opacity="0.8" />
      <rect x="7" y="20" width="18" height="4" fill="#fff" opacity="0.6" />
    </svg>
  )
}

export function Logo({
  className,
  showName = true,
}: {
  className?: string
  showName?: boolean
}) {
  return (
    <span className={cn("flex items-center gap-2.5", className)}>
      <LogoMark />
      {showName && (
        <span className="font-heading text-lg leading-none font-bold tracking-tight group-data-[collapsible=icon]:hidden">
          Starter
        </span>
      )}
    </span>
  )
}
