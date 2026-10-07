<!--
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: 2026 Contributors to the Eclipse Foundation
-->

# CDA with a SPIRE security plugin

This demo runs Eclipse OpenSOVD's Classic Diagnostic Adapter (CDA) with a
SPIFFE JWT-SVID security plugin. A SPIRE agent gives the test client a signed
identity token for audience `sovd.cda`. The client sends it as an HTTP Bearer
token; CDA asks the agent to validate it and checks the caller's SPIFFE ID
against a Rego diagnostic-service allowlist.

The workspace has two crates. `main` is the CDA with the SPIFFE security
plugin exactly as deployed in the commercial SDV stack: its `src/` files are
verbatim copies of `uservices/cda/src` at commercial-sdv-stack commit
`5cb6ec19`, and its `OpaConfig` comes from that commit's `common` crate.
`client` obtains a fresh JWT-SVID and makes a diagnostic request. The CDA
dependencies are pinned to the same `99c60782` revision the commercial stack
uses. If any updates are made here, it would be ideal to have the commercial
stack reference this as the source of truth to avoid diverging implementations.
The ECU side is the blueprint's as well: `odx/blueprint-ecu.mdd` is a verbatim
copy from the same commit, Compose builds the ECU simulator directly from the
blueprint repository at that commit, and the CDA runs with its default
configuration, as in the blueprint.

## Run with Docker Compose

Prerequisites: Docker Compose with Linux containers. SPIRE's Docker workload
attestor needs access to the Docker socket and host PID namespace. On Windows,
use Docker Desktop's Linux-container mode.

From this directory:

```bash
# Creates the demo SPIRE certificates in spire/certs/ (needed once per clone)
docker compose --profile setup run --build --rm certgen
docker compose build cda client-allowed
docker compose up -d spire-server spire-agent ecu-sim cda
./register-workloads.sh
./smoke-test.sh
```

Give the SPIRE agent a few seconds to attest before registering or running the
tests. If CDA started before the agent socket was ready, its `on-failure`
restart policy retries it. `docker compose logs -f spire-agent cda` shows startup
progress.

The smoke test checks four requests against
`/vehicle/v15/components/blueprint-ecu/data/powertrain_mode`:

| Request | Expected result |
| --- | --- |
| No Bearer token | HTTP 401 |
| JWT-SVID for the wrong audience | HTTP 403 |
| Valid token for `client-denied` | HTTP 403 |
| Valid token for `client-allowed` | HTTP 200 |

For a single call, run `docker compose run --rm client-allowed`. You can
override its URL with `--url`, its audience with `--audience`, or its expected
status with `--expect-status`. The client prints the caller's SPIFFE ID and
response, but never prints the token.

The test policy permits only `Powertrain_Mode_Read` and `Powertrain_Mode_Write`
for
`spiffe://demo.example.org/demo/client-allowed`. Edit
`config/authorization-data.json` to grant other diagnostic services, then
restart CDA to reload the compiled policy. SPIRE registration maps Docker
selectors to each test workload's SPIFFE ID. The CDA `/authorize` route
returns HTTP 501 because JWT-SVIDs are issued through the SPIRE Workload API.

The setup command creates 30-day local demonstration certificates in
`spire/certs/`; Git ignores them. It refuses to overwrite existing keys.
Do not use them outside this disposable demo. To reset the SPIRE registration
database, run `docker compose down -v` (this removes this Compose project's
named volumes), then start the stack and register workloads again.

If a corporate TLS proxy prevents Cargo from downloading crates during the
Docker build, export its trusted root CA as a PEM file and prebuild the two
Rust images with a BuildKit secret (replace `/path/to/root.pem`):

```sh
docker build --secret id=cargo_ca,src=/path/to/root.pem --target cda -t cda-spiffe-demo:cda .
docker build --secret id=cargo_ca,src=/path/to/root.pem --target client -t cda-spiffe-demo:client .
```

The ECU simulator is built from the blueprint's git repository, and its
Dockerfile has no CA hook. If the proxy blocks that build, reuse an `ecu-sim`
image already built by the commercial-sdv-stack (same image name).

Then continue with `docker compose up` above. The CA file is used only during
the builds and is not copied into the runtime images.

To build the Rust code locally, run `cargo build --locked` from this directory.
Running the CDA binary also requires a reachable SPIRE Workload API socket,
`SPIFFE_ENDPOINT_SOCKET`, the policy files, and the `odx/` database directory.
