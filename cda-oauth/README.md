# CDA OAuth Demo Project

<!--
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: 2025 The Contributors to Eclipse OpenSOVD (see CONTRIBUTORS)

See the NOTICE file(s) distributed with this work for additional
information regarding copyright ownership.

This program and the accompanying materials are made available under the
terms of the Apache License Version 2.0 which is available at
https://www.apache.org/licenses/LICENSE-2.0
-->

A demonstration of Eclipse OpenSOVD's Classic Diagnostic Adapter (CDA) with Google OAuth 2.0 authentication, automotive diagnostic format conversion tools, and ECU simulation capabilities.
This was used as demo for `Diagnostics Reimagined` at OCA 2026. 

## 🎯 Overview

This repository showcases a complete automotive diagnostic ecosystem with enterprise-grade security:

- **CDA with OAuth Plugin** – A production-ready Classic Diagnostic Adapter implementing Google OAuth 2.0 and OpenID Connect for secure authentication and role-based access control
- **Diagnostic Converter** – A high-performance Rust tool for converting between ODX, YAML, and MDD diagnostic formats
- **ECU Simulator** – A Kotlin-based ECU simulator for testing diagnostic services
- **ECU Databases** – Sample diagnostic database in yaml and converted MDD file

## ⚠️ Disclaimer

**This is demonstration code only.** The code in this folder is functional and demonstrates the integration concepts, but it has **not been manually reviewed** and is **not production-ready**. It is intended for:

- Proof-of-concept demonstrations
- Development and testing environments

For production use, this code requires thorough security review, testing, and hardening. 
At the very least it requires an in depth manual review, as major parts of this are AI generated.

## 📁 Project Structure

```
cda-oauth/
├── cda-with-oauth-plugin/     # Main CDA application with OAuth security
│   ├── main/                  # CDA application entry point
│   ├── plugin-google-oauth/   # Google OAuth 2.0 security plugin
│   ├── oauth_roles.toml       # Role-based access control configuration
│   ├── FLXC1000.mdd           # Sample diagnostic database
│   └── launch.sh              # Quick-start launcher
│
├── diag-converter/           # Diagnostic format converter (ODX ↔ YAML ↔ MDD)
│   ├── diag-cli/             # Command-line interface
│   ├── diag-ir/              # Intermediate representation
│   ├── diag-odx/             # ODX parser/writer
│   ├── diag-yaml/            # YAML parser/writer
│   └── mdd-format/           # MDD binary format handler
│
├── ecu-databases/            # Sample ECU diagnostic databases
│   └── ocx-ecu.{mdd,yaml}    # OCX ECU diagnostic data
│
└── ecu-sim/                   # ECU simulator for testing
    ├── src/                   # Kotlin source code
    └── docker/                # Docker deployment
```

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.88+ with Cargo
- **Java** 17+ (for ECU simulator)
- **Google Cloud Console** account with OAuth 2.0 credentials
- **Protocol Buffers** compiler (`protoc`) for building the diagnostic converter

### 1. Set Up Google OAuth Credentials

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Navigate to **APIs & Services** → **Credentials**
4. Click **Create Credentials** → **OAuth client ID**
5. Select **Desktop app** as the application type
6. Copy the **Client ID** and **Client Secret**

### 2. Configure Environment

```bash
export GOOGLE_CLIENT_ID="your-client-id.apps.googleusercontent.com"
export GOOGLE_CLIENT_SECRET="your-client-secret"
```

### 3. Build and Run

```bash
# Build all components
cd cda-with-oauth-plugin
cargo build --release

# Launch CDA with OAuth
./launch.sh
```

The CDA server will start on `http://localhost:8080` with OAuth authentication enabled.

## 🔐 Authentication Flow

This implementation uses the OAuth 2.0 **desktop/installed application flow**:

1. **Initialize**: Application prompts user to authenticate
2. **Authorize**: User opens Google consent URL in browser
3. **Authenticate**: User logs in with Google and grants permissions
4. **Code Exchange**: User copies authorization code back to application
5. **Token Validation**: Application exchanges code for ID token and verifies JWT signature
6. **Access Control**: Application validates user roles and grants/denies access to services

### Security Features

- ✅ **JWT Signature Verification** using Google's JWKS public keys
- ✅ **Token Expiration Validation** prevents replay attacks
- ✅ **Claim Verification** (issuer, audience, expiration)
- ✅ **JWKS Key Caching** (1-hour cache as recommended by Google)
- ✅ **Role-Based Access Control** via `oauth_roles.toml`
- ✅ **Audience-Based Authorization** for service-level permissions

## 🔧 Components

### 1. CDA with OAuth Plugin

**Location**: `cda-with-oauth-plugin/`

The main application integrating Eclipse OpenSOVD's CDA with Google OAuth 2.0 authentication.

**Key Features**:
- Google OAuth 2.0 desktop flow implementation
- OpenID Connect token validation
- Bearer token authentication
- Role-based access control (RBAC)
- Runtime role configuration reload

**Quick Start**:
```bash
cd cda-with-oauth-plugin
./launch.sh
```

**Documentation**:
- [Main README](cda-with-oauth-plugin/README.md)
- [Google OAuth Details](cda-with-oauth-plugin/plugin-google-oauth/GOOGLE_OAUTH.md)
- [Implementation Guide](cda-with-oauth-plugin/plugin-google-oauth/doc/implementation_guide.md)

### 2. Diagnostic Converter

**Location**: `diag-converter/`

Rust tool for converting between automotive diagnostic formats through a canonical intermediate representation.

