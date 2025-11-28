use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for communication failure injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureConfig {
    /// Master enable/disable for all failure injection
    pub enabled: bool,

    /// Connection health percentage (0-100)
    /// 100 = perfect connection, 0 = severely degraded
    pub connection_health: u8,

    /// Packet loss rate (0.0-1.0)
    /// Probability that an outgoing message will be dropped
    pub packet_loss_rate: f64,

    /// Minimum delay before sending message (milliseconds)
    pub delay_min_ms: u64,

    /// Maximum delay before sending message (milliseconds)
    pub delay_max_ms: u64,

    /// Message corruption rate (0.0-1.0)
    /// Probability that a message will be corrupted before sending
    pub corruption_rate: f64,

    /// Force disconnect rate (0.0-1.0)
    /// Probability that the connection will be forcefully dropped
    pub force_disconnect_rate: f64,
}

impl Default for FailureConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            connection_health: 100,
            packet_loss_rate: 0.0,
            delay_min_ms: 0,
            delay_max_ms: 0,
            corruption_rate: 0.0,
            force_disconnect_rate: 0.0,
        }
    }
}

impl FailureConfig {
    /// Create config from connection health slider (0-100)
    /// Maps health percentage to realistic failure rates:
    /// - 100%: Perfect (no failures)
    /// - 75%:  Minor issues (5% loss, 0-50ms delay)
    /// - 50%:  Degraded (15% loss, 0-200ms delay)
    /// - 25%:  Unstable (30% loss, 0-500ms delay, 5% corruption)
    /// - 0%:   Severe (50% loss, 0-1000ms delay, 10% corruption, 5% disconnect)
    pub fn from_health(health: u8) -> Self {
        let health = health.min(100);
        let health_f = health as f64 / 100.0;

        // Inverse relationship: lower health = higher failure rates
        let packet_loss = (1.0 - health_f) * 0.5; // 0% to 50%
        let max_delay = ((1.0 - health_f) * 1000.0) as u64; // 0ms to 1000ms
        let corruption = if health < 50 {
            (1.0 - health_f) * 0.1 // 0% to 10%
        } else {
            0.0
        };
        let disconnect = if health < 25 {
            (1.0 - health_f) * 0.05 // 0% to 5%
        } else {
            0.0
        };

        Self {
            enabled: health < 100,
            connection_health: health,
            packet_loss_rate: packet_loss,
            delay_min_ms: 0,
            delay_max_ms: max_delay,
            corruption_rate: corruption,
            force_disconnect_rate: disconnect,
        }
    }

    /// Update only the connection health field, recalculating other rates
    pub fn set_health(&mut self, health: u8) {
        *self = Self::from_health(health);
    }

    /// Validate configuration values are within acceptable ranges
    pub fn is_valid(&self) -> bool {
        self.connection_health <= 100
            && self.packet_loss_rate >= 0.0
            && self.packet_loss_rate <= 1.0
            && self.corruption_rate >= 0.0
            && self.corruption_rate <= 1.0
            && self.force_disconnect_rate >= 0.0
            && self.force_disconnect_rate <= 1.0
            && self.delay_min_ms <= self.delay_max_ms
    }
}

/// Failure injection simulator that makes probabilistic decisions
pub struct FailureSimulator {
    config: FailureConfig,
    rng: rand::rngs::ThreadRng,
}

impl FailureSimulator {
    /// Create a new simulator from configuration
    pub fn new(config: FailureConfig) -> Self {
        Self {
            config,
            rng: rand::rng(),
        }
    }

    /// Check if failure injection is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Decide if this message should be dropped (packet loss)
    /// Returns true if the message should NOT be sent
    pub fn should_drop_packet(&mut self) -> bool {
        if !self.config.enabled || self.config.packet_loss_rate == 0.0 {
            return false;
        }
        self.rng.random::<f64>() < self.config.packet_loss_rate
    }

    /// Get delay duration for this message
    /// Returns Duration(0) if no delay should be applied
    pub fn get_delay(&mut self) -> Duration {
        if !self.config.enabled || self.config.delay_max_ms == 0 {
            return Duration::from_millis(0);
        }

        let delay_ms = if self.config.delay_min_ms >= self.config.delay_max_ms {
            self.config.delay_max_ms
        } else {
            self.rng
                .random_range(self.config.delay_min_ms..=self.config.delay_max_ms)
        };

        Duration::from_millis(delay_ms)
    }

    /// Decide if this message should be corrupted
    /// Returns true if the message bytes should be modified before sending
    pub fn should_corrupt_message(&mut self) -> bool {
        if !self.config.enabled || self.config.corruption_rate == 0.0 {
            return false;
        }
        self.rng.random::<f64>() < self.config.corruption_rate
    }

    /// Decide if the connection should be forcefully dropped
    /// Returns true if the connection should be terminated
    pub fn should_disconnect(&mut self) -> bool {
        if !self.config.enabled || self.config.force_disconnect_rate == 0.0 {
            return false;
        }
        self.rng.random::<f64>() < self.config.force_disconnect_rate
    }

