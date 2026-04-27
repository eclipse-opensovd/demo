# CDA with Google OAuth Plugin

<!--
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: 2025 The Contributors to Eclipse OpenSOVD (see CONTRIBUTORS)

See the NOTICE file(s) distributed with this work for additional
information regarding copyright ownership.

This program and the accompanying materials are made available under the
terms of the Apache License Version 2.0 which is available at
https://www.apache.org/licenses/LICENSE-2.0
-->

A demonstration of the Classic Diagnostic Adapter (CDA) integrated with a Google OAuth 2.0 security plugin for secure authentication and authorization.

## Overview

This project showcases the integration of Google OAuth 2.0 authentication with the Eclipse OpenSOVD Classic Diagnostic Adapter (CDA). It uses the desktop/installed application OAuth flow, where users authenticate through their browser and manually provide authorization codes to the application.

## Project Structure

```
├── main/                      # Main CDA application
│   ├── src/main.rs           # Application entry point
│   └── Cargo.toml            # Main package configuration
│
├── plugin-google-oauth/       # Google OAuth security plugin
│   ├── src/lib.rs            # Plugin implementation
│   ├── Cargo.toml            # Plugin package configuration
│   ├── GOOGLE_OAUTH.md        # Detailed OAuth documentation
│   ├── instructions.md        # Implementation instructions
│   ├── examples/              # Usage examples
│   └── doc/                   # Additional documentation
│
├── oauth_roles.toml           # OAuth role configuration
├── FLXC1000.mdd              # Vehicle diagnostic data definition
└── launch.sh                  # Launch script
```

## Features

- **Google OAuth 2.0 Desktop Flow**: Implements the desktop/installed application flow without redirect URIs
- **OpenID Connect Support**: Validates ID tokens from Google with JWT verification
- **Bearer Token Authentication**: Uses Google ID tokens for API authentication
- **Production-Ready Security**:
  - JWT signature verification using Google's JWKS public keys
  - Token expiration validation
  - Claim verification (issuer, audience, expiration)
  - JWKS key caching (1 hour as recommended by Google)
- **Environment-Based Configuration**: Simple setup via environment variables
- **SOVD Compliant**: Follows SOVD security specifications

## Prerequisites

- Rust 1.70+ with Cargo
- Google Cloud Console account with OAuth 2.0 credentials
- Environment variables configured for Google OAuth

## Setup

### 1. Create Google OAuth Credentials

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Navigate to **APIs & Services** > **Credentials**
4. Click **Create Credentials** > **OAuth client ID**
5. Select **Desktop app** as the application type
6. Copy the **Client ID** and **Client Secret**

### 2. Configure Environment Variables

```bash
export GOOGLE_CLIENT_ID="your-client-id.apps.googleusercontent.com"
export GOOGLE_CLIENT_SECRET="your-client-secret"
```

### 3. Build the Project

```bash
cargo build --release
```

### 4. Run the Application

```bash
./launch.sh
```

Or manually:

```bash
cargo run --bin demo --release
```

## Usage

### OAuth Flow Overview

#### Step 1: Initiate Authorization

```bash
curl -X POST http://localhost:20002/vehicle/v15/authorize \
  -H "Content-Type: application/json" \
  -d '{
    "state": "random-csrf-token",
    "scopes": ["openid", "email", "profile"]
  }'
```

Response:
```json
{
  "authorization_url": "https://accounts.google.com/o/oauth2/v2/auth?...",
  "state": "random-csrf-token"
}
```

#### Step 2: User Authentication

1. Open the `authorization_url` in your browser
2. Log in to your Google account
3. Grant the requested permissions
4. Copy the authorization code displayed in the browser

#### Step 3: Exchange Code for Token

```bash
curl -X POST http://localhost:20002/vehicle/v15/oauth/callback \
  -H "Content-Type: application/json" \
  -d '{
    "code": "4/0AY0e-...",
    "state": "random-csrf-token"
  }'
```

