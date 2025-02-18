use std::collections::HashMap;
use error_stack::ResultExt;
use health_check::{health_check_response::ServingStatus, health_client::HealthClient, HealthCheckRequest, HealthCheckResponse};
use hyper_util::client::legacy::connect::HttpConnector;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::Bytes;

pub mod health_check {
    tonic::include_proto!("grpc.health.v1");
}
pub type Client = hyper_util::client::legacy::Client<HttpConnector, UnsyncBoxBody<Bytes, tonic::Status>>;
pub type HealthCheckMap = HashMap<HealthCheckServices, bool>;
pub type HealthCheckResult<T> = CustomResult<T, HealthCheckError>;
pub type CustomResult<T, E> = error_stack::Result<T, E>;
#[derive(Debug, Clone)]
pub struct HealthCheckClient {
    /// Health clients for all gRPC based services
    pub clients: HashMap<HealthCheckServices, HealthClient<Client>>,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthCheckServices {
    /// Dynamic routing service
    DynamicRoutingService,
}
#[derive(Debug, Clone, thiserror::Error)]
pub enum HealthCheckError {
    /// The required input is missing
    #[error("Missing fields: {0} for building the Health check connection")]
    MissingFields(String),
    /// Error from gRPC Server
    #[error("Error from gRPC Server : {0}")]
    ConnectionError(String),
    /// status is invalid
    #[error("Invalid Status from server")]
    InvalidStatus,
}
pub async fn build_connections(client: Client) -> Result<HealthCheckClient, Box<dyn std::error::Error>> {
    let connection = Some(("127.0.0.1".to_string(), 8000u16, "dynamo".to_string()));

    let mut client_map = HashMap::new();

    if let Some(conn) = connection {
        let uri = format!("http://{}:{}", conn.0, conn.1).parse::<tonic::transport::Uri>()?;
        let health_client = HealthClient::with_origin(client, uri);

        client_map.insert(HealthCheckServices::DynamicRoutingService, health_client);
    }

    Ok(HealthCheckClient {
        clients: client_map,
    })
}

pub async fn perform_health_check(health_client: &HealthCheckClient) -> HealthCheckResult<HealthCheckMap> {
    let connection = ("127.0.0.1".to_string(), 8000u16, "dynamo".to_string());

    let health_client = health_client.clients.get(&HealthCheckServices::DynamicRoutingService);

    #[allow(clippy::as_conversions)]
    let expected_status = ServingStatus::Serving as i32;

    let mut service_map = HealthCheckMap::new();
    println!("Performing health check on Dynamic Routing Service");
    // let health_check_succeed = connection
    //         .as_ref()
    //         .async_map(|conn| get_response_from_grpc_service(conn.2.clone(), health_client))
            // .await
            // .transpose()
            // .change_context(HealthCheckError::ConnectionError(
            //     "error calling dynamic routing service".to_string(),
            // ))
            // // .map_err(|err)
            // .ok()
            // .flatten()
            // .is_some_and(|resp| resp.status == expected_status);
        let health_check_succeed = get_response_from_grpc_service(connection.2.clone(), health_client)
            .await.unwrap();

        // connection.and_then(|_conn| {
            service_map.insert(
                HealthCheckServices::DynamicRoutingService,
                true,
            );
        // });

    Ok(service_map)
}

async fn get_response_from_grpc_service(
    service: String,
    client: Option<&HealthClient<Client>>
) -> HealthCheckResult<HealthCheckResponse> {
    let request = tonic::Request::new(HealthCheckRequest { service});

    let mut client = client
        .ok_or(HealthCheckError::MissingFields(
            "[health_client]".to_string(),
        ))?
        .clone();

    let response = client
        .check(request)
        .await
        .change_context(HealthCheckError::ConnectionError(
            "Failed to call dynamic routing service".to_string(),
        ))?
        .into_inner();
    println!("Response from gRPC Server: {:?}", response);

    Ok(response)
}