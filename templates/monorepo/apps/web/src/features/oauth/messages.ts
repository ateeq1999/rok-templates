const messages: Record<string, string> = {
  access_denied: "Sign-in was cancelled.",
  invalid_state:
    "That sign-in link expired or was already used. Please try again.",
  provider_error: "Google/GitHub could not be reached. Please try again.",
  email_not_verified:
    "That account has no verified email address, so we cannot sign you in.",
  account_exists_unverified:
    "An account with this email exists but its email was never verified. Sign in with your password (or verify the email first), then connect the provider in Settings.",
  already_linked: "That account is already connected to a different user.",
  account_disabled: "This account is disabled.",
  server_error: "Something went wrong. Please try again.",
}

export function oauthErrorMessage(reason: string | undefined): string {
  if (!reason) return messages.server_error
  return messages[reason] ?? messages.server_error
}
