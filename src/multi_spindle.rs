use serde::{Deserialize, Serialize};

/// Configuration for multi-spindle operation mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSpindleConfig {
    /// Whether multi-spindle mode is enabled
    pub enabled: bool,

    /// Number of spindles in the multi-spindle system (2-16 typically)
    pub spindle_count: u8,

    /// Sync tightening ID that groups spindles together
    /// All spindles with the same sync_id tighten simultaneously
    pub sync_id: u32,
}

impl Default for MultiSpindleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            spindle_count: 1,
            sync_id: 0,
        }
    }
}

impl MultiSpindleConfig {
    pub fn new(spindle_count: u8, sync_id: u32) -> Self {
        Self {
            enabled: true,
            spindle_count,
            sync_id,
        }
    }

    /// Disable multi-spindle mode (revert to single-spindle)
    pub fn disable() -> Self {
        Self::default()
    }

    /// Validate configuration
    pub fn is_valid(&self) -> bool {
        if !self.enabled {
            return true;
        }

        // Spindle count must be at least 2 for multi-spindle
        self.spindle_count >= 2 && self.spindle_count <= 16
    }
}

/// Individual spindle result within a multi-spindle operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpindleResult {
    /// Spindle number (1-based indexing)
    pub spindle_id: u8,

    /// Channel ID (same as spindle_id typically)
    pub channel_id: u8,

    /// Final torque value achieved (Nm * 100)
    pub torque: i32,

    /// Final angle value achieved (degrees * 10)
    pub angle: i32,

    /// Torque status: OK (0) or NOK (1)
    pub torque_status: u8,

    /// Angle status: OK (0) or NOK (1)
    pub angle_status: u8,
}

impl SpindleResult {
    /// Create a successful (OK) spindle result
    ///
    /// Convenience constructor for manual result creation in testing
    /// or programmatic generation scenarios (e.g., webUI custom result creation).
    #[allow(dead_code)]
    pub fn ok(spindle_id: u8, torque: i32, angle: i32) -> Self {
        Self {
            spindle_id,
            channel_id: spindle_id,
            torque,
            angle,
            torque_status: 0, // OK
            angle_status: 0,  // OK
        }
    }

    /// Create a failed (NOK) spindle result
    ///
    /// Convenience constructor for manual result creation with specific failure modes.
    /// Useful for simulating particular failure scenarios in testing and webUI.
    #[allow(dead_code)]
    pub fn nok(
        spindle_id: u8,
        torque: i32,
        angle: i32,
        torque_failed: bool,
        angle_failed: bool,
    ) -> Self {
        Self {
            spindle_id,
            channel_id: spindle_id,
            torque,
            angle,
            torque_status: if torque_failed { 1 } else { 0 },
            angle_status: if angle_failed { 1 } else { 0 },
        }
    }

    /// Check if this spindle result is OK
    pub fn is_ok(&self) -> bool {
        self.torque_status == 0 && self.angle_status == 0
    }
}

/// Aggregated multi-spindle tightening result
/// Used for MID 0101 broadcasts and batch tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSpindleResult {
    /// Unique identifier for this multi-spindle operation
    pub result_id: u32,

    /// Sync tightening ID that groups these spindles
    pub sync_id: u32,

    /// Timestamp of the tightening operation
    pub timestamp: String, // Format: "2024-01-15 14:30:45"

    /// Overall tightening status: OK (0) or NOK (1)
    pub overall_status: u8,

    /// Number of spindles involved
    pub spindle_count: u8,

    /// Individual results for each spindle
    pub spindle_results: Vec<SpindleResult>,
}

impl MultiSpindleResult {
    /// Create a new multi-spindle result
    ///
    /// Public constructor for creating results from individual spindle data.
    /// Used in testing and programmatic result generation with custom
    /// per-spindle outcomes (e.g., simulation, webUI features).
    #[allow(dead_code)]
    pub fn new(result_id: u32, sync_id: u32, spindle_results: Vec<SpindleResult>) -> Self {
        let spindle_count = spindle_results.len() as u8;

        // Overall status is OK only if ALL spindles are OK
        let overall_status = if spindle_results.iter().all(|r| r.is_ok()) {
            0 // OK
        } else {
            1 // NOK
        };

        Self {
            result_id,
            sync_id,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            overall_status,
            spindle_count,
            spindle_results,
        }
    }

