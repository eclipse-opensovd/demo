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

//! CDA security plugin trait implementations.
//!
//! This module contains **all** the glue between the Google OAuth
//! provider and the CDA plugin security API.  Everything here is a
//! thin delegation layer – the actual OAuth logic lives in [`crate::plugin`].

use async_trait::async_trait;
use axum::{Json, RequestPartsExt, body::Bytes, http::StatusCode, response::IntoResponse};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use cda_plugin_security::{
    AuthApi, AuthError, AuthorizationRequestHandler, Claims as ClaimsTrait, SecurityApi,
    SecurityPlugin, SecurityPluginInitializer, SecurityPluginLoader,
};
use aide::axum::IntoApiResponse;
use http::{HeaderMap, request::Parts};
use sovd_interfaces::error::{ApiErrorResponse, ErrorCode};

use crate::{
    access_control::lookup_user_roles,
    plugin::{GoogleOAuthSecurityPlugin, GoogleOAuthSecurityPluginData},
    types::{OAuthAuthorizationRequest, OAuthAuthorizationResponse},
};

// ── SecurityPluginLoader (marker trait) ──────────────────────────────────────

/// Registers this type as a loadable CDA/OpenSOVD security plugin.
///
/// The trait is a marker only: the framework uses it to accept
/// `GoogleOAuthSecurityPlugin` as the plugin type that backs authentication
/// and authorization for incoming SOVD requests.
/// This trait represents a complete security plugin implementation that can both
/// initialize plugin instances from requests and handle authorization requests.
/// It is used as the main interface for integrating security plugins into the
/// web server framework.

impl SecurityPluginLoader for GoogleOAuthSecurityPlugin {}

// ── AuthorizationRequestHandler ──────────────────────────────────────────────

#[async_trait]
impl AuthorizationRequestHandler for GoogleOAuthSecurityPlugin {
    /// Implements the CDA/OpenSOVD authorization entry point at
    /// `/vehicle/v15/authorize`.
    ///
    /// The framework calls this trait method when a client wants to start an
    /// authentication flow. In this plugin, the method translates the generic
    /// OpenSOVD authorization request into a Google OAuth desktop-flow URL.
    ///
    /// # Arguments
    /// * `headers` - The HTTP request headers
    /// * `body_bytes` - The raw request body containing client credentials
    ///
    /// # Should Return
    /// An HTTP response containing either:
    /// - Success: the OAuth authorization URL and CSRF state
    /// - Error: an OpenSOVD-compatible error response with appropriate status code
    async fn authorize(_headers: HeaderMap, body_bytes: Bytes) -> impl IntoApiResponse {
        // Parse the authorization request
        let request =
            match axum::extract::Json::<OAuthAuthorizationRequest>::from_bytes(&body_bytes) {
                Ok(req) => req.0,
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to parse authorization request");
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiErrorResponse::<String> {
                            message: "Invalid request payload".to_string(),
                            error_code: ErrorCode::VendorSpecific,
                            vendor_code: Some("bad-request".to_string()),
                            parameters: None,
                            error_source: None,
                            schema: None,
                        }),
                    )
                        .into_response();
                }
            };

        // Generate a random state for CSRF protection
        let state = request.state.unwrap_or_else(|| {
            use rand::Rng;
            let random_bytes: [u8; 16] = rand::rng().random();
            hex::encode(random_bytes)
        });

        let scopes = request.scopes.unwrap_or_else(|| {
            vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ]
        });

        // Generate authorization URL
        match GoogleOAuthSecurityPlugin::generate_auth_url(&state, &scopes) {
            Ok(auth_url) => (
                StatusCode::OK,
                Json(OAuthAuthorizationResponse {
                    authorization_url: auth_url,
                    state,
                }),
            )
                .into_response(),
            Err(e) => {
                tracing::error!(error = %e, "Failed to generate auth URL");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiErrorResponse::<()> {
                        message: "Failed to initialize OAuth flow".to_string(),
                        error_code: ErrorCode::SovdServerFailure,
                        vendor_code: None,
                        parameters: None,
                        error_source: None,
                        schema: None,
                    }),
                )
                    .into_response()
            }
        }
    }
}

// ── SecurityPluginInitializer ────────────────────────────────────────────────

#[async_trait]
impl SecurityPluginInitializer for GoogleOAuthSecurityPlugin {
    /// Builds a request-scoped security plugin instance from the incoming HTTP
    /// request.
    ///
    /// OpenSOVD calls this trait method before dispatching a protected API
    /// request. This is the handoff from the transport layer into the security
    /// layer: we read the bearer token, verify it with Google, and return a
    /// boxed plugin object that carries authenticated identity data for the
    /// rest of request processing.
    async fn initialize_from_request_parts(
        &self,
        parts: &mut Parts, // The HTTP request head consists of a method, uri, version, and a set of header fields.
    ) -> Result<Box<dyn SecurityPlugin>, AuthError> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, "Failed to extract token");
                AuthError::NoTokenProvided
            })?;

        // Verify and decode the Google ID token
        let claims = GoogleOAuthSecurityPlugin::verify_id_token(bearer.token()).await?;

        Ok(Box::new(GoogleOAuthSecurityPluginData { claims }))
    }
}

