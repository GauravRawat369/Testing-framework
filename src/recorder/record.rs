use crate::types::config::{Config, Key, PspSimulationConfig, Status, UserSimulationConfig};
use anyhow::Result;
use csv::Writer;
use std::fs::OpenOptions;

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
        // Open the CSV file in append mode
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("records.csv")
            .unwrap();
        let mut wtr = Writer::from_writer(file);

        // Write the transaction details to the CSV file
        wtr.write_record(&[
            &connector.0,
            &format!("{:?}", verdict),
            &serde_json::to_string(user).unwrap(),
            &serde_json::to_string(psp).unwrap(),
        ])?;
        wtr.flush()?;
        Ok(())
    }
}