    /// Check if the overall result is OK
    pub fn is_ok(&self) -> bool {
        self.overall_status == 0
    }

    /// Get the count of OK spindles
    ///
    /// Diagnostic method for analyzing multi-spindle results.
    /// Used in webUI statistics and monitoring dashboards.
    #[allow(dead_code)]
    pub fn ok_count(&self) -> usize {
        self.spindle_results.iter().filter(|r| r.is_ok()).count()
    }

    /// Get the count of NOK spindles
    ///
    /// Diagnostic method for analyzing multi-spindle failures.
    /// Used in webUI statistics and monitoring dashboards.
    #[allow(dead_code)]
    pub fn nok_count(&self) -> usize {
        self.spindle_results.iter().filter(|r| !r.is_ok()).count()
    }
}

/// Lightweight multi-spindle status information
/// Used for MID 0091 status broadcasts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSpindleStatus {
    /// Sync tightening ID
    pub sync_id: u32,

    /// Status: 0=Waiting, 1=Running, 2=Completed
    pub status: u8,

    /// Number of spindles configured
    pub spindle_count: u8,

    /// Timestamp when status changed
    pub timestamp: String,
}

impl MultiSpindleStatus {
    /// Create a status in "Waiting" state
    ///
    /// Used for MID 0091 broadcasts when multi-spindle operation is waiting to start.
    /// Triggered by auto-tightening or webUI simulation of multi-spindle operations.
    #[allow(dead_code)]
    pub fn waiting(sync_id: u32, spindle_count: u8) -> Self {
        Self {
            sync_id,
            status: 0,
            spindle_count,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    /// Create a status in "Running" state
    ///
    /// Used for MID 0091 broadcasts when multi-spindle operation is in progress.
    /// Sent during active multi-spindle tightening simulations.
    #[allow(dead_code)]
    pub fn running(sync_id: u32, spindle_count: u8) -> Self {
        Self {
            sync_id,
            status: 1,
            spindle_count,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    /// Create a status in "Completed" state
    ///
    /// Used for MID 0091 broadcasts when multi-spindle operation has finished.
    /// Sent after multi-spindle tightening cycle completes.
    #[allow(dead_code)]
    pub fn completed(sync_id: u32, spindle_count: u8) -> Self {
        Self {
            sync_id,
            status: 2,
            spindle_count,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// Generate simulated multi-spindle tightening results
///
/// Creates realistic spindle results based on the configuration.
/// Each spindle has slightly varying torque/angle values to simulate
/// real-world variation across multiple spindles.
///
/// Used by auto-tightening to generate multi-spindle results with per-pset
/// configuration, and by webUI simulation controls for manual testing.
pub fn generate_multi_spindle_results(
    config: &MultiSpindleConfig,
    result_id: u32,
    _pset_id: u32,
) -> MultiSpindleResult {
    let mut spindle_results = Vec::new();

    // Base values (will vary per spindle)
    let base_torque = 5000; // 50.00 Nm
    let base_angle = 1800; // 180.0 degrees

    for spindle_id in 1..=config.spindle_count {
        // Add slight variation per spindle (±10%)
        let variation = (spindle_id as i32 - 1) * 5;
        let torque = base_torque + (variation * 10);
        let angle = base_angle + (variation * 2);

        // Simulate 90% success rate (last spindle might fail occasionally)
        let is_ok = spindle_id != config.spindle_count || !result_id.is_multiple_of(10);

        let result = if is_ok {
            SpindleResult::ok(spindle_id, torque, angle)
        } else {
            // Simulate torque failure on last spindle occasionally
            SpindleResult::nok(spindle_id, torque - 500, angle, true, false)
        };

        spindle_results.push(result);
    }

    MultiSpindleResult::new(result_id, config.sync_id, spindle_results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_spindle_config_default() {
        let config = MultiSpindleConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.spindle_count, 1);
        assert_eq!(config.sync_id, 0);
    }

    #[test]
    fn test_multi_spindle_config_validation() {
        let config = MultiSpindleConfig::new(2, 100);
        assert!(config.is_valid());
        assert!(config.enabled);

        let invalid = MultiSpindleConfig {
            enabled: true,
            spindle_count: 1, // Too few for multi-spindle
            sync_id: 100,
        };
        assert!(!invalid.is_valid());

        let too_many = MultiSpindleConfig {
            enabled: true,
            spindle_count: 17, // Too many
            sync_id: 100,
        };
        assert!(!too_many.is_valid());
    }

    #[test]
    fn test_spindle_result_ok() {
        let result = SpindleResult::ok(1, 5000, 1800);
        assert_eq!(result.spindle_id, 1);
        assert_eq!(result.torque, 5000);
        assert_eq!(result.angle, 1800);
        assert!(result.is_ok());
    }

    #[test]
    fn test_spindle_result_nok() {
        let result = SpindleResult::nok(2, 4500, 1750, true, false);
        assert_eq!(result.spindle_id, 2);
        assert_eq!(result.torque_status, 1); // NOK
        assert_eq!(result.angle_status, 0); // OK
        assert!(!result.is_ok());
    }

    #[test]
    fn test_multi_spindle_result_all_ok() {
        let spindles = vec![
            SpindleResult::ok(1, 5000, 1800),
            SpindleResult::ok(2, 5100, 1810),
        ];

        let result = MultiSpindleResult::new(1, 100, spindles);
        assert_eq!(result.spindle_count, 2);
        assert!(result.is_ok());
        assert_eq!(result.ok_count(), 2);
        assert_eq!(result.nok_count(), 0);
    }

    #[test]
    fn test_multi_spindle_result_with_failure() {
        let spindles = vec![
            SpindleResult::ok(1, 5000, 1800),
            SpindleResult::nok(2, 4500, 1750, true, false),
        ];

        let result = MultiSpindleResult::new(1, 100, spindles);
        assert_eq!(result.spindle_count, 2);
        assert!(!result.is_ok());
        assert_eq!(result.overall_status, 1); // NOK
        assert_eq!(result.ok_count(), 1);
        assert_eq!(result.nok_count(), 1);
    }

    #[test]
    fn test_multi_spindle_status_transitions() {
        let waiting = MultiSpindleStatus::waiting(100, 4);
        assert_eq!(waiting.status, 0);
        assert_eq!(waiting.spindle_count, 4);

        let running = MultiSpindleStatus::running(100, 4);
        assert_eq!(running.status, 1);

        let completed = MultiSpindleStatus::completed(100, 4);
        assert_eq!(completed.status, 2);
    }

    #[test]
    fn test_generate_multi_spindle_results() {
        let config = MultiSpindleConfig::new(4, 100);
        let result = generate_multi_spindle_results(&config, 1, 42);

        assert_eq!(result.spindle_count, 4);
        assert_eq!(result.spindle_results.len(), 4);
        assert_eq!(result.sync_id, 100);

        // Check spindle IDs are sequential
        for (idx, spindle) in result.spindle_results.iter().enumerate() {
            assert_eq!(spindle.spindle_id, (idx + 1) as u8);
        }
    }

    #[test]
    fn test_generate_multi_spindle_results_variation() {
        let config = MultiSpindleConfig::new(3, 200);
        let result = generate_multi_spindle_results(&config, 5, 10);

        // Each spindle should have different torque/angle values
        let torques: Vec<i32> = result.spindle_results.iter().map(|s| s.torque).collect();
        assert_ne!(torques[0], torques[1]);
        assert_ne!(torques[1], torques[2]);
    }

    #[test]
    fn test_generate_multi_spindle_results_occasional_failure() {
        let config = MultiSpindleConfig::new(2, 300);

        // Result ID divisible by 10 should cause last spindle to fail
        let result_fail = generate_multi_spindle_results(&config, 10, 1);
        assert!(!result_fail.is_ok());
        assert!(!result_fail.spindle_results[1].is_ok());

        // Other result IDs should all pass
        let result_ok = generate_multi_spindle_results(&config, 11, 1);
        assert!(result_ok.is_ok());
    }
}
