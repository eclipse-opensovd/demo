/*
 * SPDX-License-Identifier: Apache-2.0
 * SPDX-FileCopyrightText: 2025 The Contributors to Eclipse OpenSOVD (see CONTRIBUTORS)
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 */

//! OAuth-related data types, request/response models, and error definitions.

use cda_plugin_security::Claims as ClaimsTrait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ── Error type ───────────────────────────────────────────────────────────────

/// Error type for OAuth plugin initialization failures
#[derive(Debug, thiserror::Error)]
pub enum OAuthPluginError {
    #[error("Missing OAuth configuration: {0}")]
    MissingConfiguration(String),
    #[error("OAuth plugin initialization failed: {0}")]
    InitializationFailed(String),
}

// ── Configuration ────────────────────────────────────────────────────────────

/// Google OAuth configuration loaded from environment variables
#[derive(Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

impl GoogleOAuthConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| "GOOGLE_CLIENT_ID environment variable not set".to_string())?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .map_err(|_| "GOOGLE_CLIENT_SECRET environment variable not set".to_string())?;

        Ok(Self {
            client_id,
            client_secret,
        })
    }
}

// ── JWKS types (Google signing keys) ─────────────────────────────────────────

/// JWKS (JSON Web Key Set) response from Google
#[derive(Debug, Deserialize)]
pub struct JwksResponse {
    pub keys: Vec<Jwk>,
}

/// JSON Web Key
#[derive(Debug, Deserialize, Clone)]
pub struct Jwk {
    pub kid: String,
    #[serde(rename = "use")]
    #[allow(dead_code)]
    pub key_use: String,
    #[allow(dead_code)]
    pub kty: String,
    #[allow(dead_code)]
    pub alg: String,
    pub n: String,
    pub e: String,
}

/// Cached JWKS keys
pub struct JwksCache {
    pub keys: Vec<Jwk>,
    pub cached_at: std::time::Instant,
}

// ── JWT claims ───────────────────────────────────────────────────────────────

/// JWT claims from Google ID token
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoogleClaims {
    /// Subject (user identifier)
    pub sub: String,
    /// Email address
    pub email: String,
    /// Email verified flag
    pub email_verified: bool,
    /// Issuer
    pub iss: String,
    /// Audience (client ID)
    pub aud: String,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: usize,
}

impl ClaimsTrait for GoogleClaims {
    fn sub(&self) -> &str {
        &self.sub
    }
}

// ── OAuth request / response models ──────────────────────────────────────────

/// Authorization request payload for initiating OAuth flow
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OAuthAuthorizationRequest {
    /// Optional state parameter for CSRF protection
    pub state: Option<String>,
    /// Optional scopes to request (defaults to profile and email)
    pub scopes: Option<Vec<String>>,
}

/// Authorization response containing the redirect URL
#[derive(Debug, Serialize, JsonSchema)]
pub struct OAuthAuthorizationResponse {
    /// URL to redirect the user to for Google authentication
    pub authorization_url: String,
    /// State parameter for verification
    pub state: String,
}

/// OAuth callback request containing the authorization code
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OAuthCallbackRequest {
    /// Authorization code from Google
    pub code: String,
    /// State parameter for CSRF verification
    pub state: String,
}

/// Token response from Google
#[derive(Debug, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

/// Access token response for the client
#[derive(Debug, Serialize, JsonSchema)]
pub struct AccessTokenResponse {
    /// The access token (ID token from Google)
    pub access_token: String,
    /// Token type (Bearer)
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: i64,
}

// ── Reload-roles response ────────────────────────────────────────────────────

/// Response data for the reload roles endpoint.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ReloadRolesData {
    /// Whether the reload was successful
    pub success: bool,
}

/// Response from the reload roles endpoint.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ReloadRolesResponse {
    /// Endpoint identifier
    pub id: &'static str,
    /// Response data
    pub data: ReloadRolesData,
}
