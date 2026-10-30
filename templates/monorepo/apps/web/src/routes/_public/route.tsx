import { Outlet, createFileRoute } from "@tanstack/react-router"
import { Logo } from "@/components/brand/logo"

export const Route = createFileRoute("/_public")({
  component: PublicLayout,
})

const points = [
  "Sign in once, manage your account anywhere.",
  "Verified email, secure password reset.",
  "Admin tools for users, roles and status.",
]

function PublicLayout() {
  return (
    <div className="grid min-h-svh lg:grid-cols-[minmax(0,5fr)_minmax(0,6fr)]">
      <aside className="relative hidden flex-col justify-between overflow-hidden bg-foreground p-10 text-background lg:flex">
        <div
          aria-hidden
          className="absolute inset-0 [background-image:linear-gradient(currentColor_1px,transparent_1px),linear-gradient(90deg,currentColor_1px,transparent_1px)] [background-size:32px_32px] opacity-[0.07]"
        />
        <Logo className="relative [&_span]:text-background" />
        <div className="relative flex flex-col gap-6">
          <h2 className="font-heading text-4xl leading-[1.05] font-semibold tracking-tight">
            Stack it up.
            <br />
            Ship it sharp.
          </h2>
          <ul className="flex flex-col gap-2 text-sm text-background/70">
            {points.map((p) => (
              <li key={p} className="flex items-center gap-2">
                <span className="size-1.5 bg-primary" aria-hidden />
                {p}
              </li>
            ))}
          </ul>
        </div>
        <p className="relative text-xs text-background/50">
          © {new Date().getFullYear()} Starter
        </p>
      </aside>
      <main className="flex flex-col items-center justify-center gap-6 bg-muted/40 p-4">
        <Logo className="lg:hidden" />
        <Outlet />
      </main>
    </div>
  )
}
