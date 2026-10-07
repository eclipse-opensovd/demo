#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Contributors to the Eclipse Foundation
# Assisted By: Codex (GPT-6 Sol High)

set -eu
umask 077

mkdir -p /certs
if [ -e /certs/ca-key.pem ] || [ -e /certs/agent-key.pem ]; then
    echo "Demo certificates already exist; refusing to overwrite them." >&2
    exit 1
fi

openssl req -x509 -newkey rsa:3072 -sha256 -nodes -days 30 \
    -keyout /certs/ca-key.pem -out /certs/ca-cert.pem \
    -subj '/CN=Demo SPIRE CA' -config /config/openssl.cnf -extensions v3_ca
openssl req -newkey rsa:3072 -sha256 -nodes \
    -keyout /certs/agent-key.pem -out /certs/agent.csr \
    -subj '/CN=demo-agent'
openssl x509 -req -in /certs/agent.csr -days 30 -sha256 \
    -CA /certs/ca-cert.pem -CAkey /certs/ca-key.pem -CAcreateserial \
    -extfile /config/openssl.cnf -extensions agent \
    -out /certs/agent-cert.pem
cp /certs/ca-cert.pem /certs/trusted-certs.pem
rm /certs/agent.csr
echo "Created 30-day demo certificates in spire/certs/"
