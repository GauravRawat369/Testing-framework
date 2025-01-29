use crate::types::config::{Payment_Recorder_data, Key, Status,};
use anyhow::Result;
use csv::Writer;
use std::fs::OpenOptions;

pub trait Recorder {
    fn record_transaction(
        &self,
    ) -> Result<()>;
}

impl Recorder for Payment_Recorder_data {
    fn record_transaction(
        &self,
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
            &self.connector.0,
            &format!("{:?}", &self.verdict),
            &serde_json::to_string(&self.payment_data).unwrap(),
        ])?;
        wtr.flush()?;
        Ok(())
    }
}
