import { Link } from "@tanstack/react-router"
import { useSelector } from "@tanstack/react-store"
import { HugeiconsIcon } from "@hugeicons/react"
import {
  DashboardSquare01Icon,
  LockIcon,
  Settings01Icon,
  UserGroupIcon,
  UserIcon,
  ViewIcon,
} from "@hugeicons/core-free-icons"
import { Logo } from "@/components/brand/logo"
import { buttonVariants } from "@/components/ui/button"
import { session } from "@/lib/session"
import { cn } from "@/lib/utils"

const features = [
  {
    icon: LockIcon,
    title: "Complete auth flows",
    body: "Sign in, register, forgot and reset password, email verification and email-change confirmation. One-time links are redeemed exactly once.",
  },
  {
    icon: UserIcon,
    title: "Profile and settings",
    body: "Edit your display name, change email or password with re-authentication, and pick light, dark or system theme.",
  },
  {
    icon: UserGroupIcon,
    title: "User management",
    body: "Search, sort and page through users. Create, invite, edit roles, deactivate or delete, with guards against locking yourself out.",
  },
  {
    icon: DashboardSquare01Icon,
    title: "Admin dashboard",
    body: "Totals, admins, active and unverified counts, a 30-day signups chart, recent signups and live system status.",
  },
  {
    icon: ViewIcon,
    title: "URL is the state",
    body: "Page, search and sort live in the address bar, so reloads, back and forward and shared links all work.",
  },
  {
    icon: Settings01Icon,
    title: "Honest sessions",
    body: "A short-lived bearer token, an expiry warning, cross-tab sign-out and a cleared cache on logout.",
  },
]

const stack = [
  ["TanStack Start", "SPA mode, static build"],
  ["TanStack Router", "Typed file routes and guards"],
  ["TanStack Query", "Cache, mutations, invalidation"],
  ["TanStack Form", "Zod schemas, server errors"],
  ["TanStack Table", "Server-side paging and sorting"],
  ["shadcn/ui", "Accessible, owned components"],
]

const stats = [
  ["20", "API endpoints wired"],
  ["6", "auth pages"],
  ["4", "states per screen"],
  ["0", "servers to run"],
]

const security = [
  "Strict role checks: the server decides, the UI only hides",
  "Tokens stripped from the URL the moment a page loads",
  "Forgot password never reveals whether an address exists",
  "Redirect targets limited to same-origin paths",
]

function Container({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      className={cn("mx-auto w-full max-w-6xl px-4 md:px-6", className)}
      {...props}
    />
  )
}

function Header() {
  const signedIn = useSelector(session.store, (s) => Boolean(s.token))
  return (
    <header className="sticky top-0 z-20 border-b bg-background/85 backdrop-blur">
      <Container className="flex h-14 items-center justify-between">
        <Logo />
        <nav aria-label="Sections" className="hidden gap-6 text-sm md:flex">
          {[
            ["Features", "#features"],
            ["Stack", "#stack"],
            ["Security", "#security"],
          ].map(([label, href]) => (
            <a
              key={href}
              href={href}
              className="text-muted-foreground hover:text-foreground"
            >
              {label}
            </a>
          ))}
        </nav>
        <div className="flex items-center gap-2">
          {signedIn ? (
            <Link to="/profile" className={buttonVariants()}>
              Open app
            </Link>
          ) : (
            <>
              <Link
                to="/login"
                className={buttonVariants({ variant: "ghost" })}
              >
                Sign in
              </Link>
              <Link to="/register" className={buttonVariants()}>
                Get started
              </Link>
            </>
          )}
        </div>
      </Container>
    </header>
  )
}

/** A static sketch of the admin dashboard, built from the same tokens. */
function DashboardPreview() {
  const bars = [3, 5, 2, 6, 4, 8, 5, 7, 3, 9, 6, 8, 10, 7]
  return (
    <div aria-hidden className="border bg-card text-card-foreground shadow-xl">
      <div className="flex items-center gap-1.5 border-b px-3 py-2">
        <span className="size-2 bg-muted-foreground/30" />
        <span className="size-2 bg-muted-foreground/30" />
        <span className="size-2 bg-muted-foreground/30" />
        <span className="ml-3 text-[10px] text-muted-foreground">
          Dashboard
        </span>
      </div>
      <div className="grid grid-cols-3 gap-2 p-3">
        {[
          ["Total users", "128"],
          ["Active", "117"],
          ["Unverified", "9"],
        ].map(([k, v]) => (
          <div key={k} className="border p-2.5">
            <div className="text-[10px] text-muted-foreground">{k}</div>
            <div className="font-heading text-xl font-semibold">{v}</div>
          </div>
        ))}
      </div>
      <div className="mx-3 mb-3 border p-3">
        <div className="mb-2 text-[10px] text-muted-foreground">
          Signups, last 14 days
        </div>
        <div className="flex h-24 items-end gap-1.5">
          {bars.map((h, i) => (
            <div
              key={i}
              className="flex-1 bg-primary"
              style={{ height: `${h * 10}%`, opacity: 0.45 + h / 20 }}
            />
          ))}
        </div>
      </div>
    </div>
  )
}

