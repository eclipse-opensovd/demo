use aide::axum::ApiRouter;
use axum::Extension;
use cda_comm_uds::UdsManager;
use cda_sovd::dynamic_router::DynamicRouter;

use crate::handlers::AppState;

pub async fn init_oem_routes<S>(
    dynamic_router: &DynamicRouter,
    uds: &UdsManager<
        cda_comm_can::MultiTransportGateway<
            cda_comm_doip::DoipDiagGateway<cda_core::EcuManager<S>>,
        >,
        cda_core::EcuManager<S>,
    >,
) where
    S: cda_plugin_security::SecurityPlugin + Send + Sync + 'static,
{
    let state = AppState::<S> { uds: uds.clone() };

    let router = ApiRouter::new()
        .route(
            "/vehicle/v15/components/{ecu}/x-sov2uds-my-company/dynamic-define-data",
            axum::routing::put::<_, _, _>(crate::handlers::dynamic_define_data_handler::<S>),
        )
        .route(
            "/vehicle/v15/components/{ecu}/x-sov2uds-my-company/periodic-read/{subfunction}",
            axum::routing::put::<_, _, _>(crate::handlers::periodic_read_handler::<S>),
        )
        .layer(Extension(state));

    dynamic_router.add_routes(router).await;

    tracing::info!(
        "OEM routes registered under /vehicle/v15/components/{{ecu}}/x-sov2uds-my-company/*"
    );
}
