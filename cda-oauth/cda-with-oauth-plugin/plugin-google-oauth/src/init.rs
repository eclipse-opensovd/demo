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

//! Plugin initialization and route registration.

use cda_sovd::dynamic_router::DynamicRouter;

use crate::{
    access_control::load_access_control_config,
    handlers::{oauth_callback_handler, reload_roles_handler},
    plugin::GoogleOAuthSecurityPlugin,
    types::OAuthPluginError,
};

/// Initializes the Google OAuth security plugin and registers the OAuth callback route.
///
/// This function performs two operations:
/// 1. Initializes the Google OAuth plugin configuration from environment variables
/// 2. Registers the `/vehicle/v15/oauth/callback` endpoint to the dynamic router
///
/// # Errors
/// Returns an error if the Google OAuth plugin configuration cannot be loaded from
/// environment variables (GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET must be set).
pub async fn init_google_oauth_plugin(
    dynamic_router: &DynamicRouter,
) -> Result<(), OAuthPluginError> {
    // Initialize OAuth configuration from environment variables
    GoogleOAuthSecurityPlugin::init().map_err(|e| OAuthPluginError::InitializationFailed(e))?;

    // Load role-based access control configuration
    load_access_control_config().map_err(|e| OAuthPluginError::InitializationFailed(e))?;

    // Register the OAuth routes
    add_oauth_routes(dynamic_router).await;

    Ok(())
}

/// Adds the OAuth routes to the dynamic router.
///
/// This function registers:
/// - `/vehicle/v15/oauth/callback` - Exchange authorization codes for access tokens
/// - `/vehicle/v15/oauth/reload-roles` - Reload the oauth_roles.toml configuration
///
/// This is a private helper function used internally by the OAuth plugin integration.
async fn add_oauth_routes(dynamic_router: &DynamicRouter) {
    dynamic_router
        .update_router(|router| {
            router
                .route(
                    "/vehicle/v15/oauth/callback",
                    axum::routing::post(oauth_callback_handler),
                )
                .route(
                    "/vehicle/v15/oauth/reload-roles",
                    axum::routing::post(reload_roles_handler),
                )
        })
        .await;
    tracing::debug!("OAuth routes added to webserver (callback + reload-roles)");
}
