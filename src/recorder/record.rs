use crate::types::config::{Config, Key, PspSimulationConfig, Status, UserSimulationConfig};
use anyhow::Result;

pub trait Recorder {
    fn record_transaction(
        connector: &Key,
        verdict: Status,
        user: &UserSimulationConfig,
        psp: &PspSimulationConfig,
    ) -> Result<()>;
}

impl Recorder for Config {
    fn record_transaction(
        connector: &Key,
        verdict: Status,
        user: &UserSimulationConfig,
        psp: &PspSimulationConfig,
    ) -> Result<()> {
        // println!("Recording transaction for connector: {}", connector.0);
        // println!("Verdict: {:?}", verdict);
        // println!("User: {:?}", user);
        // println!("PSP: {:?}", psp);
        Ok(())
    }
}
