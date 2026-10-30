import { useEffect, useState } from "react"
import { checkPasswordStrength } from "@/features/auth/password-strength"
import type { Strength } from "@/features/auth/password-strength"
import { cn } from "@/lib/utils"

const BAR_COLORS = [
  "bg-destructive",
  "bg-destructive",
  "bg-amber-500",
  "bg-emerald-500",
  "bg-emerald-600",
]

/** Advisory strength bar under a new-password field. The server has the last word. */
export function PasswordStrengthMeter({
  password,
  knownInputs = [],
  weakNote,
}: {
  password: string
  knownInputs?: (string | null | undefined)[]
  /** Extra sentence shown only while the password is below the server's bar. */
  weakNote?: string
}) {
  const [strength, setStrength] = useState<Strength>()
  const inputsKey = JSON.stringify(knownInputs)

  useEffect(() => {
    if (!password) {
      setStrength(undefined)
      return
    }
    let cancelled = false
    const id = setTimeout(() => {
      checkPasswordStrength(password, JSON.parse(inputsKey))
        .then((result) => {
          if (!cancelled) setStrength(result)
        })
        .catch(() => {
          // advisory only: if the dictionaries fail to load, show nothing
          if (!cancelled) setStrength(undefined)
        })
    }, 150)
    return () => {
      cancelled = true
      clearTimeout(id)
    }
  }, [password, inputsKey])

  if (!password || !strength) return null

  return (
    <div className="flex flex-col gap-1" data-testid="password-strength">
      <div className="flex gap-1" aria-hidden>
        {[0, 1, 2, 3].map((i) => (
          <span
            key={i}
            className={cn(
              "h-1 flex-1 bg-muted",
              strength.score > i && BAR_COLORS[strength.score]
            )}
          />
        ))}
      </div>
      <p className="text-xs text-muted-foreground" aria-live="polite">
        {strength.label}
        {strength.warning ? `: ${strength.warning}` : ""}
        {!strength.acceptable && !strength.warning && strength.suggestion
          ? `: ${strength.suggestion}`
          : ""}
        {!strength.acceptable && weakNote ? ` ${weakNote}` : ""}
        <span className="sr-only"> (advisory)</span>
      </p>
    </div>
  )
}
