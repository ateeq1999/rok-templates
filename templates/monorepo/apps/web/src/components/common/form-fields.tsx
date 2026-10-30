import { useId, useState } from "react"
import type { ComponentProps, ReactNode } from "react"
import { HugeiconsIcon } from "@hugeicons/react"
import { Alert02Icon, ViewIcon, ViewOffIcon } from "@hugeicons/core-free-icons"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Field,
  FieldDescription,
  FieldError,
  FieldLabel,
} from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Spinner } from "@/components/ui/spinner"
import { Switch } from "@/components/ui/switch"

// Structural subset of TanStack Form's FieldApi, so any typed field fits.
export interface FieldLike {
  name: string
  state: { value: any; meta: { isTouched: boolean; errors: unknown[] } }
  form: { state: { isSubmitted: boolean } }
  handleBlur: () => void
  handleChange: (value: any) => void
}

/**
 * DOM ids must be unique per page, but field names are not: a dialog can show
 * `current_password` while the page behind it has one too. Suffix with useId so
 * labels and aria-describedby always bind to their own input.
 */
function useDomId(name: string) {
  const uid = useId()
  return `${name}-${uid}`
}

function fieldErrors(field: FieldLike) {
  if (!field.state.meta.isTouched && !field.form.state.isSubmitted) return []
  return field.state.meta.errors.map((e: unknown) => ({
    message:
      typeof e === "string"
        ? e
        : (e as { message?: string } | undefined)?.message,
  }))
}

type TextFieldProps = Omit<
  ComponentProps<typeof Input>,
  "value" | "onChange" | "onBlur" | "name" | "id"
> & {
  field: FieldLike
  label: string
  description?: ReactNode
}

export function TextField({
  field,
  label,
  description,
  ...inputProps
}: TextFieldProps) {
  const errors = fieldErrors(field)
  const invalid = errors.length > 0
  const id = useDomId(field.name)
  const errorId = `${id}-error`
  return (
    <Field data-invalid={invalid}>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Input
        {...inputProps}
        id={id}
        name={field.name}
        value={field.state.value ?? ""}
        onBlur={field.handleBlur}
        onChange={(e) => field.handleChange(e.target.value)}
        aria-invalid={invalid}
        aria-describedby={invalid ? errorId : undefined}
      />
      {description && <FieldDescription>{description}</FieldDescription>}
      <FieldError id={errorId} errors={errors} />
    </Field>
  )
}

export function PasswordField({
  field,
  label,
  autoComplete = "current-password",
  description,
  below,
}: {
  field: FieldLike
  label: string
  autoComplete?: "current-password" | "new-password"
  description?: ReactNode
  /** Extra content under the input, e.g. a strength meter. */
  below?: ReactNode
}) {
  const [visible, setVisible] = useState(false)
  const errors = fieldErrors(field)
  const invalid = errors.length > 0
  const id = useDomId(field.name)
  const errorId = `${id}-error`
  return (
    <Field data-invalid={invalid}>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <div className="relative">
        <Input
          id={id}
          name={field.name}
          type={visible ? "text" : "password"}
          autoComplete={autoComplete}
          value={field.state.value ?? ""}
          onBlur={field.handleBlur}
          onChange={(e) => field.handleChange(e.target.value)}
          aria-invalid={invalid}
          aria-describedby={invalid ? errorId : undefined}
          className="pr-9"
        />
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          className="absolute top-0.5 right-0.5"
          aria-label={visible ? "Hide password" : "Show password"}
          aria-pressed={visible}
          onClick={() => setVisible((v) => !v)}
        >
          <HugeiconsIcon icon={visible ? ViewOffIcon : ViewIcon} />
        </Button>
      </div>
      {below}
      {description && <FieldDescription>{description}</FieldDescription>}
      <FieldError id={errorId} errors={errors} />
    </Field>
  )
}

export function SelectField<T extends string>({
  field,
  label,
  options,
}: {
  field: FieldLike
  label: string
  options: { value: T; label: string }[]
}) {
  const id = useDomId(field.name)
  const errors = fieldErrors(field)
  return (
    <Field data-invalid={errors.length > 0}>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Select
        value={field.state.value}
        onValueChange={(v) => field.handleChange(v)}
        items={options}
      >
        <SelectTrigger id={id} className="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {options.map((o) => (
            <SelectItem key={o.value} value={o.value}>
              {o.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <FieldError errors={errors} />
    </Field>
  )
}

export function SwitchField({
  field,
  label,
  description,
  disabled,
}: {
  field: FieldLike
  label: string
  description?: string
  disabled?: boolean
}) {
  const id = useDomId(field.name)
  return (
    <Field orientation="horizontal">
      <div className="flex flex-1 flex-col gap-0.5">
        <FieldLabel htmlFor={id}>{label}</FieldLabel>
        {description && <FieldDescription>{description}</FieldDescription>}
      </div>
      <Switch
        id={id}
        checked={Boolean(field.state.value)}
        onCheckedChange={(v) => field.handleChange(v)}
        disabled={disabled}
      />
    </Field>
  )
}

export function FormError({ message }: { message?: string }) {
  if (!message) return null
  return (
    <Alert variant="destructive" role="alert">
      <HugeiconsIcon icon={Alert02Icon} />
      <AlertDescription>{message}</AlertDescription>
    </Alert>
  )
}

export function SubmitButton({
  children,
  pendingLabel,
  form,
  ...props
}: Omit<ComponentProps<typeof Button>, "type" | "form"> & {
  form: { Subscribe: React.ComponentType<any> }
  pendingLabel?: string
}) {
  return (
    <form.Subscribe selector={(s: { isSubmitting: boolean }) => s.isSubmitting}>
      {(isSubmitting: boolean) => (
        <Button type="submit" disabled={isSubmitting} {...props}>
          {isSubmitting && <Spinner />}
          {isSubmitting && pendingLabel ? pendingLabel : children}
        </Button>
      )}
    </form.Subscribe>
  )
}