function Hero() {
  return (
    <section className="relative overflow-hidden border-b">
      <div
        aria-hidden
        className="absolute inset-0 [background-image:linear-gradient(currentColor_1px,transparent_1px),linear-gradient(90deg,currentColor_1px,transparent_1px)] [background-size:32px_32px] opacity-[0.05]"
      />
      <Container className="relative grid items-center gap-12 py-16 md:py-24 lg:grid-cols-2">
        <div className="flex flex-col items-start gap-6">
          <span className="border px-2 py-1 text-xs text-muted-foreground">
            React · TanStack · shadcn/ui · Rust API
          </span>
          <h1 className="font-heading text-5xl leading-[1.02] font-semibold tracking-tight md:text-6xl">
            Stack it up.
            <br />
            <span className="text-primary">Ship it sharp.</span>
          </h1>
          <p className="max-w-lg text-base text-muted-foreground">
            A production-minded frontend starter for your Rust API: every auth
            flow, a profile and settings area, and an admin console with stats
            and user management, ready to run.
          </p>
          <div className="flex flex-wrap gap-2">
            <Link to="/register" className={buttonVariants({ size: "lg" })}>
              Create an account
            </Link>
            <Link
              to="/login"
              className={buttonVariants({ size: "lg", variant: "outline" })}
            >
              Sign in
            </Link>
          </div>
        </div>
        <DashboardPreview />
      </Container>
    </section>
  )
}

function Stats() {
  return (
    <section className="border-b">
      <Container className="grid grid-cols-2 divide-x md:grid-cols-4">
        {stats.map(([n, label]) => (
          <div key={label} className="flex flex-col gap-1 px-4 py-8 md:px-6">
            <span className="font-heading text-4xl font-semibold">{n}</span>
            <span className="text-sm text-muted-foreground">{label}</span>
          </div>
        ))}
      </Container>
    </section>
  )
}

function Features() {
  return (
    <section id="features" className="scroll-mt-14 border-b py-16 md:py-24">
      <Container className="flex flex-col gap-10">
        <div className="max-w-xl">
          <h2 className="font-heading text-3xl font-semibold tracking-tight">
            Everything the API offers, on a screen
          </h2>
          <p className="mt-2 text-muted-foreground">
            Each endpoint has a home, and each screen has loading, empty, error
            and success states.
          </p>
        </div>
        <div className="grid gap-px border bg-border sm:grid-cols-2 lg:grid-cols-3">
          {features.map((f) => (
            <div
              key={f.title}
              className="flex flex-col gap-3 bg-background p-6"
            >
              <span className="flex size-9 items-center justify-center bg-primary text-primary-foreground">
                <HugeiconsIcon icon={f.icon} className="size-5" />
              </span>
              <h3 className="font-heading text-lg font-semibold">{f.title}</h3>
              <p className="text-sm text-muted-foreground">{f.body}</p>
            </div>
          ))}
        </div>
      </Container>
    </section>
  )
}

function Stack() {
  return (
    <section
      id="stack"
      className="scroll-mt-14 border-b bg-muted/40 py-16 md:py-24"
    >
      <Container className="grid gap-10 lg:grid-cols-[2fr_3fr]">
        <div>
          <h2 className="font-heading text-3xl font-semibold tracking-tight">
            Built on the TanStack stack
          </h2>
          <p className="mt-2 text-muted-foreground">
            A static single-page app: no Node server to run, deploy it behind
            the same proxy as the API.
          </p>
        </div>
        <ul className="grid gap-px border bg-border sm:grid-cols-2">
          {stack.map(([name, note]) => (
            <li key={name} className="bg-background p-4">
              <div className="font-heading font-semibold">{name}</div>
              <div className="text-sm text-muted-foreground">{note}</div>
            </li>
          ))}
        </ul>
      </Container>
    </section>
  )
}

function Security() {
  return (
    <section id="security" className="scroll-mt-14 border-b py-16 md:py-24">
      <Container className="grid gap-10 lg:grid-cols-2">
        <div>
          <h2 className="font-heading text-3xl font-semibold tracking-tight">
            Careful with the sharp edges
          </h2>
          <p className="mt-2 text-muted-foreground">
            Small decisions that keep accounts and links safe.
          </p>
        </div>
        <ul className="flex flex-col gap-3">
          {security.map((s) => (
            <li key={s} className="flex items-start gap-3 text-sm">
              <span className="mt-1.5 size-2 shrink-0 bg-primary" aria-hidden />
              {s}
            </li>
          ))}
        </ul>
      </Container>
    </section>
  )
}

function Cta() {
  return (
    <section className="bg-foreground text-background">
      <Container className="flex flex-col items-start justify-between gap-6 py-14 md:flex-row md:items-center">
        <h2 className="font-heading text-3xl font-semibold tracking-tight">
          Ready to try it?
        </h2>
        <div className="flex gap-2">
          <Link to="/register" className={buttonVariants({ size: "lg" })}>
            Create an account
          </Link>
          <Link
            to="/login"
            className={buttonVariants({ size: "lg", variant: "secondary" })}
          >
            Sign in
          </Link>
        </div>
      </Container>
    </section>
  )
}

export function Landing() {
  return (
    <div className="min-h-svh">
      <Header />
      <main>
        <Hero />
        <Stats />
        <Features />
        <Stack />
        <Security />
        <Cta />
      </main>
      <footer className="border-t py-6 text-xs text-muted-foreground">
        <Container className="flex justify-between">
          <span>© {new Date().getFullYear()} Starter</span>
          <span>Frontend for the Rust API</span>
        </Container>
      </footer>
    </div>
  )
}
