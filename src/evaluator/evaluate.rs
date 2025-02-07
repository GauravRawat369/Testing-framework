use anyhow::Result;
use crate::types::config::{Key, PspSimulationConfig, Status,};
use crate::types::config::PaymentRecorderData;
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
            for payment_method_config in &config.key {
                let matches = user_sample.contains(&payment_method_config.payment_method.0)
                    && (user_sample.contains(&payment_method_config.payment_method_type.0) || payment_method_config.payment_method_type.0 == "*");

                if matches {
                    let success = rng.gen_bool(payment_method_config.sr as f64 / 100.0);
                    return Ok(if success { Status::Success } else { Status::Failure });
                }
            }
        }
        Ok(self.default_status())
    }
}



