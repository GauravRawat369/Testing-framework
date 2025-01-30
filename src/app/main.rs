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

fn main() -> Result<()> {
    let config = Config::load()?;

    // println!("User config: {:?}", config.user);

    let (user_sample, connectors) = generate_user_sample(&config)?;
    // println!("Available connectors for this user: {:?}", connectors);
    println!("Available connectors for this user:");
    for connector in &connectors {
        println!("{}", connector.0);
    }
    //merchant and user config will give this list of connectors
    // let connectors = vec![Key("stripe".to_string()), Key("paypal".to_string()), Key("adyen".to_string())];

    // let connectors = find_suitable_connectors(user_sample, &config.merchant);
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