use anyhow::Result;
use testing_framework::grpc_clients::health_check_client::{build_connections, perform_health_check};
use testing_framework::grpc_clients::success_rate_client::success_rate::LabelWithScore;
use testing_framework::grpc_clients::GrpcHeaders;
use testing_framework::{types::Config, sampler::Sampler};
use testing_framework::types::{find_suitable_connectors, Key, PaymentRecorderData, Status, StraightThroughRouting};
use testing_framework::evaluator::Evaluator;
use testing_framework::recorder::Recorder;
use testing_framework::types::Metrics;
// use testing_framework::grpc_clients::success_rate_client::SuccesBasedDynamicRouting;
use testing_framework::grpc_clients::success_rate_client::{CurrentBlockThreshold, SuccessBasedDynamicRouting, SuccessBasedRoutingConfig};
use hyper_util::client::legacy::connect::HttpConnector;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::Bytes;

pub mod health_check {
    tonic::include_proto!("grpc.health.v1");
}
pub mod success_rate {
    tonic::include_proto!("success_rate");
}

use testing_framework::grpc_clients::success_rate_client::success_rate::success_rate_calculator_client::SuccessRateCalculatorClient;

pub type Client = hyper_util::client::legacy::Client<HttpConnector, UnsyncBoxBody<Bytes, tonic::Status>>;

fn generate_user_sample(config: &Config) -> Result<(String, Vec<Key>)> {
    let output = config.user.generate_sample()?;
    let connectors = find_suitable_connectors(&output, &config.merchant);
    let output = serde_json::to_string_pretty(&output)?;
    Ok((output, connectors))
}

// #[async_trait::async_trait]
// impl SuccessBasedDynamicRouting for SuccessRateCalculatorClient<Client> {
//     async fn calculate_success_rate(
//         &self,
//         id: String,
//         success_rate_based_config: SuccessBasedRoutingConfig,
//         params: String,
//         connector_list: Vec<Key>,
//         headers: GrpcHeaders
//     ) -> DynamicRoutingResult<CalSuccessRateResponse> {
//         let connector_list = connector_list.into_iter().map(|key| key.0).collect();
    
//         let config = foreign_try_from(success_rate_based_config).map_err(|err| {
//             DynamicRoutingError::SuccessRateBasedRoutingFailure(err.to_string())
//         })?;
    
//         let request = create_grpc_request(
//             CalSuccessRateRequest {
//                 id,
//                 params,
//                 labels: connector_list,
//                 config: Some(config),
//             },
//             headers,
//         );
    
//         let response = self
//             .clone()
//             .fetch_success_rate(request)
//             .await
//             .change_context(DynamicRoutingError::SuccessRateBasedRoutingFailure(
//                 "Error while fetching success rate".to_string(),
//             ))?
//             .into_inner();
    
//         Ok(response)
//     }
// }

async fn call_script(metrics: &mut Metrics) -> Result<()> {

    let config = Config::load()?;
    let (user_sample, connectors) = generate_user_sample(&config)?;
    println!("User sample: {}", user_sample);
    if connectors.is_empty() {
        println!("No connectors available for this user in merchant config.");
        return Ok(());
    }
    println!("Available connectors for this user:");
    for connector in &connectors {
        println!("{}", connector.0);
    }

    let client =
    hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .http2_only(true)
        .build_http();

        let health_client = build_connections(client.clone())
        .await
        .expect("Failed to build gRPC connections");

    println!("Performing health check on all services");

    perform_health_check(&health_client)
        .await
        .expect("Failed to perform health check");

    println!("Health check successful");

    let dynamic_routing_connection = get_dynamic_routing_connection(client.clone())
        .await
        .expect("Failed to establish a connection with the Dynamic Routing Server");

    println!("{:?}", dynamic_routing_connection);

    let success_rate_client: SuccessRateCalculatorClient<Client> = dynamic_routing_connection.success_rate_client.unwrap();

    let success_based_routing_config = SuccessBasedRoutingConfig {
        min_aggregates_size: Some(2),
        default_success_rate: Some(100.0),
        max_aggregates_size: Some(10),
        current_block_threshold: Some(CurrentBlockThreshold{
            duration_in_mins: Some(5),
            max_total_count: Some(10)
        }),
        specificity_level: testing_framework::grpc_clients::success_rate_client::SuccessRateSpecificityLevel::Merchant
    };

    let headers = GrpcHeaders {
        tenant_id: "tenant_id".to_string(),
        request_id: Some("request_id".to_string()),
    };

    let success_based_connectors = success_rate_client
        .calculate_success_rate(
            "id".to_string(),
            success_based_routing_config,
            user_sample.clone(),
            connectors.clone(),
            headers,
        )
        .await
        .expect("Failed to calculate success rate");
    // let routing = StraightThroughRouting {connectors};
    // let connector = routing.get_connector(); // Get the connector name as a string

    let connector = Key(get_highest_score_connector(success_based_connectors.labels_with_score).unwrap().clone());

    println!("Using connector: {:?}", connector.0);
    match config.psp.call_evaluator(&connector, &user_sample)? {
        Status::Success => {
            println!("Transaction succeeded.");
            // Call recorder
            let record_data = PaymentRecorderData::set_values(connector.clone(), Status::Success, Key(user_sample.clone()));
            record_data.record_transaction(metrics)?;
        },
        Status::Failure => {
            println!("Transaction failed.");
            // Call recorder
            let record_data = PaymentRecorderData::set_values(connector.clone(), Status::Failure, Key(user_sample.clone()));
            record_data.record_transaction(metrics)?;
        },
    }

    Ok(())
}

fn get_highest_score_connector(labels_with_score: Vec<LabelWithScore>) -> Option<String> {
    if labels_with_score.is_empty() {
        return None;
    }

    let highest_score_label = labels_with_score
        .iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));

    highest_score_label.map(|label_with_score| label_with_score.label.clone())
}

/// Type that consists of all the services provided by the client
#[derive(Debug, Clone)]
pub struct RoutingStrategy {
    /// success rate service for Dynamic Routing
    pub success_rate_client: Option<SuccessRateCalculatorClient<Client>>,
    /// contract based routing service for Dynamic Routing
    pub contract_based_client: Option<SuccessRateCalculatorClient<Client>>,
    /// elimination service for Dynamic Routing
    pub elimination_based_client: Option<SuccessRateCalculatorClient<Client>>,
}

pub async fn get_dynamic_routing_connection(        client: Client,
)     -> Result<RoutingStrategy, Box<dyn std::error::Error>> {


    let uri = format!("http://{}:{}","127.0.0.1", "8000").parse::<tonic::transport::Uri>()?;

    let success_rate_client = Some(SuccessRateCalculatorClient::with_origin(
        client.clone(),
        uri.clone(),
    ));

    Ok(RoutingStrategy {
        success_rate_client,
        contract_based_client: None,
        elimination_based_client: None,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut metrics = Metrics::new();
    for _ in 0..10 {
        call_script(&mut metrics).await?;
    }
    // Use recorder to print metrics
    testing_framework::recorder::print_metrics(&metrics);
    Ok(())
}