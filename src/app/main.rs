use anyhow::Result;
use testing_framework::{types::config::Config, simulate::user::Sampler};
use testing_framework::types::config::{find_suitable_connectors, Key, Status, StraightThroughRouting, PaymentRecorderData};
use testing_framework::evaluator::evaluate::Evaluator;
use testing_framework::recorder::record::Recorder;

fn generate_user_sample(config: &Config) -> Result<(String, Vec<Key>)> {
    let output = config.user.generate_sample()?;
    let connectors = find_suitable_connectors(&output, &config.merchant);
    let output = serde_json::to_string_pretty(&output)?;
    println!("Simulated User Sample: {}", output);
    Ok((output, connectors))
}
fn call_script() -> Result<()>{
    let config = Config::load()?;
    let (user_sample, connectors) = generate_user_sample(&config)?;
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
            record_data.record_transaction()?;
        },
        Status::Failure => {
            println!("Transaction failed.");
            // Call recorder
            let record_data = PaymentRecorderData::set_values(connector.clone(), Status::Failure, Key(user_sample.clone()));
            record_data.record_transaction()?;
        },
    }

    Ok(())
}
fn main() -> Result<()> {
    
    for _ in 0..1500 {
        call_script()?;
    }
    Ok(())
}