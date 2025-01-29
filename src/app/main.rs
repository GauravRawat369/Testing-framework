use anyhow::Result;
use testing_framework::{types::config::Config, simulate::user::Sampler};
use testing_framework::types::config::{Key, Status, StraightThroughRouting};
use testing_framework::recorder::record::Recorder;
use testing_framework::evaluator::evaluate::Evaluator;

fn generate_user_sample(config: &Config) -> Result<String> {
    let output = config.user.generate_sample()?;
    let output = serde_json::to_string_pretty(&output)?;
    println!("{}", output);
    Ok(output)
}

fn main() -> Result<()> {
    let config = Config::load()?;
    let user_sample = generate_user_sample(&config)?;
    //merchant and user config will give this list of connectors
    let connectors = vec![Key("stripe".to_string()), Key("paypal".to_string()), Key("adyen".to_string())];
    let routing = StraightThroughRouting {connectors};
    let connector = routing.get_connector(); // Get the connector name as a string
    println!("Using connector: {:?}", connector.0);
    // let connector = Key("stripe".to_string()); // Hardcoded for now

    match config.psp.call_evaluator(&connector, &user_sample)? {
        Status::Success => {
            println!("Transaction succeeded.");
            //feedback

            // Call recorder
            Config::record_transaction(&connector, Status::Success, &config.user, &config.psp)?;
        },
        Status::Failure => {
            println!("Transaction failed.");
            //feedback
            
            // Call recorder
            Config::record_transaction(&connector, Status::Failure, &config.user, &config.psp)?;
        },
    }

    Ok(())
}