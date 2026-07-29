use cda_interfaces::config::ConfigSanity;
use cda_plugin_security::{DefaultSecurityPlugin, DefaultSecurityPluginData};
use cda_sovd::WebServerConfig;
use clap::Parser;
use futures::future::FutureExt;
use opensovd_cda_lib::{
    AppArgs, AppError, config::load_config_with_fallback, load_vehicle_data, setup_tracing,
    setup_vehicle_and_routes, shutdown_signal,
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = AppArgs::parse();

    let (mut config, _disk_loaded) = load_config_with_fallback(args.config.as_deref());
    args.update_config(&mut config);
    config.validate_sanity()?;
    let _tracing_guards = setup_tracing(&config)?;
    tracing::info!("Starting CDA - OEM Routes Demo");

    let webserver_config = WebServerConfig {
        host: config.server.address.clone(),
        port: config.server.port,
    };

    let clonable_shutdown_signal = shutdown_signal().shared();

    let (dynamic_router, webserver_task) =
        cda_sovd::launch_webserver(webserver_config, clonable_shutdown_signal.clone()).await?;

    // run_with_ext cannot be used here because its hook runs before vehicle loading and only
    // receives the router, while the OEM routes require the loaded UDS manager. Until cda-main
    // exposes a post-load hook, this lower-level flow also causes setup_vehicle_and_routes to
    // load the vehicle data a second time.
    let vehicle_data = load_vehicle_data::<_, DefaultSecurityPluginData>(
        &config,
        clonable_shutdown_signal.clone(),
        None,
    )
    .await?;

    let uds = vehicle_data.uds_manager.clone();

    plugin_oem_routes::init_oem_routes(&dynamic_router, &uds).await;

    setup_vehicle_and_routes::<DefaultSecurityPluginData, DefaultSecurityPlugin>(
        config,
        &dynamic_router,
        None,
        clonable_shutdown_signal.clone(),
    )
    .await?;

    tracing::info!("CDA fully initialized and ready to serve requests.");

    clonable_shutdown_signal.await;
    tracing::info!("Shutting down...");
    webserver_task
        .await
        .map_err(|e| AppError::RuntimeError(format!("Webserver task join error: {e}")))?;

    Ok(())
}
