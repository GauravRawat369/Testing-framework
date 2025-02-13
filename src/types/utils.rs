// src/utils.rs

use rand::Rng;

use crate::types::config::PaymentConnector;

/// Formats a success rate as a percentage.


/// Logs the current state of all connectors.
pub fn log_connectors(connectors: &Vec<PaymentConnector>) {
    println!("Current Connector States:");
    for connector in connectors {
        let success_rate = if connector.attempts == 0 {
            0.0
        } else {
            (connector.successes as f64 / connector.attempts as f64) * 100.0
        };
        println!(
            "Connector: {}, Success Rate: {:.2}%",
            connector.name, success_rate
        );
    }
    println!();
}


/// Helper function to print a separator line for better readability.
pub fn print_separator() {
    println!("{}", "-".repeat(50));
}