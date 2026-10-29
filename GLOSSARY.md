# rok-templates

The complete starter projects `rok sqlc init --template` copies, and the cross-cutting parts they ship.

## Language

**Template**:
A complete starter project that rok-data-sqlc's `init` copies into a new project: application skeleton, schema, and the API generated from it — for one web framework, or a **Monorepo template**.
_Avoid_: starter, boilerplate, scaffold (that is a single scaffolded file)

**Monorepo template**:
A Template made of several Apps — a backend, a web app, a mobile app — that share one API contract and the Clients generated from it.
_Avoid_: workspace (that is Cargo's), full-stack template, project template

**App**:
One member of a Monorepo template, such as the API, the web app or the mobile app.
_Avoid_: package, service (that is a microservice), project

**Capability**:
A cross-cutting part of a template that every API feature can use, such as mail, storage, rate limiting, jobs or metrics; it ships in the template, is inert until configured, and is the user's once scaffolded.
_Avoid_: feature (that is an API feature), plugin, add-on, module

**Service**:
One independently deployed backend of a services Template, with its own database, reached by other Services over gRPC and by Events.
_Avoid_: microservice, App (that is a monorepo member), the service layer (a feature's `service.rs`)

**Gateway**:
The one HTTP entry of a services Template: it issues auth, forwards requests to Services over gRPC and serves the API contract the Clients follow.
_Avoid_: BFF, proxy, edge

**Event**:
A typed, versioned record that something happened, published by one Service on its own transaction and delivered at least once to the Services that consume it.
_Avoid_: message (that is the broker's), notification, job (that is work, not a fact)

**Integration**:
Code that picks a vendor or an opt-in dependency — a tracing exporter, a cache, a message broker — placed into a project at its Integration points by a command, and the user's once placed.
_Avoid_: plugin, add-on, Capability (that ships built in), feature (that is an API feature)

**Integration point**:
A marked place in a Template's code or configuration where an Integration's code is put by a command, so the place is fixed in advance rather than guessed.
_Avoid_: hook, slot, injection site, extension point

**Blueprint**:
A saved choice of a Template, its Integrations and its init options, named by an id, from which new projects are created alike.
_Avoid_: preset (that is the engine's), template id, recipe, starter config

**Shared file**:
A framework-independent file both templates hold byte for byte, so a change to one template's copy is a change to the other's.
_Avoid_: common file, duplicate
