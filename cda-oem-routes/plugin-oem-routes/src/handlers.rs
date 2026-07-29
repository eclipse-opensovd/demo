use aide::axum::IntoApiResponse;
use axum::{
    Json,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};
use cda_comm_uds::UdsManager;
use cda_interfaces::{DynamicPlugin, HashMap};
use cda_plugin_security::{Secured, SecurityPlugin};
use schemars::JsonSchema;
use serde::Serialize;

pub struct AppState<S: SecurityPlugin> {
    pub uds: UdsManager<
        cda_comm_can::MultiTransportGateway<
            cda_comm_doip::DoipDiagGateway<cda_core::EcuManager<S>>,
        >,
        cda_core::EcuManager<S>,
    >,
}

impl<S: SecurityPlugin> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            uds: self.uds.clone(),
        }
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn dynamic_define_data_handler<S: SecurityPlugin + Send + Sync + 'static>(
    axum::Extension(state): axum::Extension<AppState<S>>,
    Path(ecu): Path<String>,
    Secured(security_plugin): Secured,
    Json(params): Json<HashMap<String, serde_json::Value>>,
) -> impl IntoApiResponse {
    let security_plugin: DynamicPlugin = security_plugin;

    // SID 0xBA - DynamicDefineData
    let result = state
        .uds
        .send_by_sid(&ecu, &[0xBA, 0x42 /*optional subfucntion id */], &security_plugin, params, true)
        .await;

    match result {
        Ok(response) => match response.serialize_to_json() {
            Ok(response) => (StatusCode::OK, Json(response.data)).into_response(),
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn periodic_read_handler<S: SecurityPlugin + Send + Sync + 'static>(
    axum::Extension(state): axum::Extension<AppState<S>>,
    Path((ecu, function_name)): Path<(String, String)>,
    Secured(security_plugin): Secured,
    Json(params): Json<HashMap<String, serde_json::Value>>,
) -> impl IntoApiResponse {
    let security_plugin: DynamicPlugin = security_plugin;

    // SID 0xBB - PeriodicReadDataByIdentifier
    // Subfunction path parameter used as the name for lookup_service_by_sid_and_name
    let result = state
        .uds
        .send_by_sid_and_name(
            &ecu,
            0xBB, // service id
            &function_name,
            &security_plugin,
            params,
            true,
        )
        .await;
    
    // ToDo: In this demo we will return the periodic response immediately, but in 
    // a real implementation we could process the response here further as well. 
    // Currently not support is to publish these in a event bus like websocket. 
    // You could start a task to call this repeatedly and publish the response to a websocket or other event bus (i.e. mqtt)

    match result {
        Ok(response) => match response.serialize_to_json() {
            Ok(response) => (StatusCode::OK, Json(response.data)).into_response(),
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}
