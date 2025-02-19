use anyhow::Result;
use crate::config::{Key, PspSimulationConfig, Status,};
use crate::config::PaymentRecorderData;
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

        if let Some(config) = self.psp_variants.get(&Key(connector.0.clone())) {
            // Iterate over each payment method and handle both variants
            for (pm_key, pm_value) in &config.payment_methods {
                match pm_value {
                    crate::config::PaymentMethodTypes::PaymentTypes(details) => {
                        for detail in details {
                            if user_sample.contains(&pm_key.0) &&
                               (user_sample.contains(&detail.payment_method_type.0) || detail.payment_method_type.0 == "*")
                            {
                                let success = rng.gen_bool(detail.sr as f64 / 100.0);
                                return Ok(if success { Status::Success } else { Status::Failure });
                            }
                        }
                    },
                    crate::config::PaymentMethodTypes::Simple { sr } => {
                        if user_sample.contains(&pm_key.0) {
                            let success = rng.gen_bool(*sr as f64 / 100.0);
                            return Ok(if success { Status::Success } else { Status::Failure });
                        }
                    }
                }
            }
            // If no matching payment method is found, return default status
            return Ok(self.default_status());
        }
        Ok(self.default_status())
    }
}



