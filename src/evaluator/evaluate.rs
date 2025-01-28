use anyhow::Result;
use crate::types::config::{PspSimulationConfig, Status, Possible};
use rand::Rng;
// Evaluator trait for validation

pub trait Evaluator {
    fn call_evaluator(
        &self,
        connector: &str,
        user_sample: &str,
    ) -> Result<Status>;
}

// Implement Evaluator for PspSimulationConfig
impl Evaluator for PspSimulationConfig {
    fn call_evaluator(
        &self,
        connector: &str,
        user_sample: &str,
    ) -> Result<Status> {
        let mut rng = rand::thread_rng();

        // Find the connector configuration
        if let Some(config) = self.config.get(connector) {
            // Check if user sample matches the sample inside the config
            let matches = config.key.iter().all(|(key, possible)| {
                // println!("possible value is : {:?}", key.0);
                match possible {
                    Possible::Value(value) => {
                        // println!("value is : {:?}", value.0);
                        if value.0 == "*" {
                            true
                        } else {
                            user_sample.contains(&value.0)
                        }
                    }
                    Possible::Pattern(pattern) => {
                        // println!("pattern value is : {:?}", pattern.0);
                        if pattern.0 == "*" {
                            true
                        } else {
                            user_sample.contains(&pattern.0)
                        }
                    },
                }
            });

            if matches {
                let success = rng.gen_bool(config.sr as f64 / 100.0);
                return Ok(if success { Status::Success } else { Status::Failure });
            }
        }
        Ok(self.default_status())
    }
}



