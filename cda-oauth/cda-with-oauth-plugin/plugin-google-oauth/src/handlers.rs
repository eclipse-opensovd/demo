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

//! HTTP route handlers for the OAuth plugin endpoints.

use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::{
    access_control::reload_access_control_config,
    plugin::GoogleOAuthSecurityPlugin,
    types::{AccessTokenResponse, OAuthCallbackRequest, ReloadRolesData, ReloadRolesResponse},
};

/// Handler for exchanging authorization code for tokens
///
/// This endpoint is used in the desktop OAuth flow where the user
/// manually copies the authorization code from their browser and
/// submits it to exchange for access tokens.
pub async fn oauth_callback_handler(
    Json(callback_request): Json<OAuthCallbackRequest>,
) -> impl IntoResponse {
    tracing::info!("Received OAuth code exchange request");

    // Exchange authorization code for tokens
    match GoogleOAuthSecurityPlugin::exchange_code(&callback_request.code).await {
        Ok(token_response) => {
            // Return the access token to the client
            (
                StatusCode::OK,
                Json(AccessTokenResponse {
                    access_token: token_response.id_token,
                    token_type: token_response.token_type,
                    expires_in: token_response.expires_in,
                }),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Handler for reloading the OAuth roles configuration.
///
/// POST /vehicle/v15/oauth/reload-roles
///
/// Reloads the oauth_roles.toml configuration file without restarting the server.
/// This allows updating user role assignments at runtime.
pub async fn reload_roles_handler() -> impl IntoResponse {
    tracing::info!("Received request to reload OAuth roles configuration");

    match reload_access_control_config() {
        Ok(user_count) => {
            tracing::info!(
                users = user_count,
                "OAuth roles configuration reloaded successfully"
            );
            (
                StatusCode::OK,
                Json(ReloadRolesResponse {
                    id: "reload-roles",
                    data: ReloadRolesData { success: true },
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to reload OAuth roles configuration");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ReloadRolesResponse {
                    id: "reload-roles",
                    data: ReloadRolesData { success: false },
                }),
            )
                .into_response()
        }
    }
}
