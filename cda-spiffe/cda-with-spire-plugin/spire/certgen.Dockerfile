# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Contributors to the Eclipse Foundation
# Assisted By: Codex (GPT-6 Sol High)

FROM rust:1.88-slim-trixie
COPY spire/openssl.cnf /config/openssl.cnf
COPY spire/generate-certs.sh /generate-certs.sh
ENTRYPOINT ["/bin/sh", "/generate-certs.sh"]
