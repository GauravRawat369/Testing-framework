// src/sliding_window_ucb.rs

use crate::types::config::{PaymentConnector, RoutingAlgorithm};

/// Sliding Window UCB.
pub struct SlidingWindowUCB {
    window_size: usize,
    exploration_factor: f64,
}

impl SlidingWindowUCB {
    pub fn new(window_size: usize, exploration_factor: f64) -> Self {
        SlidingWindowUCB {
            window_size,
            exploration_factor,
        }
    }
}

impl RoutingAlgorithm for SlidingWindowUCB {
    /// Select a connector using UCB.
    fn select_connector(&self, connectors: &mut Vec<PaymentConnector>) -> usize {
        let total_attempts: usize = connectors.iter().map(|c| c.attempts).sum();
        let mut best_connector_index = 0;
        let mut best_ucb_score = 0.0;

        for (index, connector) in connectors.iter().enumerate() {
            if connector.attempts == 0 {
                return index; // Explore untried connectors
            }

            let success_rate = connector.successes as f64 / connector.attempts as f64;
            let ucb_score = success_rate
                + self.exploration_factor
                    * ((total_attempts as f64).ln() / connector.attempts as f64).sqrt();

            if ucb_score > best_ucb_score {
                best_ucb_score = ucb_score;
                best_connector_index = index;
            }
        }

        best_connector_index
    }

    /// Update the connector's sliding window.
    fn update_connector(&mut self, connectors: &mut Vec<PaymentConnector>, connector_index: usize, success: bool) {
        let connector = &mut connectors[connector_index];

        // Update the sliding window
        if connector.window.len() == self.window_size {
            let oldest_outcome = connector.window.remove(0);
            if oldest_outcome {
                connector.successes -= 1;
            }
            connector.attempts -= 1;
        }
        connector.window.push(success);

        // Update successes and attempts
        if success {
            connector.successes += 1;
        }
        connector.attempts += 1;
    }
}