import { format } from "date-fns"

/**
 * The text of the downloadable recovery-codes file. Plain text so it can be
 * printed or stored in a password manager. Nothing here is retrievable again.
 */
export function formatRecoveryCodesFile(
  codes: string[],
  opts: { account?: string; generatedAt?: Date; appName?: string } = {}
): string {
  const appName = opts.appName ?? "Starter"
  const lines = [
    `${appName} two-factor recovery codes`,
    ...(opts.account ? [`Account: ${opts.account}`] : []),
    `Generated: ${format(opts.generatedAt ?? new Date(), "yyyy-MM-dd HH:mm")}`,
    "",
    "Each code works once. Use one instead of your authenticator code if you",
    "lose access to your device. Keep this file somewhere safe and private.",
    "",
    ...codes.map((code, i) => `${String(i + 1).padStart(2, " ")}. ${code}`),
    "",
  ]
  return lines.join("\n")
}

export function recoveryCodesFilename(generatedAt: Date = new Date()) {
  return `recovery-codes-${format(generatedAt, "yyyy-MM-dd")}.txt`
}
