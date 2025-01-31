use crate::types::config::PaymentRecorderData;
use crate::types::config::Key;
use std::collections::HashMap;

use anyhow::Result;
use csv::Writer;
use std::fs::OpenOptions;

pub trait Recorder {
    fn record_transaction(
        &self,
    ) -> Result<()>;
}

impl Recorder for PaymentRecorderData {
    fn record_transaction(
        &self,
    ) -> Result<()> {
        // Open the CSV file in append mode
        let data = self.clone();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("records.csv")
            .unwrap();
        let mut wtr = Writer::from_writer(file);

        // Write the transaction details to the CSV file
        let payment_data: HashMap<&str, &str> = serde_json::from_str(&data.payment_data.0).unwrap();
        let verdict_str = format!("{:?}", &self.verdict);
        let mut record = vec![
            &self.connector.0,
            &verdict_str,
        ];

        let mut payment_method_str = String::new();
        if let Some(payment_method) = payment_data.get("payment_method") {
            payment_method_str = payment_method.to_string();
            record.push(&payment_method_str);
        }
        let mut payment_method_type_str = String::new();
        if let Some(payment_method_type) = payment_data.get("payment_method_type") {
            payment_method_type_str = payment_method_type.to_string();
            record.push(&payment_method_type_str);
        }

        wtr.write_record(&record)?;
        wtr.flush()?;
        Ok(())
    }
}
