use anyhow::Result;
use crate::types::config::{Key, Possible, PspSimulationConfig, Status};
use rand::Rng;


pub trait Evaluator {
    fn call_evaluator(
        &self,
        connector: &Key,
        user_sample: &str,
    ) -> Result<Status>;
}

impl Evaluator for PspSimulationConfig {
    fn call_evaluator(
        &self,
        connector: &Key,
        user_sample: &str,
    ) -> Result<Status> {
        let mut rng = rand::thread_rng();

        if let Some(config) = self.config.get(&connector.0) {
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



