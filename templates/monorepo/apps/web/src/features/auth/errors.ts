/**
 * Several distinct rejections share one status (400 `bad_request`), so the few
 * places that must tell them apart go by the server's message, kept here in
 * one place. Anything unmatched is treated as the more common case by callers.
 */

/** "this link is invalid or has expired", as opposed to a password-policy message. */
export function isLinkProblem(message: string) {
  return /link is invalid|has expired/i.test(message)
}

/** "current password is incorrect" from change-password / disable-2fa. */
export function isWrongCurrentPassword(message: string) {
  return /current password/i.test(message)
}
