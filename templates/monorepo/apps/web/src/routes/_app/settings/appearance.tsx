import { createFileRoute } from "@tanstack/react-router"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { useTheme } from "@/components/layout/theme"
import type { Theme } from "@/components/layout/theme"

export const Route = createFileRoute("/_app/settings/appearance")({
  component: AppearanceSettings,
})

const options: { value: Theme; label: string }[] = [
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
  { value: "system", label: "System" },
]

function AppearanceSettings() {
  const { theme, setTheme } = useTheme()
  return (
    <Card>
      <CardHeader>
        <CardTitle>Theme</CardTitle>
        <CardDescription>Stored on this device only.</CardDescription>
      </CardHeader>
      <CardContent>
        <div role="radiogroup" aria-label="Theme" className="flex gap-2">
          {options.map((o) => (
            <Button
              key={o.value}
              role="radio"
              aria-checked={theme === o.value}
              variant={theme === o.value ? "default" : "outline"}
              onClick={() => setTheme(o.value)}
            >
              {o.label}
            </Button>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
