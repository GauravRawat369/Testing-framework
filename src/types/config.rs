use serde::{Deserialize, Serialize};
use core::ops::Deref;
use std::collections::HashMap;
use crate::simulate::user::Sampler;
use std::path::Path;
use anyhow::{ensure, Context, Result};
use rand::Rng;
use serde_json::Value;

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
pub struct UserSimulationConfig {
    #[serde(default = "default_amount")]
    pub amount: Option<AmountRange>,
    pub currency: Option<String>,
    #[serde(flatten)]
    pub payment_methods: SimulationConfig,
    pub extra_fields: Option<HashMap<Key, Value>>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AmountRange { pub min: u32, pub max: u32}

fn default_amount() -> Option<AmountRange> {
    Some(AmountRange { min: 0, max: 2000 })
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SimulationConfig(HashMap<Key, PaymentMethods>);

impl Sampler for UserSimulationConfig {
    fn generate_sample(&self) -> Result<HashMap<Key, Key>> {
        let amt = Self::generate_random_amount(&self.amount);
        let binding = "USD".to_string();
        let currency = self.currency.as_ref().unwrap_or(&binding);
        let mut sample = HashMap::new();
        sample.insert(Key("amount".to_string()), Key(amt.to_string()));
        sample.insert(Key("currency".to_string()), Key(currency.clone()));
        let res = Self::list_payment_methods(&self.payment_methods);
        res.map(|payment_methods| {
            sample.extend(payment_methods.into_iter().map(|(k, v)| (k.to_owned(), v.to_owned())));
            sample
        })
    }
}

impl Deref for SimulationConfig {
    type Target = HashMap<Key, PaymentMethods>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PaymentMethods(HashMap<Key, PaymentMethodDetails>);

impl Deref for PaymentMethods {
    type Target = HashMap<Key, PaymentMethodDetails>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PaymentMethodDetails {
    Percentage(u8),
    Composite {
        percentage: u8,
        next: SimulationConfig,
        extra_fields: Option<HashMap<Key, Value>>
    }
}


impl PaymentMethods {
    pub fn validate(&self) -> Result<()> {
        let mut total = 0;
        for (_key, value) in self.0.iter() {
            match value {
                &PaymentMethodDetails::Percentage(value) => total += value,
                &PaymentMethodDetails::Composite { percentage, ref next , extra_fields: _} => {
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
        self.payment_methods.validate()
    }
}

//psp structs
#[derive(Debug, Deserialize, Serialize)]
pub struct PspSimulationConfig {
    pub psp_variants: HashMap<Key, PspDetails>,
    pub otherwise: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PspDetails {
    pub payment_methods: HashMap<Key, PaymentMethodTypes>,
    pub psp_time_config: Option<PspTimeConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PaymentMethodTypes {
    PaymentTypes(Vec<PaymentTypeDetails>),
    Simple { sr: u32 },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaymentTypeDetails {
    pub payment_method_type: Key,
    pub sr: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PspTimeConfig {
    pub mean: u32,
    pub stddev: u32,
}

impl PspSimulationConfig {
    pub fn default_status(&self) -> Status {
        match self.otherwise {
            Some(ref status) => {
                match status.as_str() {
                    "success" => Status::Success,
                    _ => Status::Failure,
                }
            },
            None => Status::Failure
        }
    }
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
#[derive(Debug, Deserialize, Serialize)]
pub struct MerchantConfig {
    pub connectors_list: HashMap<Key, ConnectorDetails>,
    pub extra_fields: Option<HashMap<Key, Value>>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectorDetails {
    pub supported_payment_methods: HashMap<Key, PaymentMethodConfig>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaymentMethodConfig {
    pub payment_method_types: Option<Vec<String>>,
    pub supported_behaviours: Option<HashMap<Key, Value>>
}

// pub fn find_suitable_connectors (
//     sample: &HashMap<&Key, &Key>,
//     merchant_config: &MerchantConfig) -> Vec<Key> {
//         let mut suitable_connectors = Vec::new();
//         let payment_method = sample.get(&Key("payment_method".to_string())).unwrap().to_owned();
//         let payment_method_type = sample.get(&Key("payment_method_type".to_string())).unwrap().to_owned();

//         for (k, v) in &merchant_config.config  {
//             for config in &v.key {
//                 if config.payment_method == *payment_method && (config.payment_method_type == *payment_method_type || config.payment_method_type.0 == "*") {
//                     suitable_connectors.push(k.clone());
//                 }
//             }
//         }
        
//         suitable_connectors
// }

pub struct StraightThroughRouting{
    pub connectors: Vec<Key>
}
impl StraightThroughRouting{
    pub fn get_connector(&self) -> Key{
        let mut rng = rand::thread_rng();
        self.connectors[rng.gen_range(0..self.connectors.len())].clone()
    }
    
}

#[derive(Debug, Deserialize, Eq, PartialEq, Hash, Clone)]
pub enum Status {
    Success,
    Failure
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
#[derive(Debug)]

pub struct Metrics {
    pub key: std::collections::HashMap<String, std::collections::HashMap<String, std::collections::HashMap<String, std::collections::HashMap<Status, usize>>>>,
}

impl Metrics {
   pub fn new() -> Self {
        Metrics {
            key: std::collections::HashMap::new(),
        }
    }
}