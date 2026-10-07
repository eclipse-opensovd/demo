# syntax=docker/dockerfile:1
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Contributors to the Eclipse Foundation
# Assisted By: Codex (GPT-6 Sol High)

FROM rust:1.93-slim-trixie AS builder
ARG SOURCE_DATE_EPOCH=0
ARG SOURCE_GIT_SHA=99c60782a6e25f6cff9ba3a3418d0750f936016e
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates build-essential pkg-config libssl-dev make \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY . .
RUN --mount=type=secret,id=cargo_ca,required=false \
    if [ -f /run/secrets/cargo_ca ]; then export CARGO_HTTP_CAINFO=/run/secrets/cargo_ca; fi; \
    SOURCE_DATE_EPOCH="$SOURCE_DATE_EPOCH" SOURCE_GIT_SHA="$SOURCE_GIT_SHA" \
    cargo build --release --locked -p cda-spiffe-demo -p spiffe-demo-client

FROM debian:trixie-slim AS cda
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3t64 iproute2 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/target/release/cda-spiffe-demo /app/cda-spiffe-demo
COPY --chmod=755 entrypoint.sh /app/entrypoint.sh
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["-d", "/app/odx"]

FROM debian:trixie-slim AS client
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/spiffe-demo-client /usr/local/bin/spiffe-demo-client
ENTRYPOINT ["/usr/local/bin/spiffe-demo-client"]
