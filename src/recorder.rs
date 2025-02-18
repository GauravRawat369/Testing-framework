use crate::types::PaymentRecorderData;
use std::collections::HashMap;
use anyhow::Result;
use csv::Writer;
use std::fs::OpenOptions;
use crate::types::Metrics;
use crate::types::Status;

pub trait Recorder {
    fn record_transaction(
        &self,
        metrics: &mut Metrics,
    ) -> Result<()>;
}

impl Recorder for PaymentRecorderData {
    fn record_transaction(
        &self,
        metrics: &mut Metrics,
    ) -> Result<()> {
        // Open the CSV file in append mode
        let data = self;
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
        if let Some(payment_method) = payment_data.get("payment_methods") {
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

        // New: Update metrics data with the transaction
        let pm = if payment_method_str.is_empty() { "N/A".to_string() } else { payment_method_str.clone() };
        let pmt = if payment_method_type_str.is_empty() { "N/A".to_string() } else { payment_method_type_str.clone() };
        let connector = self.connector.0.clone();
        let verdict = self.verdict.clone();

        // Assuming metrics.key is: HashMap<String, HashMap<String, HashMap<String, HashMap<Status, u32>>>>
        metrics.key.entry(connector).or_insert_with(HashMap::new)
            .entry(pm).or_insert_with(HashMap::new)
            .entry(pmt).or_insert_with(HashMap::new)
            .entry(verdict).and_modify(|e| *e += 1).or_insert(1);

        Ok(())
    }
}

pub fn print_metrics(metrics: &Metrics) {
    // Print the success rate for each connector
    let mut map: HashMap<String, usize> = HashMap::new();
    let mut total_success_count = 0;
    // println!("Success Rate Metrics: {:?}",metrics);
    for (connector, payment_method_map) in &metrics.key {
        let mut total_count = 0;
        let mut success_count = 0;
        for (_, payment_method_type_map) in payment_method_map {
            for (_, status_map) in payment_method_type_map {
                for (status, count) in status_map {
                    total_count += count;
                    if *status == Status::Success {
                        success_count += count;
                        total_success_count += count;
                    }
                    
                }
            }
        }
        // println!("Total transactions from {:?} connector: {}", connector, total_count);
        map.insert(connector.clone(), total_count);
        let success_rate = (success_count as f64 / total_count as f64) * 100.0;
        println!("Connector: {:?}, Success Rate: {:.2}%", connector, success_rate);

        // For each payment method
        for (payment_method, payment_method_type_map) in payment_method_map {
            let mut total_count = 0;
            let mut success_count = 0;
            for (_, status_map) in payment_method_type_map {
                for (status, count) in status_map {
                    total_count += count;
                    if *status == Status::Success {
                        success_count += count;
                    }
                }
            }
            let success_rate = (success_count as f64 / total_count as f64) * 100.0;
            println!("Connector: {:?}, Payment Method: {:?}, Success Rate: {:.2}%", connector, payment_method, success_rate);

            // For each payment method type
            for (payment_method_type, status_map) in payment_method_type_map {
                let mut total_count = 0;
                let mut success_count = 0;
                for (status, count) in status_map {
                    total_count += count;
                    if *status == Status::Success {
                        success_count += count;
                    }
                }
                let success_rate = (success_count as f64 / total_count as f64) * 100.0;
                println!("Connector: {:?}, Payment Method: {:?}, Payment Method Type: {:?}, Success Rate: {:.2}%", connector, payment_method, payment_method_type, success_rate);
            }
        }
    }
    let total_transactions: usize = map.values().sum();
    println!("Total transactions: {}", total_transactions);
    for (connector, total_count) in map {
        println!("Total transactions from {:?} connector: {:.2}%", connector, (total_count as f64 / total_transactions as f64) * 100.0);
    }
    let total_success_rate = (total_success_count as f64 / total_transactions as f64) * 100.0;
    println!("Total Success Rate: {:.2}%", total_success_rate);
}
