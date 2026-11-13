/**
 * A provider's name as people read it: `google` is "Google", `acme-id`
 * "Acme Id". The API knows providers only by the name in `OIDC_PROVIDERS`.
 */
export function providerLabel(name: string): string {
  return name
    .split(/[-_]/)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}
