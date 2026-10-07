#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Contributors to the Eclipse Foundation
# Assisted By: Codex (GPT-6 Sol High)

set -eu
cd "$(dirname "$0")"

docker compose run --rm client-allowed --without-token --expect-status 401
docker compose run --rm client-allowed --audience wrong.audience --expect-status 403
docker compose run --rm client-denied --expect-status 403
docker compose run --rm client-allowed --expect-status 200
