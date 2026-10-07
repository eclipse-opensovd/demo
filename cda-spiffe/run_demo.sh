#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Contributors to the Eclipse Foundation
# Assisted by: Github Copilot (Opus 5.5)
#
# Builds and starts the CDA SPIFFE/SPIRE demo, registers the workloads and runs the smoke test.
# Safe to re-run: existing certificates and registration entries are kept.
# Behind a TLS-intercepting proxy: CA_BUNDLE=/path/to/root-ca.pem ./run_demo.sh

set -eu
cd "$(dirname "$0")/cda-with-spire-plugin"

SERVER_SOCKET=/run/spire/server/private/api.sock
AGENT_SOCKET=/run/spire/agent/public/api.sock

# wait_for <description> <timeout-seconds> <command...>
wait_for() {
    description="$1"
    timeout="$2"
    shift 2
    printf 'Waiting for %s ' "$description"
    elapsed=0
    until "$@" >/dev/null 2>&1; do
        if [ "$elapsed" -ge "$timeout" ]; then
            echo "timed out after ${timeout}s" >&2
            echo "Check: docker compose logs spire-server spire-agent cda" >&2
            exit 1
        fi
        printf '.'
        sleep 2
        elapsed=$((elapsed + 2))
    done
    echo ' ready'
}

spire_server() {
    docker compose exec -T spire-server /opt/spire/bin/spire-server "$@" -socketPath "$SERVER_SOCKET"
}

if [ -e spire/certs/ca-key.pem ]; then
    echo "Using existing demo certificates in spire/certs/"
else
    docker compose --profile setup run --build --rm certgen
fi

if [ -n "${CA_BUNDLE:-}" ]; then
    # Behind a TLS-intercepting proxy: pass its root CA (PEM) to Cargo as a build secret.
    docker build --secret id=cargo_ca,src="$CA_BUNDLE" --target cda -t cda-spiffe-demo:cda .
    docker build --secret id=cargo_ca,src="$CA_BUNDLE" --target client -t cda-spiffe-demo:client .
else
    docker compose build cda client-allowed
fi
# Compose builds the blueprint's ecu-sim image only if it is not present yet (e.g. from commercial-sdv-stack).
docker compose up -d spire-server spire-agent ecu-sim cda

wait_for "SPIRE server" 60 spire_server healthcheck
# The agent only reports healthy after it has attested to the server.
wait_for "SPIRE agent" 60 \
    docker compose exec -T spire-agent /opt/spire/bin/spire-agent healthcheck -socketPath "$AGENT_SOCKET"

if spire_server entry show -spiffeID spiffe://demo.example.org/demo/cda | grep -q 'Entry ID'; then
    echo "Workloads already registered"
else
    sh ./register-workloads.sh
fi

# Entries reach the agent on its next sync, and CDA may still be restarting or loading the MDD.
wait_for "SVID issuance and CDA" 120 docker compose run --rm client-allowed

sh ./smoke-test.sh
echo "Smoke test passed. Stop the demo with: (cd cda-with-spire-plugin && docker compose down -v)"
