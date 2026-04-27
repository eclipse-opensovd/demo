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

use cda_core::DiagServiceResponseStruct;
use clap::Parser;
use futures::FutureExt as _;
use opensovd_cda_lib::{
    AppError, cda_version, config::configfile::ConfigSanity, setup_tracing, shutdown_signal,
};
use plugin_google_oauth::{
    GoogleOAuthSecurityPlugin, GoogleOAuthSecurityPluginData, OAuthPluginError,
    init_google_oauth_plugin,
};

mod args;
use args::AppArgs;

/// Initialize the Google OAuth plugin, load vehicle data, and register vehicle routes.
async fn setup_oauth_plugin_and_routes(
    dynamic_router: &cda_sovd::dynamic_router::DynamicRouter,
    config: &opensovd_cda_lib::config::configfile::Configuration,
    shutdown_signal: impl std::future::Future<Output = ()> + Clone + Send + 'static,
) -> Result<(), AppError> {
    init_google_oauth_plugin(dynamic_router)
        .await
        .map_err(|e| match e {
            OAuthPluginError::MissingConfiguration(msg) => AppError::ConfigurationError(msg),
            OAuthPluginError::InitializationFailed(msg) => AppError::InitializationFailed(msg),
        })?;

    tracing::debug!("Loading vehicle data...");
    let vehicle_data = opensovd_cda_lib::load_vehicle_data::<_, GoogleOAuthSecurityPluginData>(
        config,
        shutdown_signal,
        None,
    )
    .await?;

    if vehicle_data.databases.is_empty() && config.database.exit_no_database_loaded {
        return Err(AppError::ResourceError(
            "No database loaded, exiting as configured".to_string(),
        ));
    }

    cda_sovd::add_vehicle_routes::<DiagServiceResponseStruct, _, _, GoogleOAuthSecurityPlugin>(
        dynamic_router,
        vehicle_data.uds_manager,
        config.flash_files_path.clone(),
        vehicle_data.file_managers,
        vehicle_data.locks,
        config.functional_description.clone(),
        config.components.clone(),
    )
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = AppArgs::parse();
    let mut config = opensovd_cda_lib::config::load_config().unwrap_or_else(|e| {
        println!("Failed to load configuration: {e}");
        println!("Using default values");
        opensovd_cda_lib::config::default_config()
    });
    config.validate_sanity()?;

    args.update_config(&mut config);

    let _tracing_guards = setup_tracing(&config)?;
    tracing::info!("Starting CDA - version {}", cda_version());

    let webserver_config = cda_sovd::WebServerConfig {
        host: config.server.address.clone(),
        port: config.server.port,
    };

    let clonable_shutdown_signal = shutdown_signal().shared();

    let (dynamic_router, webserver_task) =
        cda_sovd::launch_webserver(webserver_config.clone(), clonable_shutdown_signal.clone())
            .await?;

    tracing::debug!("Webserver is running. Loading sovd routes...");

    setup_oauth_plugin_and_routes(&dynamic_router, &config, clonable_shutdown_signal.clone())
        .await?;

    tracing::info!("CDA fully initialized and ready to serve requests");

    // Wait for shutdown signal
    clonable_shutdown_signal.await;
    tracing::info!("Shutting down...");

    webserver_task
        .await
        .map_err(|e| AppError::RuntimeError(format!("Webserver task join error: {e}")))?;

    Ok(())
}
