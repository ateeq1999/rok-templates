# syntax=docker/dockerfile:1
#
# The API's image: dependencies built in a layer of their own (cargo-chef),
# so a code change rebuilds only your crate; run from distroless as a
# non-root user. One image serves every APP_ROLE (all, web, worker).
#
# It builds from the project root, where the workspace's `Cargo.lock` is
# (Dockerfile.dockerignore beside this file keeps the other Apps out):
#
#   docker build -f apps/api/Dockerfile -t api .
#   docker run --env-file apps/api/.env -p 8080:8080 api

ARG RUST_VERSION=1.99

FROM rust:${RUST_VERSION}-bookworm AS chef
RUN cargo install cargo-chef --locked --version 0.1.78
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Only the API's package is built: the workspace's other members (the
# desktop App) need toolchains this image doesn't have. Its binary is
# named after the package (`default-run`).
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
COPY apps/api/Cargo.toml apps/api/Cargo.toml
RUN api="$(sed -n 's/^name = "\(.*\)"$/\1/p' apps/api/Cargo.toml | head -n 1)" \
    && cargo chef cook --release --locked --recipe-path recipe.json --package "$api"
COPY . .
RUN api="$(sed -n 's/^name = "\(.*\)"$/\1/p' apps/api/Cargo.toml | head -n 1)" \
    && cargo build --release --locked --package "$api" --bin "$api" \
    && cp "target/release/$api" /server

# The migrations image (`docker build --target migrate`): `rok migrate up`
# over `migrations/`, what the deploy runs once before the new version
# starts (deploy/terraform's migrate task). Its `rok` is the release CI
# pins (`ROK_VERSION` in the root .github/workflows/api.yml), downloaded
# and checked against its published SHA-256, for the architecture built.
FROM debian:bookworm-slim AS rok
ARG ROK_VERSION=0.9.0
ARG TARGETARCH
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /rok
RUN case "${TARGETARCH:-amd64}" in \
        amd64) arch=x86_64 ;; \
        arm64) arch=aarch64 ;; \
        *) echo "no rok release for ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
    && name="rok-${ROK_VERSION}-${arch}-unknown-linux-musl" \
    && url="https://github.com/ateeq1999/rok-v2/releases/download/v${ROK_VERSION}" \
    && curl -fsSL --retry 3 -O "$url/$name.tar.gz" -O "$url/$name.tar.gz.sha256" \
    && sha256sum -c "$name.tar.gz.sha256" \
    && tar -xzf "$name.tar.gz" \
    && mv "$name/rok" /usr/local/bin/rok

FROM gcr.io/distroless/cc-debian12:nonroot AS migrate
COPY --from=rok /usr/local/bin/rok /rok
COPY apps/api/migrations /migrations
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
