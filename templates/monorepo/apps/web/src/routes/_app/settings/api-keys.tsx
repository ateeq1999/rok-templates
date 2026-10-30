import { createFileRoute } from "@tanstack/react-router"
import { ApiKeysPanel } from "@/features/api-keys/components/api-keys-panel"

export const Route = createFileRoute("/_app/settings/api-keys")({
  component: ApiKeysPanel,
})