// ── Trait impls for GoogleOAuthSecurityPluginData ─────────────────────────────

impl AuthApi for GoogleOAuthSecurityPluginData {
    /// Exposes the authenticated claims through the CDA `AuthApi` trait.
    ///
    /// OpenSOVD uses this accessor when other framework components need the
    /// normalized identity attached to the current request.
    fn claims(&self) -> Box<&dyn ClaimsTrait> {
        Box::new(&self.claims)
    }
}

impl SecurityApi for GoogleOAuthSecurityPluginData {
    /// Performs service-level authorization for the current request.
    ///
    /// After `initialize_from_request_parts` has authenticated the caller,
    /// OpenSOVD invokes this trait method for each diagnostic service access.
    /// The implementation maps service audience metadata from the ECU database
    /// to configured user roles and denies access when the authenticated user
    /// does not satisfy the required audience.
    fn validate_service(
        &self,
        service: &cda_database::datatypes::DiagService,
    ) -> Result<(), cda_interfaces::DiagServiceError> {
        let email = &self.claims.email;

        // Extract audience from the service's DiagComm
        if let Some(diag_comm) = service.diag_comm() {
            let service_name = diag_comm.short_name().unwrap_or("<unknown>");

            if let Some(audience) = diag_comm.audience() {
                // Check if any audience restrictions are actually defined
                let has_standard_roles = audience.is_development()
                    || audience.is_manufacturing()
                    || audience.is_after_sales()
                    || audience.is_after_market()
                    || audience.is_supplier();

                let has_additional_roles = audience
                    .enabled_audiences()
                    .map(|enabled| !enabled.is_empty())
                    .unwrap_or(false);

                // If no audiences are defined, everyone is allowed
                if !has_standard_roles && !has_additional_roles {
                    tracing::debug!(
                        email = %email,
                        service = %service_name,
                        "No audience restrictions defined, allowing access"
                    );
                    return Ok(());
                }

                // Audience restrictions exist, so look up user roles from configuration
                let user_roles = lookup_user_roles(email).ok_or_else(|| {
                    tracing::warn!(email = %email, "No role configuration found for user");
                    cda_interfaces::DiagServiceError::AccessDenied(format!(
                        "No role configuration for user '{email}'"
                    ))
                })?;

                // ── Check standard audience flags ────────────────────────
                let checks: &[(&str, bool)] = &[
                    ("Development", audience.is_development()),
                    ("Manufacturing", audience.is_manufacturing()),
                    ("AfterSales", audience.is_after_sales()),
                    ("AfterMarket", audience.is_after_market()),
                    ("Supplier", audience.is_supplier()),
                ];

                let required_roles: Vec<&str> = checks
                    .iter()
                    .filter_map(|&(role_name, service_requires)| {
                        if service_requires {
                            Some(role_name)
                        } else {
                            None
                        }
                    })
                    .collect();

                if !required_roles.is_empty() {
                    let has_any_role = required_roles
                        .iter()
                        .any(|&required_role| user_roles.roles.iter().any(|r| r == required_role));

                    if !has_any_role {
                        tracing::warn!(
                            email = %email,
                            service = %service_name,
                            required_roles = ?required_roles,
                            user_roles = ?user_roles.roles,
                            "User lacks any of the required audience roles"
                        );
                        return Err(cda_interfaces::DiagServiceError::AccessDenied(format!(
                            "Access denied: user has roles {:?} but service '{service_name}' \
                             requires one of {:?}",
                            user_roles.roles, required_roles
                        )));
                    }
                }

                // ── Check additional (custom) audiences ──────────────────
                if let Some(enabled) = audience.enabled_audiences() {
                    if !enabled.is_empty() {
                        let required_additional: Vec<&str> =
                            enabled.iter().filter_map(|a| a.short_name()).collect();

                        if !required_additional.is_empty() {
                            let has_any_additional = required_additional.iter().any(|&required| {
                                user_roles.additional_roles.iter().any(|r| r == required)
                            });

                            if !has_any_additional {
                                tracing::warn!(
                                    email = %email,
                                    service = %service_name,
                                    required_additional = ?required_additional,
                                    user_additional = ?user_roles.additional_roles,
                                    "User lacks any of the required additional audiences"
                                );
                                return Err(cda_interfaces::DiagServiceError::AccessDenied(
                                    format!(
                                        "Access denied: user has additional roles {:?} but \
                                         service '{service_name}' requires one of {:?}",
                                        user_roles.additional_roles, required_additional
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl SecurityPlugin for GoogleOAuthSecurityPluginData {
    /// Returns the authentication facet of this request-scoped plugin.
    ///
    /// The umbrella `SecurityPlugin` trait lets the framework hold one boxed
    /// trait object and then downcast it into the authentication API it needs.
    fn as_auth_plugin(&self) -> &dyn AuthApi {
        self
    }

    /// Returns the authorization facet of this request-scoped plugin.
    ///
    /// This gives OpenSOVD access to the `SecurityApi` view used for
    /// per-service permission checks after authentication has succeeded.
    fn as_security_plugin(&self) -> &dyn SecurityApi {
        self
    }
}
