import { queryOptions } from "@tanstack/react-query"
import { fetchStatsClientSide } from "./compute"

export const statsKeys = {
  all: ["stats"] as const,
}

// Swap the queryFn for GET /api/v1/admin/stats once the endpoint exists;
// the dashboard only consumes the `Stats` type.
export const statsQueryOptions = queryOptions({
  queryKey: statsKeys.all,
  queryFn: fetchStatsClientSide,
})