Response:
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3599
}
```

#### Step 4: Access Protected Resources

```bash
curl -X GET http://localhost:20002/vehicle/v15/components \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIs..."
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/vehicle/v15/authorize` | POST | Initiate OAuth authorization flow |
| `/vehicle/v15/oauth/callback` | POST | Exchange authorization code for token |
| `/vehicle/v15/components` | GET | Access protected vehicle components |

## Security Features

### Token Verification
- **JWT Signature Verification**: Uses Google's RSA public keys from JWKS endpoint
- **Issuer Validation**: Ensures token is from Google
- **Audience Validation**: Confirms token is for this application
- **Expiration Checking**: Validates token has not expired
- **Key Caching**: JWKS keys cached for 1 hour as recommended by Google

### Production Recommendations

1. **Secret Management**:
   - Never commit secrets to version control
   - Use secret management services (AWS Secrets Manager, HashiCorp Vault, etc.)
   - Rotate secrets regularly

2. **HTTPS/TLS**:
   - Always use HTTPS in production
   - Use valid TLS certificates from trusted CAs

3. **Token Handling**:
   - Exchange authorization codes immediately
   - Never log or store authorization codes
   - Implement token refresh mechanisms

4. **Rate Limiting**:
   - Implement rate limiting on OAuth endpoints
   - Protect against brute force and DoS attacks

## Testing

### Using the OAuth Flow Test Script

```bash
./oauth_flow_test.sh
```

This script tests the complete OAuth flow end-to-end.

### Manual Testing with curl

See the Usage section above for curl commands.

## Configuration Files

### oauth_roles.toml
Contains OAuth role and permission configurations for the security plugin.

### FLXC1000.mdd
Vehicle diagnostic data definition file containing diagnostic service specifications.

## Documentation

- **[GOOGLE_OAUTH.md](plugin-google-oauth/GOOGLE_OAUTH.md)** - Comprehensive Google OAuth plugin documentation
- **[instructions.md](plugin-google-oauth/instructions.md)** - Implementation instructions and design guidelines
- **[plugin-google-oauth/doc/](plugin-google-oauth/doc/)** - Additional plugin documentation

## Troubleshooting

### "GOOGLE_CLIENT_ID environment variable not set"
Ensure you've exported the environment variables before running the application:
```bash
export GOOGLE_CLIENT_ID="your-client-id"
export GOOGLE_CLIENT_SECRET="your-client-secret"
```

### "Invalid token audience"
- Verify that the `GOOGLE_CLIENT_ID` matches the audience in the token
- Ensure you created a "Desktop app" OAuth client in Google Cloud Console

### "Token has expired"
- Tokens are time-limited; request a new token by repeating the OAuth flow
- Verify system time is synchronized correctly

### "Failed to exchange code for token"
- Authorization codes are single-use; cannot reuse the same code
- Ensure you copied the complete authorization code
- Verify the client secret is correct
- Check that you're using a Desktop app OAuth client type

## Dependencies

### Core Dependencies
- **cda-core, cda-database, cda-interfaces** - Eclipse OpenSOVD CDA libraries
- **cda-plugin-security** - Security plugin architecture
- **cda-sovd** - SOVD integration

### Web Framework
- **axum** 0.8 - Web framework for Rust
- **aide** 0.16 - API documentation support

### Authentication
- **jsonwebtoken** 10.2 - JWT handling with JWKS support
- **reqwest** 0.12 - HTTP client for Google API calls

### Other
- **tokio** - Async runtime
- **serde/serde_json** - JSON serialization
- **tracing** - Structured logging

## License

Licensed under the Apache License, Version 2.0. See the [LICENSE](LICENSE) file in the root directory for details.

Copyright (c) 2025 The Contributors to Eclipse OpenSOVD

## Contributing

Contributions are welcome. Please ensure:
- Code follows Rust best practices
- All tests pass
- Security best practices are maintained
- Documentation is updated accordingly

## References

- [Google OAuth 2.0 Documentation](https://developers.google.com/identity/protocols/oauth2)
- [OpenID Connect Documentation](https://openid.net/connect/)
- [Eclipse OpenSOVD Project](https://projects.eclipse.org/projects/automotive.opensovd)
- [SOVD Specification](https://www.asam.net/standards/detail/mdf423)

