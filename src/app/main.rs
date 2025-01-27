use anyhow::Result;
use testing_framework::{types::config::Config, simulate::user::Sampler};

fn generate_user_sample(config: &Config) -> Result<()> {
    let output = config.user.generate_sample()?;
    let output = serde_json::to_string_pretty(&output)?;
    println!("{}", output);
    Ok(())
}

fn main() -> Result<()> {
    let config = Config::load()?;

    let _user_sample = generate_user_sample(&config)?;

    

    Ok(())
}