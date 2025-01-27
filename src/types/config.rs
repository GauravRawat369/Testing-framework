use serde::{Deserialize, Serialize};
use core::ops::Deref;
use std::collections::HashMap;
use crate::simulate::user::Sampler;
use anyhow::{ensure, Context, Result};
use rand::Rng;
use std::fs;

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


#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)] // For supporting values and wildcard patterns
pub enum Possible {
    Value(Key),
    Pattern(String),
}

// Status enum for the transaction result
#[derive(Debug)]
pub enum Status {
    Success,
    Failure,
}

// Configuration for a single connector
#[derive(Debug, Deserialize)]
pub struct ConnectorConfig {
    pub key: HashMap<Key, Possible>,
    pub sr: u8, // Success rate
}

// Main PSP configuration loaded from JSON
#[derive(Debug, Deserialize)]
pub struct PspSimulationConfig {
    pub config: HashMap<String, ConnectorConfig>,
    pub otherwise: String, // Default result as a string
}

// Convert String into Status for easy mapping
impl PspSimulationConfig {
    pub fn default_status(&self) -> Status {
        match self.otherwise.as_str() {
            "success" => Status::Success,
            _ => Status::Failure,
        }
    }
}

// Evaluator trait for validation
pub trait Evaluator {
    fn validate_parameters(
        &self,
        connector: &str,
    ) -> Result<Status>;
}
pub trait Recorder {
    fn record_transaction(
        &self,
        connector: &str,
        verdict: Status,
    ) -> Result<()>;
    
}

// Implement Evaluator for PspSimulationConfig
impl Evaluator for PspSimulationConfig {
    fn validate_parameters(
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

// Function to load the configuration from input.json
pub fn load_config(file_path: &str) -> Result<PspSimulationConfig> {
    let json_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read configuration file: {}", file_path))?;
    
    // Extract only the "psp" part of the JSON
    let json_value: serde_json::Value = serde_json::from_str(&json_content)
        .with_context(|| "Failed to parse JSON configuration")?;
    let psp_json = json_value.get("psp")
        .with_context(|| "Failed to find 'psp' in JSON configuration")?;
    // println!("Extracted 'psp' JSON: {}", psp_json);
    
    let config: PspSimulationConfig = serde_json::from_value(psp_json.clone())
        .with_context(|| "Failed to parse 'psp' JSON configuration")?;
    // println!("Parsed configuration: {:?}", config);
    Ok(config)
}

