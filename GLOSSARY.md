# rok-templates

The complete starter projects `rok sqlc init --template` copies, and the cross-cutting parts they ship.

## Language

**Template**:
A complete starter project, for one web framework, that rok-data-sqlc's `init` copies into a new project: application skeleton, schema, and the API generated from it.
_Avoid_: starter, boilerplate, scaffold (that is a single scaffolded file)

**Capability**:
A cross-cutting part of a template that every API feature can use, such as mail, storage, rate limiting, jobs or metrics; it ships in the template, is inert until configured, and is the user's once scaffolded.
_Avoid_: feature (that is an API feature), plugin, add-on, module

**Shared file**:
A framework-independent file both templates hold byte for byte, so a change to one template's copy is a change to the other's.
_Avoid_: common file, duplicate
