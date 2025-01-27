use serde::{Deserialize, Serialize};
use core::ops::Deref;
use std::collections::HashMap;
use crate::simulate::user::Sampler;
use anyhow::{ensure, Context, Result};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Hash)]
#[serde(transparent)]
pub struct Key(pub String);

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Parameters(HashMap<Key, ParameterConfig>);

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ParameterConfig {
    Percentage(u8),
    Composite {
        percentage: u8,
        next: SimulationConfig,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SimulationConfig(HashMap<Key, Parameters>);

#[derive(Debug, Deserialize, Serialize)]
pub struct UserSimulationConfig {
    #[serde(flatten)]
    pub parameters: SimulationConfig,
}

impl Sampler for UserSimulationConfig {
    fn generate_sample(&self) -> Result<HashMap<&Key, &Key>> {
        Self::list_parameters(&self.parameters)
    }
}

impl Parameters {
    pub fn validate(&self) -> Result<()> {
        let mut total = 0;
        for (_key, value) in self.0.iter() {
            match value {
                ParameterConfig::Percentage(value) => total += value,
                ParameterConfig::Composite { percentage, next } => {
                    total += percentage;
                    next.validate()?;
                }
            }
        }
        ensure!(total == 100, "Total percentage must be 100");
        Ok(())
    }
}

impl SimulationConfig {
    pub fn validate(&self) -> Result<()> {
        self.0.iter().try_for_each(|(key, value)| {
            value
                .validate()
                .context(format!("validation failed for: {}", key.0))?;
            Ok(())
        })
    }
}

impl UserSimulationConfig {
    pub fn validate(&self) -> Result<()> {
        self.parameters.validate()
    }
}

impl Deref for SimulationConfig {
    type Target = HashMap<Key, Parameters>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Parameters {
    type Target = HashMap<Key, ParameterConfig>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}