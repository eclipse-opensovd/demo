<!--
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: 2026 Contributors to the Eclipse Foundation
Assisted by: Github Copilot (Opus 5.5)
-->

# CDA SPIFFE/SPIRE Demo

Eclipse OpenSOVD's Classic Diagnostic Adapter (CDA) with a security plugin
that authenticates calling **workloads** by their SPIFFE identity and
authorizes each diagnostic service with a local Rego policy. A SPIRE agent
attests a client container and issues it a short-lived JWT-SVID; the CDA
accepts the request only if the token is valid and the policy allows that
identity to call that service.

The plugin is copied unchanged from the Eclipse SDV
[commercial-sdv-stack](https://github.com/eclipse-sdv-blueprints/commercial-sdv-stack)
blueprint work done by Kai Hudalla on [PR
#5](https://github.com/eclipse-sdv-blueprints/commercial-sdv-stack/pull/5). It
is replicated here so it can be run and evaluated without the rest of the
blueprint and so the wider community has easier access to it.

> **Demonstration code only.** Certificates are generated locally, valid for
> 30 days and must not be reused. Parts of the demo scaffolding are AI
> assisted and have not had an in-depth security review.

## Quick start

Requires Docker with Linux containers (Linux, WSL2 or Docker Desktop). From
this directory:

```sh
./run_demo.sh
# Behind a TLS-intercepting proxy, pass its root CA for the image builds:
CA_BUNDLE=/path/to/root-ca.pem ./run_demo.sh
```

The script generates the demo SPIRE certificates (first run only), builds and
starts the stack, waits for the SPIRE server and agent, registers the
workloads, waits until a client can obtain an SVID and reach the CDA, and then
runs the smoke test. It is safe to re-run. The equivalent manual steps, from
[`cda-with-spire-plugin/`](cda-with-spire-plugin/), are:

```sh
docker compose --profile setup run --build --rm certgen   # once: demo SPIRE certificates
docker compose build cda client-allowed
docker compose up -d spire-server spire-agent ecu-sim cda   # builds ecu-sim if its image is missing
sh ./register-workloads.sh
sh ./smoke-test.sh
```

If you run them by hand, wait a few seconds after `up` for the agent to
attest before registering.
The smoke test calls
`GET /vehicle/v15/components/blueprint-ecu/data/powertrain_mode`, the resource
the blueprint's powertrain mode controller uses, four times:

| Caller | Expected |
| --- | --- |
| No token | 401 |
| `client-allowed`, token for the wrong audience | 403 |
| `client-denied`, valid token, not in policy | 403 |
| `client-allowed`, valid token, allowed by policy | 200 |

Reset everything with `docker compose down -v`. Proxy, single-call and
troubleshooting details are in the
[component README](cda-with-spire-plugin/README.md).

## Repository structure

```
cda-spiffe/
└── cda-with-spire-plugin/
    ├── main/                      # CDA binary + SPIFFE security plugin (verbatim from commercial-sdv-stack)
    │   └── src/
    │       ├── main.rs            # starts CDA via opensovd_cda_lib::run_with_ext
    │       ├── args.rs            # CDA args + Rego policy file options
    │       └── spiffe_security_plugin.rs
    ├── client/                    # test workload: fetches a JWT-SVID, calls CDA
    ├── config/
    │   ├── authz.rego             # policy: allow if service is listed for the caller's SPIFFE ID
    │   └── authorization-data.json# SPIFFE ID -> allowed diagnostic services
    ├── odx/blueprint-ecu.mdd      # blueprint ECU diagnostic database (verbatim from commercial-sdv-stack)
    ├── spire/                     # SPIRE server + agent config, demo cert generation
    ├── compose.yaml               # spire-server, spire-agent, ecu-sim (built from the blueprint repo), cda, test clients
    ├── Dockerfile                 # builds the cda and client images
    ├── register-workloads.sh      # SPIRE registration entries (Docker selectors)
    └── smoke-test.sh              # positive and negative requests
```

## How it works

1. The SPIRE agent attests to the SPIRE server with an X.509 certificate.
2. `register-workloads` maps Docker selectors (image ID plus a
   `SPIFFE_DEMO_ROLE` environment variable) to SPIFFE IDs such as
   `spiffe://demo.example.org/demo/client-allowed`.
3. The client asks the agent's Workload API (a Unix socket) for a JWT-SVID
   with audience `sovd.cda` and sends it as `Authorization: Bearer ...`.
4. The CDA validates the token through its own Workload API connection
   (signature, expiry, audience `sovd.cda`) and records the SPIFFE ID.
5. For each diagnostic service, the plugin evaluates `data.authz.allow` with
   `{spiffe_id, service_name}`. Anything other than `true` is denied.

Only `client-allowed` is granted anything: `Powertrain_Mode_Read` and
`Powertrain_Mode_Write`, the services the blueprint grants its powertrain mode
controller. Edit
[`authorization-data.json`](cda-with-spire-plugin/config/authorization-data.json)
and restart the `cda` service to change that. The CDA `/authorize` endpoint
returns 501, because tokens come from SPIRE, not from the CDA.

## Why SPIFFE/SPIRE for in-vehicle workloads

The [OAuth demo](../cda-oauth/) answers "which person or tool is calling the
CDA, and on whose behalf?". Inside a vehicle, most calls to the CDA come from
other software, not people: a powertrain mode controller, a fleet management
agent, an update orchestrator. For those calls the question is "which approved
piece of software is actually running and making this request?". Client
secrets and API keys answer that poorly: they are long-lived, have to be
provisioned into each image, can be copied, and say nothing about whether the
code holding them is the code that was approved.

[SPIFFE](https://spiffe.io/) gives each workload a standard identity
(`spiffe://<trust-domain>/<path>`). SPIRE issues it only after attesting the
running process: the agent checks properties it can observe on the host, such
as Unix UID, container labels or the SHA-256 hash of the binary, against
entries registered on the server. The workload holds no secret of its own. It
asks the local agent and gets back a JWT-SVID that is valid for minutes and
for one audience. A modified or unregistered binary gets no identity at all.

For a diagnostic adapter in a vehicle this brings:

- **Least privilege per workload.** A local Rego policy maps each SPIFFE ID to
  the exact diagnostic services it may invoke; everything else is denied.
- **No static credentials in images.** Nothing to provision, rotate or leak;
  SVIDs expire quickly and are renewed automatically.
- **Works with intermittent connectivity.** Validation needs only the trust
  bundle held by the local agent, not a round trip to a cloud identity
  provider.
- **One trust model from vehicle to backend.** The same trust domain, or
  federated ones, can span ECUs, vehicle compute and fleet services, and can
  sit alongside OAuth/OIDC for human or external access.

In the commercial-sdv-stack, SPIRE attests the powertrain mode controller, the
fleet management and vehicle properties services, and the CDA, so that only
the approved controller can read or change the powertrain mode through SOVD.

## Relationship to commercial-sdv-stack

| Item | Source |
| --- | --- |
| `main/src/*.rs`, `config/authz.rego` | Verbatim from `uservices/cda/src` and `config/cda/config` at commit `5cb6ec19` |
| `odx/blueprint-ecu.mdd` | Verbatim from `config/cda/odx` at the same commit |
| ECU simulator | Not vendored: Compose builds the blueprint's `ecu-sim` directly from the blueprint repository at the same commit, under the same image name, so an image already built by the commercial-sdv-stack is reused |
| `OpaConfig` (policy file options) | `common` crate, git dependency on the same commit |
| CDA libraries | `eclipse-opensovd/classic-diagnostic-adapter` at `99c60782`, as in the blueprint |
| SPIRE config, test clients, compose | Demo-specific |

### Future Work

In the future the Commercial SDV Blueprint should be updated to potentially reference this implementation to avoid divergence.

## Building without Docker

On Linux or WSL2, with `pkg-config` and `libssl-dev` installed, run
`cargo build --locked` in `cda-with-spire-plugin/`. Running the binary also
needs a SPIRE agent socket (`SPIFFE_ENDPOINT_SOCKET`), the policy files
(`--policy-file`, `--authorization-data-file`) and the database directory
(`-d odx`). Like the blueprint, it runs with the CDA's default configuration.
