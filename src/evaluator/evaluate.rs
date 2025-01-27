use anyhow::Result;
use testing_framework::types::config::{Status, PspSimulationConfig, Config};
use rand::Rng;
use testing_framework::recorder::record::Recorder;

// Evaluator trait for validation
pub trait Evaluator {
    fn call_evaluator(
        &self,
        connector: &str,
    ) -> Result<Status>;
}

// Implement Evaluator for PspSimulationConfig
impl Evaluator for PspSimulationConfig {
    fn call_evaluator(
        &self,
        connector: &str,
    ) -> Result<Status> {
        let mut rng = rand::thread_rng();

        // Find the connector configuration
        if let Some(config) = self.config.get(connector) {
            let success = rng.gen_bool(config.sr as f64 / 100.0);
            return Ok(if success { Status::Success } else { Status::Failure });
        }
        Ok(self.default_status())
    }
}

// Example usage
fn main() -> Result<()> {
    let config = Config::load()?;
    let connector = "stripe"; // Example connector name
    println!("Loaded configuration: {:?}", config.psp);
    match config.psp.call_evaluator(connector)? {
        Status::Success => {
            println!("Transaction succeeded.");
            Config::record_transaction("stripe", Status::Success, &config.user, &config.psp)?;
        },
        Status::Failure => {
            println!("Transaction failed.");
            Config::record_transaction("stripe", Status::Failure, &config.user, &config.psp)?;
        },
    }

    Ok(())
}

