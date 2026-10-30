# syntax=docker/dockerfile:1
#
# The API's image: dependencies built in a layer of their own (cargo-chef),
# so a code change rebuilds only your crate; run from distroless as a
# non-root user. One image serves every APP_ROLE (all, web, worker).
#
#   docker build -t api .
#   docker run --env-file .env -p 8080:8080 api

ARG RUST_VERSION=1.99

FROM rust:${RUST_VERSION}-bookworm AS chef
RUN cargo install cargo-chef --locked --version 0.1.78
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked \
    # The crate's one binary, whatever the crate is named, as /server.
    && find target/release -maxdepth 1 -type f -executable -exec cp {} /server \;

# The migrations image (`docker build --target migrate`): `rok migrate up`
# over `migrations/`, what the deploy runs once before the new version
# starts (deploy/terraform's migrate task).
FROM chef AS rok
RUN cargo install rok-data-cli --locked --version 0.4.0

FROM gcr.io/distroless/cc-debian12:nonroot AS migrate
COPY --from=rok /usr/local/cargo/bin/rok /rok
COPY migrations /migrations
WORKDIR /
USER nonroot
ENTRYPOINT ["/rok", "migrate", "up"]

# The server, the default target.
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /server /server
# JSON lines for the log collector; override with LOG_FORMAT=pretty.
ENV BIND_ADDR=0.0.0.0:8080 \
    LOG_FORMAT=json
EXPOSE 8080
USER nonroot
ENTRYPOINT ["/server"]
