#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Contributors to the Eclipse Foundation
# Assisted By: Codex (GPT-6 Sol High)

set -eu
cd "$(dirname "$0")"

register() {
    spiffe_id="$1"
    image="$2"
    role="$3"
    docker compose exec -T spire-server /opt/spire/bin/spire-server entry create \
        -socketPath /run/spire/server/private/api.sock \
        -parentID spiffe://demo.example.org/spire/agent/x509pop/demo-agent \
        -spiffeID "$spiffe_id" \
        -selector "docker:image_id:$image" \
        -selector "docker:env:SPIFFE_DEMO_ROLE=$role"
}

register spiffe://demo.example.org/demo/cda cda-spiffe-demo:cda cda
register spiffe://demo.example.org/demo/client-allowed cda-spiffe-demo:client allowed
register spiffe://demo.example.org/demo/client-denied cda-spiffe-demo:client denied
