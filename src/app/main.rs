use anyhow::Result;
use testing_framework::grpc_clients::health_check_client::{build_connections, perform_health_check};
use testing_framework::{types::Config, sampler::Sampler};
use testing_framework::types::{find_suitable_connectors, Key, PaymentRecorderData, Status, StraightThroughRouting};
use testing_framework::evaluator::Evaluator;
use testing_framework::recorder::Recorder;
use testing_framework::types::Metrics;
use hyper_util::client::legacy::connect::HttpConnector;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::Bytes;

pub mod health_check {
    tonic::include_proto!("grpc.health.v1");
}
pub mod success_rate {
    tonic::include_proto!("success_rate");
}

use crate::success_rate::success_rate_calculator_client::SuccessRateCalculatorClient;

pub type Client = hyper_util::client::legacy::Client<HttpConnector, UnsyncBoxBody<Bytes, tonic::Status>>;

fn generate_user_sample(config: &Config) -> Result<(String, Vec<Key>)> {
    let output = config.user.generate_sample()?;
    let connectors = find_suitable_connectors(&output, &config.merchant);
    let output = serde_json::to_string_pretty(&output)?;
    Ok((output, connectors))
}

fn call_script(metrics: &mut Metrics) -> Result<()> {
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

    let routing = StraightThroughRouting {connectors};
    let connector = routing.get_connector(); // Get the connector name as a string

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
    // let (success_rate_client, contract_based_client, elimination_based_client) = match self {
    //     Self::Enabled { host, port, .. } => {
    //         let uri = format!("http://{}:{}", host, port).parse::<tonic::transport::Uri>()?;
    //         logger::info!("Connection established with dynamic routing gRPC Server");
    //         (
    //             Some(),
    //             Some(ContractScoreCalculatorClient::with_origin(
    //                 client.clone(),
    //                 uri.clone(),
    //             )),
    //             Some(EliminationAnalyserClient::with_origin(client, uri)),
    //         )
    //     }
    //     Self::Disabled => (None, None, None),
    // };
    Ok(RoutingStrategy {
        success_rate_client,
        contract_based_client: None,
        elimination_based_client: None,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    // let mut metrics = Metrics::new();
    // for _ in 0..1 {
    //     call_script(&mut metrics)?;
    // }
    // // Use recorder to print metrics
    // testing_framework::recorder::print_metrics(&metrics);
    // Ok(())
    let client =
    hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .http2_only(true)
        .build_http();

    let dynamic_routing_connection = get_dynamic_routing_connection(client.clone())
        .await
        .expect("Failed to establish a connection with the Dynamic Routing Server");

    println!("{:?}", dynamic_routing_connection);

    let health_client = build_connections(client.clone())
        .await
        .expect("Failed to build gRPC connections");

    println!("Performing health check on all services");

    perform_health_check(&health_client)
        .await
        .expect("Failed to perform health check");

    println!("Health check successful");

    Ok(())
}