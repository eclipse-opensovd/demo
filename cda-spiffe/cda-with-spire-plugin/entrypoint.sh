#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Contributors to the Eclipse Foundation
# Assisted By: Codex (GPT-6 Sol High)

set -eu
tester_ip=$(ip -4 -o addr show dev eth0 | awk '{split($4, address, "/"); print address[1]; exit}')
if [ -z "$tester_ip" ]; then
    echo "Could not determine CDA's DoIP tester address" >&2
    exit 1
fi
exec /app/cda-spiffe-demo --tester-address "$tester_ip" "$@"
