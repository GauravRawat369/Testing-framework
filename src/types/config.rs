use serde::{Deserialize, Serialize};
use core::ops::Deref;
use std::collections::HashMap;
use crate::simulate::user::Sampler;
use std::path::Path;
use anyhow::{ensure, Context, Result};
use rand::Rng;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub user: UserSimulationConfig,
    pub psp: PspSimulationConfig,
    pub merchant: MerchantConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        let default_path = Path::new("input.json");
        if default_path.exists() {
            let output = Self::load_from_path(default_path)?;
            output.user.validate()?;
            // println!("psp config {:?}: ", output.psp);
            return Ok(output);
        }

        anyhow::bail!("No config file found. Please provide it either in ./config.json or set `CONFIG_FILE` environment variable")
    }

    fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_str = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        serde_json::from_str(&config_str).with_context(|| "Failed to parse config file")
    }
}

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
    pub amount: AmountRange,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AmountRange { min: u64, max: u64}

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

#[derive(Debug, Eq, Deserialize, Serialize)]
#[serde(untagged)] 
pub enum Possible {
    Value(Key),
    Any,
}
impl PartialEq for Possible {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Possible::Any, _) => true,
            (_, Possible::Any) => true,
            (Possible::Value(a), Possible::Value(b)) => a == b,
        }
    }
}

// Status enum for the transaction result
#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum Status {
    Success,
    Failure,
}

// Configuration for a single connector
#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectorConfig {
    pub key: Vec<PaymentMethodTypeConfig>,
    pub psp_time_config: HashMap<Key, Key>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaymentMethodTypeConfig {
    pub payment_method: Key,
    pub payment_method_type: Key,
    pub sr: u8, // Success rate
}

#[derive(Debug, Deserialize, Serialize)]    
pub struct MerchantMethodTypeConfig {
    pub payment_method: Key,
    pub payment_method_type: Key,
    
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PspTimeConfig {
    pub key: HashMap<Key, Key>,
}

// Main PSP configuration loaded from JSON
#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct PspConfig {
    pub key: Vec<MerchantMethodTypeConfig>
}
#[derive(Debug, Deserialize, Serialize)]
pub struct MerchantConfig {
    pub config: HashMap<Key, PspConfig>,
    pub time_config: Key,
} 

#[derive(Debug, Deserialize, Serialize)]
pub struct PaymentMethodConfig {
    pub key: Vec<PaymentMethodTypeConfig>,
}

// impl MerchantConfig {
//     pub fn get_connector_list(&self) -> Result<Vec<String>> {
//         let mut connectors = Vec::new();
//         for (key, _) in self.config.iter() {
//             connectors.push(key.clone());
//         }
//         Ok(connectors)
//     }
// }
// #[derive(Debug, Deserialize, Serialize)]
// pub struct PaymentMethodKey {
//     pub payment_method: String,
//     pub payment_method_type: String, 
//     pub amount_less_than: Option<u64>,
// }

pub fn find_suitable_connectors (
    sample: &HashMap<&Key, &Key>,
    merchant_config: &MerchantConfig) -> Vec<Key> {
        let mut suitable_connectors = Vec::new();
        let payment_method = sample.get(&Key("payment_method".to_string())).unwrap().to_owned();
        let payment_method_type = sample.get(&Key("payment_method_type".to_string())).unwrap().to_owned();

        for (k, v) in &merchant_config.config  {
            for config in &v.key {
                if config.payment_method == *payment_method && (config.payment_method_type == *payment_method_type || config.payment_method_type.0 == "*") {
                    suitable_connectors.push(k.clone());
                }
            }
        }
        
        suitable_connectors
}

pub struct StraightThroughRouting{
    pub connectors: Vec<Key>
}
impl StraightThroughRouting{
    pub fn get_connector(&self) -> Key{
        let mut rng = rand::thread_rng();
        self.connectors[rng.gen_range(0..self.connectors.len())].clone()
    }
    
}
pub struct PaymentRecorderData{
    pub connector: Key,
    pub verdict: Status,
    pub payment_data: Key,
}
impl PaymentRecorderData {
    pub fn set_values(connector: Key, verdict: Status, payment_data: Key) -> Self {
        PaymentRecorderData {
            connector,
            verdict,
            payment_data,
        }
    }
    
}
