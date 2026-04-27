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

//! Role-based access control configuration.
//!
//! Loads user → role mappings from a TOML file (`oauth_roles.toml`) and exposes
//! helpers that the security plugin uses to perform audience-based authorization.

use std::sync::RwLock;

use serde::Deserialize;

// ── Types ────────────────────────────────────────────────────────────────────

/// Top-level access control configuration loaded from TOML.
///
/// Example `oauth_roles.toml`:
/// ```toml
/// [[users]]
/// email = "admin@example.com"
/// roles = ["Development", "Manufacturing", "AfterSales", "AfterMarket", "Supplier"]
/// additional_roles = ["FlashExpert"]
///
/// [[users]]
/// email = "technician@partner.com"
/// roles = ["AfterSales"]
/// additional_roles = []
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AccessControlConfig {
    /// List of user role assignments
    pub users: Vec<UserRoleEntry>,
}

/// Maps an email address to its granted audience roles and additional roles.
#[derive(Debug, Clone, Deserialize)]
pub struct UserRoleEntry {
    /// The user's email address (must match the Google OAuth `email` claim)
    pub email: String,
    /// Standard audience roles: `Development`, `Manufacturing`, `AfterSales`,
    /// `AfterMarket`, `Supplier`
    #[serde(default)]
    pub roles: Vec<String>,
    /// Additional (custom) audience roles verified against
    /// `Audience::enabled_audiences` on the service
    #[serde(default)]
    pub additional_roles: Vec<String>,
}

// ── Global state ─────────────────────────────────────────────────────────────

/// Global access-control configuration, supports runtime reloading.
static ACCESS_CONTROL_CONFIG: RwLock<Option<AccessControlConfig>> = RwLock::new(None);

// ── Public API ───────────────────────────────────────────────────────────────

/// Load the access-control configuration from a TOML file (initial load).
pub fn load_access_control_config() -> Result<(), String> {
    let config = load_access_control_config_inner()?;
    let mut guard = ACCESS_CONTROL_CONFIG
        .write()
        .map_err(|e| format!("Failed to acquire write lock: {e}"))?;
    *guard = Some(config);
    Ok(())
}

/// Reload the access-control configuration from the TOML file.
/// Returns the number of users loaded.
pub fn reload_access_control_config() -> Result<usize, String> {
    let config = load_access_control_config_inner()?;
    let user_count = config.users.len();
    let mut guard = ACCESS_CONTROL_CONFIG
        .write()
        .map_err(|e| format!("Failed to acquire write lock: {e}"))?;
    *guard = Some(config);
    Ok(user_count)
}

/// Look up the role entry for a given email address.
pub fn lookup_user_roles(email: &str) -> Option<UserRoleEntry> {
    ACCESS_CONTROL_CONFIG.read().ok().and_then(|guard| {
        guard
            .as_ref()
            .and_then(|cfg| cfg.users.iter().find(|u| u.email == email).cloned())
    })
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Load or reload the access-control configuration from a TOML file.
///
/// The path is read from the `OAUTH_ROLES_CONFIG` environment variable.
/// Falls back to `oauth_roles.toml` in the current working directory.
fn load_access_control_config_inner() -> Result<AccessControlConfig, String> {
    let path =
        std::env::var("OAUTH_ROLES_CONFIG").unwrap_or_else(|_| "oauth_roles.toml".to_string());

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read access-control config '{path}': {e}"))?;

    let config: AccessControlConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse access-control config '{path}': {e}"))?;

    tracing::info!(
        users = config.users.len(),
        path = %path,
        "Loaded access-control configuration"
    );

    Ok(config)
}
