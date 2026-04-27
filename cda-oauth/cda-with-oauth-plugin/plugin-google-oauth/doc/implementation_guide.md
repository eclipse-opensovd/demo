# Implementation Guide – `plugin-google-oauth`

<!--
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: 2025 The Contributors to Eclipse OpenSOVD (see CONTRIBUTORS)

See the NOTICE file(s) distributed with this work for additional
information regarding copyright ownership.

This program and the accompanying materials are made available under the
terms of the Apache License Version 2.0 which is available at
https://www.apache.org/licenses/LICENSE-2.0
-->

Overview of every source file in `src/` and its role in the plugin.

---

## `lib.rs`

Crate root. Declares the public module tree (`access_control`, `cda_security`, `handlers`, `init`, `plugin`, `types`) and re-exports the items that downstream code (the `main` crate) uses most often:

- `init_google_oauth_plugin` – top-level initialization function
- `GoogleOAuthSecurityPlugin` / `GoogleOAuthSecurityPluginData` – plugin types
- `OAuthPluginError` – error enum

---

## `types.rs`

All data types, request/response models, and the plugin error enum in one place:

| Type | Purpose |
|---|---|
| `OAuthPluginError` | Error variants for initialization failures (`MissingConfiguration`, `InitializationFailed`) |
| `GoogleOAuthConfig` | Holds `client_id` / `client_secret` loaded from environment variables |
| `JwksResponse`, `Jwk`, `JwksCache` | Types for fetching and caching Google's JSON Web Key Set (signing keys) |
| `GoogleClaims` | Decoded JWT claims from a Google ID token (`sub`, `email`, `iss`, `aud`, `iat`, `exp`). Implements the CDA `Claims` trait |
| `OAuthAuthorizationRequest` / `OAuthAuthorizationResponse` | Request and response payloads for the `/authorize` endpoint |
| `OAuthCallbackRequest` | Payload for the code-exchange callback (`code` + `state`) |
| `GoogleTokenResponse` | Raw token response from Google's token endpoint |
| `AccessTokenResponse` | Simplified token response returned to the client |
| `ReloadRolesData` / `ReloadRolesResponse` | Response payloads for the role-reload endpoint |

---

## `plugin.rs`

Core Google OAuth 2.0 provider logic. Contains all Google-specific operations:

- **Configuration** – `GoogleOAuthSecurityPlugin::init()` reads `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` from the environment and stores them in a `OnceLock`.
- **Authorization URL generation** – `generate_auth_url()` builds a Google consent URL using the desktop/installed-application redirect (`urn:ietf:wg:oauth:2.0:oob`).
- **Token exchange** – `exchange_code()` POSTs the authorization code to Google's token endpoint and returns a `GoogleTokenResponse`.
- **JWKS fetching & caching** – `fetch_jwks()` / `get_jwks()` retrieve Google's public signing keys and cache them for one hour.
- **ID-token verification** – `verify_id_token()` decodes the JWT header, looks up the matching key by `kid`, and verifies the signature, audience, and issuer with `jsonwebtoken`.

Also defines the two core structs:

- `GoogleOAuthSecurityPlugin` – stateless unit struct, entry point for all static methods above.
- `GoogleOAuthSecurityPluginData` – per-request struct that carries the validated `GoogleClaims`.

---

## `cda_security.rs`

Glue layer that implements the CDA/OpenSOVD security-plugin traits on top of the provider in `plugin.rs`. Every function here is a thin delegation or translation:

| Trait | Method | What it does |
|---|---|---|
| `SecurityPluginLoader` | *(marker)* | Registers `GoogleOAuthSecurityPlugin` as a loadable CDA plugin |
| `AuthorizationRequestHandler` | `authorize()` | Handles `POST /vehicle/v15/authorize` – parses the request, generates a CSRF state, and returns the Google auth URL |
| `SecurityPluginInitializer` | `initialize_from_request_parts()` | Extracts the `Bearer` token from incoming requests, verifies it with `verify_id_token`, and returns a boxed `SecurityPlugin` |
| `AuthApi` | `claims()` | Exposes the authenticated `GoogleClaims` |
| `SecurityApi` | `validate_service()` | Per-service authorization – maps audience metadata from the ECU database to user roles and denies access when the user lacks the required audience |
| `SecurityPlugin` | `as_auth_plugin()` / `as_security_plugin()` | Downcasting helpers so the framework can access the auth and authz facets of `GoogleOAuthSecurityPluginData` |

---

## `handlers.rs`

Axum HTTP route handlers mounted by `init.rs`:

| Handler | Route | Description |
|---|---|---|
| `oauth_callback_handler` | `POST /vehicle/v15/oauth/callback` | Accepts an authorization code, calls `exchange_code()`, and returns an `AccessTokenResponse` containing the ID token |
| `reload_roles_handler` | `POST /vehicle/v15/oauth/reload-roles` | Reloads `oauth_roles.toml` at runtime without restarting the server and reports the number of users loaded |

---

## `access_control.rs`

Role-based access control configuration backed by a TOML file (`oauth_roles.toml`):

- **Types** – `AccessControlConfig` (top-level) and `UserRoleEntry` (per-user email → roles mapping).
- **Global state** – `ACCESS_CONTROL_CONFIG` is a `RwLock<Option<…>>` that supports runtime reloading.
- **Public API**:
  - `load_access_control_config()` – initial load from disk.
  - `reload_access_control_config()` – runtime reload, returns the number of users loaded.
  - `lookup_user_roles(email)` – returns the `UserRoleEntry` for a given email, used by `cda_security::validate_service()`.
- **Config path** – read from the `OAUTH_ROLES_CONFIG` env var, defaults to `oauth_roles.toml` in the working directory.

---

## `init.rs`

Plugin bootstrapping and route registration:

1. Calls `GoogleOAuthSecurityPlugin::init()` to load the OAuth config from environment variables.
2. Calls `load_access_control_config()` to read `oauth_roles.toml`.
3. Registers both OAuth routes (`/vehicle/v15/oauth/callback` and `/vehicle/v15/oauth/reload-roles`) on the CDA `DynamicRouter`.

The single public entry point is `init_google_oauth_plugin(dynamic_router)`.
