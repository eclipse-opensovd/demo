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

//! # Google OAuth Plugin Integration
//!
//! This crate provides the complete Google OAuth 2.0 security plugin implementation
//! that integrates with Google's authentication services and the CDA web server (SOVD).
//!
//! ## OAuth 2.0 Desktop Flow
//!
//! The plugin implements the OAuth 2.0 desktop/installed application flow:
//! - User authenticates directly with Google in their browser
//! - User copies the authorization code from the browser
//! - Application exchanges authorization code for access tokens
//! - Validates tokens for API requests
//!
//! ## Configuration
//!
//! The plugin reads configuration from environment variables:
//! - `GOOGLE_CLIENT_ID`: OAuth client ID from Google Cloud Console
//! - `GOOGLE_CLIENT_SECRET`: OAuth client secret from Google Cloud Console
//!
//! Note: This implementation uses the desktop flow without redirect URIs.
//!
//! ## Modules
//!
//! - [`access_control`] – Role-based access control (TOML config, user → role mapping)
//! - [`types`] – Data types, request/response models, error definitions
//! - [`plugin`] – Core security plugin and trait implementations
//! - [`handlers`] – HTTP route handlers (token exchange, role reload)
//! - [`init`] – Plugin initialization and route registration

pub mod access_control;
pub mod cda_security;
pub mod handlers;
pub mod init;
pub mod plugin;
pub mod types;

// Re-export the most commonly used items at the crate root so that
// downstream code (`main`) can keep using `plugin_google_oauth::Foo`.
pub use init::init_google_oauth_plugin;
pub use plugin::{GoogleOAuthSecurityPlugin, GoogleOAuthSecurityPluginData};
pub use types::OAuthPluginError;
