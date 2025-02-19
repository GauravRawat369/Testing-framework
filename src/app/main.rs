use std::collections::HashMap;

use anyhow::Result;
use testing_framework::grpc_clients::health_check_client::{build_connections, perform_health_check};
use testing_framework::grpc_clients::success_rate_client::success_rate::{LabelWithScore, LabelWithStatus};
use testing_framework::grpc_clients::GrpcHeaders;
use testing_framework::{types::Config, sampler::Sampler};
use testing_framework::types::{find_suitable_connectors, Key, PaymentRecorderData, Status, StraightThroughRouting};
use testing_framework::evaluator::Evaluator;
use testing_framework::recorder::Recorder;
use testing_framework::types::Metrics;
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

fn generate_user_sample(config: &Config) -> Result<(String, String, Vec<Key>)> {
    let output = config.user.generate_sample()?;
    let connectors = find_suitable_connectors(&output, &config.merchant);
    let params = flatten_payment_info(output.clone());
    let output = serde_json::to_string_pretty(&output)?;
    println!("User sample: {}", output);
    Ok((params, output, connectors))
}

fn flatten_payment_info(payment_info: HashMap<Key, Key>) -> String {
    // Assuming we know the keys we're interested in, like "payment_methods", "payment_method_type", and "currency"
    let payment_methods = payment_info.get(&Key("payment_methods".to_string())).map(|k| &k.0).unwrap();
    let currency = payment_info.get(&Key("currency".to_string())).map(|k| &k.0).unwrap();
    let payment_method_type = payment_info.get(&Key("payment_method_type".to_string())).map(|k| &k.0);
    match payment_method_type {
        Some(pmt) => {
            return format!(
                "id:{}:{}:{}",
                payment_methods, pmt, currency
            )
        }
        None => {
            return format!(
                "id:{}:{}",
                payment_methods, currency
            )
        }
    }
}

fn convert_to_label_with_status(connector: String, res: &Result<Status>) -> Vec<LabelWithStatus> {
    let status = match res {
        Ok(Status::Success) => true,
        Ok(Status::Failure) => false,
        Err(_) => false
    };
    vec![LabelWithStatus {
        label: connector,
        status,
    }]
}

async fn setup() -> Result<(Config, RoutingStrategy)> {
    let config = Config::load()?;

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

    Ok((config, dynamic_routing_connection))
}

async fn call_script(config: &Config, dynamic_routing_connection: RoutingStrategy, metrics: &mut Metrics) -> Result<()> {

    let (params, user_sample, connectors) = generate_user_sample(&config)?;

    if connectors.is_empty() {
        println!("No connectors available for this user in merchant config.");
        return Ok(())
    }
    println!("Available connectors for this user:");
    for connector in &connectors {
        println!("{}", connector.0);
    }

    let success_rate_client: SuccessRateCalculatorClient<Client> = dynamic_routing_connection.success_rate_client.unwrap();

    let success_based_routing_config = SuccessBasedRoutingConfig {
        min_aggregates_size: Some(2),
        default_success_rate: Some(100.0),
        max_aggregates_size: Some(3),
        current_block_threshold: Some(CurrentBlockThreshold{
            duration_in_mins: Some(5),
            max_total_count: Some(2)
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
            success_based_routing_config.clone(),
            params.clone(),
            connectors.clone(),
            headers.clone(),
        )
        .await
        .expect("Failed to calculate success rate");
    // let routing = StraightThroughRouting {connectors};
    // let connector = routing.get_connector(); // Get the connector name as a string

    let connector = Key(get_highest_score_connector(success_based_connectors.labels_with_score).unwrap().clone());

    println!("Using connector: {:?}", connector.0);
    let res = config.psp.call_evaluator(&connector, &user_sample);

    let connector_list_with_status = convert_to_label_with_status(connector.0.clone(), &res);

    success_rate_client
        .update_success_rate(
            "id".to_string(),
            success_based_routing_config,
            params.clone(),
            connector_list_with_status,
            headers,
        )
        .await
        .expect("Failed to update success rate");   

    match &res? {
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
    let (config, dynamic_routing_connection) = setup().await?;

    let mut metrics = Metrics::new();
    for _ in 0..100 {
        call_script(&config, dynamic_routing_connection.clone(), &mut metrics).await?;
    }
    // Use recorder to print metrics
    testing_framework::recorder::print_metrics(&metrics);

    let headers = GrpcHeaders {
        tenant_id: "tenant_id".to_string(),
        request_id: Some("request_id".to_string()),
    };

    let success_rate_client = dynamic_routing_connection.success_rate_client.unwrap();
    success_rate_client.invalidate_redis_keys("id".to_string(), headers);
    Ok(())
}