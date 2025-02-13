use anyhow::Result;
use std::env;
use testing_framework::mab::thompson_sampling::ThompsonSampling;
use testing_framework::mab::sliding_window_ucb::SlidingWindowUCB;
use testing_framework::{types::config::Config, simulate::user::Sampler};
use testing_framework::types::config::{find_suitable_connectors, Key, PaymentRecorderData, Status, StraightThroughRouting};
use testing_framework::evaluator::evaluate::Evaluator;
use testing_framework::recorder::record::Recorder;
use testing_framework::types::config::Metrics;
use testing_framework::types::config::{PaymentConnector, RoutingAlgorithm};
use std::collections::HashMap;



fn generate_user_sample(config: &Config) -> Result<(String, Vec<Key>)> {
    let output = config.user.generate_sample()?;
    let connectors = find_suitable_connectors(&output, &config.merchant);
    let output = serde_json::to_string_pretty(&output)?;
    Ok((output, connectors))
}
fn call_script_for_mab(metrics: &mut Metrics, algorithm: &mut Box<dyn RoutingAlgorithm>, map_connector: &mut HashMap<String, bool>, connectors: &mut Vec<PaymentConnector>
) -> Result<()> {
    // Load the configuration
    let config = Config::load()?;
    let (user_sample, eligible_connector) = generate_user_sample(&config)?;
    println!("User sample: {}", user_sample);
    if eligible_connector.is_empty() {
        println!("No eligible_connector available for this user in merchant config.");
        return Ok(());
    }
    println!("Available eligible_connector for this user:");
    for connector in &eligible_connector {
        println!("{}", connector.0);
    }
    //check if the eligible_connector is available in the PaymentConnector if not create one
    for connector in &eligible_connector {
        if !map_connector.contains_key(&connector.0) {
            map_connector.insert(connector.0.clone(), true);
            connectors.push(PaymentConnector::new(connector.0.clone(), 5));
        }
    }

   //plug algo
   let connector_index = algorithm.select_connector(connectors);
   let connector_name = connectors[connector_index].name.clone();
    let connector = Key(connector_name);

    

    println!("Using connector: {:?}", connector.0);
    match config.psp.call_evaluator(&connector, &user_sample)? {
        Status::Success => {
            println!("Transaction succeeded.");
            //give feadback to the algorithm
            algorithm.update_connector(connectors, connector_index, true);
            // Call recorder
            let record_data = PaymentRecorderData::set_values(connector.clone(), Status::Success, Key(user_sample.clone()));
            record_data.record_transaction(metrics)?;
        },
        Status::Failure => {
            println!("Transaction failed.");
            //give feadback to the algorithm
            algorithm.update_connector(connectors, connector_index, false);
            // Call recorder
            let record_data = PaymentRecorderData::set_values(connector.clone(), Status::Failure, Key(user_sample.clone()));
            record_data.record_transaction(metrics)?;
        },
    }

    Ok(())
}
fn call_script_for_straight_through(metrics: &mut Metrics) -> Result<()> {
    // Load the configuration
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
   //plug algo
    let routing = StraightThroughRouting {connectors};
    let connector = routing.get_connector(); // Get the connector name as a string


    println!("Using connector: {:?}", connector.0);
    match config.psp.call_evaluator(&connector, &user_sample)? {
        Status::Success => {
            println!("Transaction succeeded.");
            //give feadback to the algorithm
            // Call recorder
            let record_data = PaymentRecorderData::set_values(connector.clone(), Status::Success, Key(user_sample.clone()));
            record_data.record_transaction(metrics)?;
        },
        Status::Failure => {
            println!("Transaction failed.");
            //give feadback to the algorithm
            // Call recorder
            let record_data = PaymentRecorderData::set_values(connector.clone(), Status::Failure, Key(user_sample.clone()));
            record_data.record_transaction(metrics)?;
        },
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Available algorithms: thompson, ucb");
        eprintln!("Run cargo run -- thompson/cargo run -- ucb");
        return;
    }

    // Choose algorithm based on user input
    let algorithm_name = &args[1];
    let mut algorithm: Box<dyn RoutingAlgorithm> = match algorithm_name.as_str() {
        "thompson" => Box::new(ThompsonSampling::new(0.5)), // Discount factor = 0.95
        "ucb" => Box::new(SlidingWindowUCB::new(5, 2.0)),     // Window size = 5, exploration factor = 2.0
        _ => {
            eprintln!("Invalid algorithm. Available options: thompson, ucb");
            return;
        }
    };

    let mut metrics = Metrics::new();
    let mut connectors: Vec<PaymentConnector> = vec![];
    let mut map_connector :HashMap<String, bool>  = HashMap::new();
    for _ in 0..1000 {
        // call_script_for_straight_through(&mut metrics);
        call_script_for_mab(&mut metrics,&mut algorithm,&mut map_connector,&mut connectors);
    }
    // Use recorder to print metrics
    testing_framework::recorder::record::print_metrics(&metrics);
}