**Supported Formats**:
| Format | Extension | Read | Write | Description |
|--------|-----------|------|-------|-------------|
| ODX | `.odx`, `.pdx` | ✅ | ✅ | ISO 22901-1 XML diagnostic data |
| YAML | `.yml`, `.yaml` | ✅ | ✅ | Human-readable diagnostic descriptions |
| MDD | `.mdd` | ✅ | ✅ | Binary format (Protobuf + FlatBuffers) |

**Quick Start**:
```bash
cd diag-converter
cargo install --path diag-cli

# Convert YAML to MDD
diag-converter convert input.yml -o output.mdd

# Convert ODX to YAML
diag-converter convert input.odx -o output.yml

# Validate a diagnostic file
diag-converter validate input.mdd
```

**Documentation**: [Converter README](diag-converter/README.md)

### 3. ECU Simulator

**Location**: `ecu-sim/`

A Kotlin-based ECU simulator for testing diagnostic services and authentication flows.

**Quick Start**:
```bash
cd ecu-sim
./gradlew run
```

**Docker Deployment**:
```bash
cd ecu-sim/docker
./build_docker.sh
docker run -p 8081:8081 ecu-sim
```

**Documentation**: [Simulator README](ecu-sim/README.md)

### 4. ECU Databases

**Location**: `ecu-databases/`

Sample diagnostic database files in multiple formats (MDD, YAML) for testing and development.

## 📖 Usage Examples

### Example 1: Test OAuth Authentication Flow

```bash
# Terminal 1: Start CDA with OAuth
cd cda-with-oauth-plugin
./launch.sh

# Terminal 2: Test authorization endpoint
curl -X POST http://localhost:8080/vehicle/v15/authorize \
  -H "Content-Type: application/json" \
  -d '{"state": "test-state"}'

# Follow the authorization URL, get the code, then exchange it
curl -X POST http://localhost:8080/vehicle/v15/oauth/callback \
  -H "Content-Type: application/json" \
  -d '{"code": "YOUR_AUTH_CODE", "state": "test-state"}'
```

### Example 2: Convert Diagnostic Formats

```bash
cd diag-converter

# Convert YAML diagnostic database to MDD binary
diag-converter convert ../ecu-databases/ocx-ecu.yaml \
  -o ../ecu-databases/ocx-ecu-converted.mdd

# Validate the converted file
diag-converter validate ../ecu-databases/ocx-ecu-converted.mdd

# Get file information
diag-converter info ../ecu-databases/ocx-ecu-converted.mdd
```

### Example 3: Role-Based Access Control

Edit `cda-with-oauth-plugin/oauth_roles.toml`:

```toml
[[users]]
email = "user@example.com"
roles = ["aftermarket", "engineering"]

[[users]]
email = "admin@example.com"
roles = ["oem", "production", "engineering", "aftermarket"]
```

Reload roles without restarting:

```bash
curl -X POST http://localhost:8080/vehicle/v15/oauth/reload-roles \
  -H "Authorization: Bearer YOUR_ID_TOKEN"
```

## 🔍 API Endpoints

### CDA OAuth Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/vehicle/v15/authorize` | Initiate OAuth flow, get authorization URL |
| `POST` | `/vehicle/v15/oauth/callback` | Exchange authorization code for ID token |
| `POST` | `/vehicle/v15/oauth/reload-roles` | Reload role configuration at runtime |

### Authenticated Diagnostic Endpoints

All diagnostic endpoints require a valid Bearer token in the `Authorization` header:

```bash
curl -H "Authorization: Bearer YOUR_ID_TOKEN" \
  http://localhost:8080/vehicle/v15/diagnostics/services
```

## 🧪 Testing

### Test OAuth Flow

```bash
cd cda-with-oauth-plugin/plugin-google-oauth/examples
./oauth_flow_test.sh
```

### Test Diagnostic Conversion

```bash
cd diag-converter/diag-cli
cargo test
```

### Run ECU Simulator Tests

```bash
cd ecu-sim
./gradlew test
```

## 🏗️ Architecture

### OAuth Plugin Integration

![OAuth Plugin Integration](cda-with-oauth-plugin/plugin-google-oauth/doc/OAuth%20Plugin%20Integration.svg)

### Google OAuth 2.0 Desktop Flow

![Google OAuth 2.0 Desktop Flow](cda-with-oauth-plugin/plugin-google-oauth/doc/Google%20OAuth%202.0%20Desktop%20Flow.svg)

### Detailed Integration Architecture

![OAuth Plugin Integration - Detailed](cda-with-oauth-plugin/plugin-google-oauth/doc/OAuth%20Plugin%20Integration%20-%20Detailed.svg)

## 🤝 Contributing

This is a demonstration project, while contributions are always welcome, this isn't under active development.

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) files in individual components for details.

## 🔗 Related Projects

- [Eclipse OpenSOVD](https://github.com/eclipse-opensovd/classic-diagnostic-adapter) – Classic Diagnostic Adapter
- [SOVD Specification](https://www.asam.net/standards/detail/sovd/) – ASAM SOVD Standard
- [Google OAuth 2.0](https://developers.google.com/identity/protocols/oauth2) – Google Identity Platform

## 📞 Support

For issues and questions:
- CDA OAuth Plugin: See [GOOGLE_OAUTH.md](cda-with-oauth-plugin/plugin-google-oauth/GOOGLE_OAUTH.md)
- Diagnostic Converter: See [diag-converter README](diag-converter/README.md)
- ECU Simulator: See [ecu-sim README](ecu-sim/README.md)

