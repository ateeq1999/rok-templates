# Docs

Guides for the monorepo's mobile App.

| Guide | Use when |
|---|---|
| [Getting started](./getting-started.md) | You want the App running against the local API |
| [Architecture](./architecture.md) | You want to understand the layers (ApiClient → repository → cubit → view) |
| [Feature flags](./feature-flags.md) | You want to toggle, drop, or add a feature flag — and what is not in the App yet |
| [Adding a feature](./adding-a-feature.md) | You want to wire a new API call + screen end-to-end |
| [Push notifications](./push-notifications.md) | You want Firebase and FCM (a recipe) |
| [Security](./security.md) | You're about to ship — review what's built in and what's left to decide |

The API's contract is `../../api/openapi.json`; the App calls it through
the client generated from it, `../../../clients/dart`.