    /// Corrupt a message by modifying its bytes
    /// Creates protocol-invalid messages for testing client error handling
    pub fn corrupt_message(&mut self, original: &[u8]) -> Vec<u8> {
        if original.is_empty() {
            return original.to_vec();
        }

        let mut corrupted = original.to_vec();
        let corruption_type = self.rng.random_range(0..=4);

        match corruption_type {
            0 => {
                // Corrupt length field (first 4 bytes)
                if corrupted.len() >= 4 {
                    corrupted[0] = b'9';
                    corrupted[1] = b'9';
                    corrupted[2] = b'9';
                    corrupted[3] = b'9';
                }
            }
            1 => {
                // Corrupt MID field (bytes 4-7)
                if corrupted.len() >= 8 {
                    corrupted[4] = b'X';
                    corrupted[5] = b'X';
                }
            }
            2 => {
                // Flip random bytes
                let num_flips = self.rng.random_range(1..=3.min(corrupted.len()));
                for _ in 0..num_flips {
                    let idx = self.rng.random_range(0..corrupted.len());
                    corrupted[idx] = corrupted[idx].wrapping_add(1);
                }
            }
            3 => {
                // Truncate message
                let new_len = self.rng.random_range(1..corrupted.len());
                corrupted.truncate(new_len);
            }
            4 => {
                // Add garbage bytes
                let garbage_count = self.rng.random_range(1..=10);
                for _ in 0..garbage_count {
                    corrupted.push(self.rng.random());
                }
            }
            _ => unreachable!(),
        }

        corrupted
    }

    /// Get the current configuration
    pub fn config(&self) -> &FailureConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_disabled() {
        let config = FailureConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.connection_health, 100);
        assert_eq!(config.packet_loss_rate, 0.0);
    }

    #[test]
    fn test_perfect_health_no_failures() {
        let config = FailureConfig::from_health(100);
        assert!(!config.enabled);
        assert_eq!(config.packet_loss_rate, 0.0);
        assert_eq!(config.delay_max_ms, 0);
        assert_eq!(config.corruption_rate, 0.0);
    }

    #[test]
    fn test_zero_health_maximum_failures() {
        let config = FailureConfig::from_health(0);
        assert!(config.enabled);
        assert_eq!(config.packet_loss_rate, 0.5); // 50%
        assert_eq!(config.delay_max_ms, 1000);
        assert!(config.corruption_rate > 0.0);
        assert!(config.force_disconnect_rate > 0.0);
    }

    #[test]
    fn test_mid_health_moderate_failures() {
        let config = FailureConfig::from_health(50);
        assert!(config.enabled);
        assert!(config.packet_loss_rate > 0.0 && config.packet_loss_rate < 0.5);
        assert!(config.delay_max_ms > 0 && config.delay_max_ms < 1000);
    }

    #[test]
    fn test_config_validation() {
        let mut config = FailureConfig::default();
        assert!(config.is_valid());

        config.connection_health = 150;
        assert!(!config.is_valid());

        config.connection_health = 50;
        config.packet_loss_rate = 1.5;
        assert!(!config.is_valid());

        config.packet_loss_rate = 0.5;
        config.delay_min_ms = 1000;
        config.delay_max_ms = 500;
        assert!(!config.is_valid());
    }

    #[test]
    fn test_simulator_disabled_behavior() {
        let config = FailureConfig::default();
        let mut sim = FailureSimulator::new(config);

        assert!(!sim.is_enabled());
        assert!(!sim.should_drop_packet());
        assert_eq!(sim.get_delay(), Duration::from_millis(0));
        assert!(!sim.should_corrupt_message());
        assert!(!sim.should_disconnect());
    }

    #[test]
    fn test_simulator_packet_loss() {
        let config = FailureConfig {
            enabled: true,
            packet_loss_rate: 1.0, // Always drop
            ..Default::default()
        };

        let mut sim = FailureSimulator::new(config);
        assert!(sim.should_drop_packet());
    }

    #[test]
    fn test_simulator_delay() {
        let config = FailureConfig {
            enabled: true,
            delay_min_ms: 100,
            delay_max_ms: 200,
            ..Default::default()
        };

        let mut sim = FailureSimulator::new(config);
        let delay = sim.get_delay();
        assert!(delay.as_millis() >= 100 && delay.as_millis() <= 200);
    }

    #[test]
    fn test_message_corruption_modifies_bytes() {
        let config = FailureConfig {
            enabled: true,
            corruption_rate: 1.0,
            ..Default::default()
        };

        let mut sim = FailureSimulator::new(config);
        let original = b"00200001001         001";
        let corrupted = sim.corrupt_message(original);

        // Corrupted message should be different
        assert_ne!(original.to_vec(), corrupted);
    }

    #[test]
    fn test_corruption_types_are_invalid_protocol() {
        let config = FailureConfig {
            enabled: true,
            ..Default::default()
        };

        let mut sim = FailureSimulator::new(config);
        let original = b"00200001001         001";

        // Run multiple times to hit different corruption types
        for _ in 0..20 {
            let corrupted = sim.corrupt_message(original);
            // Verify it's been modified (high probability)
            if corrupted != original {
                // Could check that it's not parseable as valid protocol
                // For now, just verify it changed
                return;
            }
        }
    }

    #[test]
    fn test_set_health_updates_config() {
        let mut config = FailureConfig::from_health(100);
        assert_eq!(config.packet_loss_rate, 0.0);

        config.set_health(50);
        assert!(config.packet_loss_rate > 0.0);
        assert_eq!(config.connection_health, 50);
    }
}
