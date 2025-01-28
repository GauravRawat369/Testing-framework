use crate::types::config::{Config, Status, UserSimulationConfig, PspSimulationConfig};
use anyhow::Result;

pub trait Recorder {
    fn record_transaction(
        connector: &str,
        verdict: Status,
        user: &UserSimulationConfig,
        psp: &PspSimulationConfig,
    ) -> Result<()>;
}

impl Recorder for Config {
    fn record_transaction(
        connector: &str,
        verdict: Status,
        user: &UserSimulationConfig,
        psp: &PspSimulationConfig,
    ) -> Result<()> {
        // println!("Recording transaction for connector: {}", connector);
        // println!("Verdict: {:?}", verdict);
        // println!("User: {:?}", user);
        // println!("PSP: {:?}", psp);
        Ok(())
    }
}
