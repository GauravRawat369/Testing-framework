// src/thompson_sampling.rs

use rand_distr::{Beta, Distribution};
use rand::thread_rng;
use crate::types::config::{PaymentConnector, RoutingAlgorithm};

/// Thompson Sampling with Discount Factor.
pub struct ThompsonSampling {
    gamma: f64, // Discount factor
}

impl ThompsonSampling {
    pub fn new(gamma: f64) -> Self {
        ThompsonSampling { gamma }
    }
}

impl RoutingAlgorithm for ThompsonSampling {
    /// Select a connector using Thompson Sampling.
    fn select_connector(&self, connectors: &mut Vec<PaymentConnector>) -> usize {
        let mut rng = thread_rng();
        let mut best_connector_index = 0;
        let mut best_sampled_rate = 0.0;

        for (index, connector) in connectors.iter().enumerate() {
            let beta_dist = Beta::new(connector.alpha, connector.beta).unwrap();
            let sampled_rate = beta_dist.sample(&mut rng);

            if sampled_rate > best_sampled_rate {
                best_sampled_rate = sampled_rate;
                best_connector_index = index;
            }
        }

        best_connector_index
    }

    /// Update the connector's success rate using the discount factor.
    fn update_connector(&mut self, connectors: &mut Vec<PaymentConnector>, connector_index: usize, success: bool) {
        let connector = &mut connectors[connector_index];

        // Update alpha and beta for Thompson Sampling
        connector.alpha = self.gamma * connector.alpha + if success { 1.0 } else { 0.0 };
        connector.beta = self.gamma * connector.beta + if success { 0.0 } else { 1.0 };

        // Update successes and attempts for logging
        if success {
            connector.successes += 1;
        }
        connector.attempts += 1;
    }
}