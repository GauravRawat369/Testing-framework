use anyhow::Result;
use testing_framework::{types::config::Config, simulate::user::Sampler};
use testing_framework::types::config:: Status;
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
    //algo will give us connector name
    let connector = "stripe"; // Example connector name
    match config.psp.call_evaluator(connector, &user_sample)? {
        Status::Success => {
            println!("Transaction succeeded.");
            // Call recorder
            Config::record_transaction("stripe", Status::Success, &config.user, &config.psp)?;
        },
        Status::Failure => {
            println!("Transaction failed.");
            // Call recorder
            Config::record_transaction("stripe", Status::Failure, &config.user, &config.psp)?;
        },
    }

    Ok(())
}