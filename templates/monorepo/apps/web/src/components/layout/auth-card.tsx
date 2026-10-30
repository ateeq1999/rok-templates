import type { ReactNode } from "react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

export function AuthCard({
  title,
  description,
  children,
  footer,
}: {
  title: string
  description?: ReactNode
  children?: ReactNode
  footer?: ReactNode
}) {
  return (
    <Card className="w-full max-w-sm shadow-sm">
      <CardHeader>
        <CardTitle className="font-heading text-xl">{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      {children && (
        <CardContent className="flex flex-col gap-4">{children}</CardContent>
      )}
      {footer && (
        <CardContent className="text-center text-xs text-muted-foreground">
          {footer}
        </CardContent>
      )}
    </Card>
  )
}
