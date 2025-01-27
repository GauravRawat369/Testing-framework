use crate::types::config::{Key, Parameters, SimulationConfig};
use anyhow::Result;
use rand::Rng;
use std::collections::HashMap;

pub trait Sampler {
    fn generate_sample(&self) -> Result<HashMap<&Key, &Key>>;

    fn list_parameters(config: &SimulationConfig) -> Result<HashMap<&Key, &Key>> {
        config
            .iter()
            .try_fold(HashMap::new(), |mut acc, (key, param)| {
                let (value, next) = Self::choose_parameter(param)?;
                acc.insert(key, value);
                if let Some(next) = next {
                    let next = Self::list_parameters(next)?;
                    acc.extend(next);
                }
                Ok(acc)
            })
    }

    fn choose_parameter(param: &Parameters) -> Result<(&Key, Option<&SimulationConfig>)> {
        let mut rng = rand::thread_rng();
        let mut number = rng.gen_range(0..100);
        let variants = param.iter().fold(None,|acc, (key, info)| {
            if acc.is_some() {
                acc
            } else {
                match info {
                    crate::types::config::ParameterConfig::Percentage(val) => {
                        if number < *val {
                            Some((key, None))
                        } else {
                            number -= val;
                            None
                        }
                    }
                    crate::types::config::ParameterConfig::Composite { percentage, next } => {
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
        let output = variants.ok_or_else(|| anyhow::anyhow!("No parameter found"))?;
        Ok(output)
    }
}

