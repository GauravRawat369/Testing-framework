use anyhow::Result;
use testing_framework::{types::config::Config, simulate::user::Sampler};
use testing_framework::types::config::{Key, Status, StraightThroughRouting, Payment_Recorder_data};
use testing_framework::evaluator::evaluate::Evaluator;
use testing_framework::recorder::record::Recorder;

fn generate_user_sample(config: &Config) -> Result<String> {
    let output = config.user.generate_sample()?;
    let output = serde_json::to_string_pretty(&output)?;
    println!("{}", output);
    Ok(output)
}

fn main() -> Result<()> {
    let config = Config::load()?;
    let user_sample = generate_user_sample(&config)?;
    // Merchant and user config will give this list of connectors
    let connectors = vec![Key("stripe".to_string()), Key("paypal".to_string()), Key("adyen".to_string())];
    let routing = StraightThroughRouting { connectors };
    let connector = routing.get_connector(); // Get the connector name as a string
    println!("Using connector: {:?}", connector.0);

    match config.psp.call_evaluator(&connector, &user_sample)? {
        Status::Success => {
            println!("Transaction succeeded.");
            // Call recorder
            let record_data = Payment_Recorder_data::set_values(connector.clone(), Status::Success, Key(user_sample.clone()));
            record_data.record_transaction()?;
        },
        Status::Failure => {
            println!("Transaction failed.");
            // Call recorder
            let record_data = Payment_Recorder_data::set_values(connector.clone(), Status::Failure, Key(user_sample.clone()));
            record_data.record_transaction()?;
        },
    }

    Ok(())
}