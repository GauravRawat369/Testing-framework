use std::collections::HashMap;
use crate::types::config::{AmountRange, Key, PaymentMethods, SimulationConfig};
use anyhow::{anyhow, Result};
use rand::Rng;

pub trait Sampler {
    fn generate_sample(&self) -> Result<HashMap<Key, Key>>;

    fn list_payment_methods(config: &SimulationConfig) -> Result<HashMap<&Key, &Key>> {
        config
            .iter()
            .try_fold(HashMap::new(), |mut acc, (key, payment_method)| {
                let (value, next) = Self::choose_payment_method(payment_method)?;
                acc.insert(key, value);
                if let Some(next) = next {
                    let next = Self::list_payment_methods(next)?;
                    acc.extend(next);
                }
                Ok(acc)
        })
    }

    fn choose_payment_method(payment_method: &PaymentMethods) -> Result<(&Key, Option<&SimulationConfig>)> {
        let mut rng = rand::thread_rng();
        let mut number = rng.gen_range(0..100);
        let variants = payment_method.iter().fold(None, |acc, (key, info)| {
            if acc.is_some() {
                acc
            } else {
                match info {
                    crate::types::config::PaymentMethodDetails::Percentage(val) => {
                        if number < *val {
                            Some((key, None))
                        } else {
                            number -= val;
                            None
                        }
                    }
                    crate::types::config::PaymentMethodDetails::Composite { percentage, next, extra_fields: _ } => {
                        if number < *percentage {
                            Some((key, Some(next)))
                        } else {
                            number -= percentage;
                            None
                        }
                    }
                }
            }
        });
        let output = variants.ok_or_else(|| anyhow!("No payment_method found"))?;
        Ok(output)
    }

    fn generate_random_amount(amount_range: &Option<AmountRange>) -> u32 {
        let range = amount_range.as_ref().unwrap_or(&AmountRange { min: 0, max: 2000 });
        let mut rng = rand::thread_rng();
        rng.gen_range(range.min..=range.max)
    }
}