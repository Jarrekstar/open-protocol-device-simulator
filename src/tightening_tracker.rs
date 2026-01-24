use crate::batch_manager::{BatchManager, BatchStatus, TighteningInfo};
use serde::Serialize;

/// Operating mode for tightening operations
#[derive(Debug, Clone, Serialize)]
pub enum TighteningMode {
    /// Single bolt mode: each tightening is independent
    /// Integrator controls each bolt individually (pset selection + tool enable/disable)
    /// No batch tracking - device is stateless
    Single,
    /// Batch mode: multiple tightenings tracked together
    /// Triggered when integrator sends MID 0019 (set batch size)
    /// Device tracks progress through the batch
    Batch(BatchManager),
}

/// Tracks tightening operations across both single and batch modes
#[derive(Debug, Clone, Serialize)]
pub struct TighteningTracker {
    mode: TighteningMode,
    tightening_sequence: u32, // Global counter across all modes
}

impl TighteningTracker {
    /// Create new tracker in single mode (default)
    pub fn new() -> Self {
        Self {
            mode: TighteningMode::Single,
            tightening_sequence: 0,
        }
    }

    /// Enable batch mode with specified size (triggered by MID 0019)
    /// Always resets batch state - MID 0019 = "start new batch"
    pub fn enable_batch(&mut self, size: u32) {
        self.mode = TighteningMode::Batch(BatchManager::new(size));
    }

    /// Check if in batch mode
    ///
    /// Mode query method for tightening operation state.
    /// Used by webUI mode indicator to display "Single" vs "Batch"
    /// operation mode, and by HTTP API status endpoints for mode reporting.
    #[allow(dead_code)]
    pub fn is_batch_mode(&self) -> bool {
        matches!(self.mode, TighteningMode::Batch(_))
    }

    /// Add a tightening result
    /// Returns information about the tightening including batch status
    pub fn add_tightening(&mut self, ok: bool) -> TighteningInfo {
        self.tightening_sequence += 1;

        match &mut self.mode {
            TighteningMode::Single => {
                // Single mode: always report zeros, status "not used"
                TighteningInfo {
                    counter: 0,
                    tightening_id: self.tightening_sequence,
                    batch_status: BatchStatus::NotUsed,
                }
            }
            TighteningMode::Batch(batch_manager) => {
                // Batch mode: delegate to BatchManager but override tightening_id with global sequence
                let mut info = batch_manager.add_tightening(ok);
                info.tightening_id = self.tightening_sequence;
                info
            }
        }
    }

    /// Get batch size for MID 0061 reporting
    /// Returns 0 in single mode, target_size in batch mode
    pub fn batch_size(&self) -> u32 {
        match &self.mode {
            TighteningMode::Single => 0,
            TighteningMode::Batch(batch) => batch.target_size(),
        }
    }

    /// Get counter value
    /// Returns 0 in single mode, batch counter in batch mode
    pub fn counter(&self) -> u32 {
        match &self.mode {
            TighteningMode::Single => 0,
            TighteningMode::Batch(batch) => batch.counter(),
        }
    }

    /// Check if should wait for new batch configuration
    /// Returns false in single mode (never waits, integrator controls via tool enable/disable)
    /// Returns true in batch mode when batch is complete
    pub fn should_wait_for_config(&self) -> bool {
        match &self.mode {
            TighteningMode::Single => false, // Never wait in single mode
            TighteningMode::Batch(batch) => batch.is_complete(),
        }
    }

    /// Get remaining work for auto-tightening
    /// Returns None in single mode (infinite work), Some(n) in batch mode
    pub fn remaining_work(&self) -> Option<u32> {
        match &self.mode {
            TighteningMode::Single => None, // No concept of "remaining" in single mode
            TighteningMode::Batch(batch) => {
                Some(batch.target_size().saturating_sub(batch.counter()))
            }
        }
    }

    /// Check if batch is complete (only relevant in batch mode)
    pub fn is_complete(&self) -> bool {
        match &self.mode {
            TighteningMode::Single => false, // Never "complete" in single mode
            TighteningMode::Batch(batch) => batch.is_complete(),
        }
    }

    /// Get global tightening sequence number
    ///
    /// Global counter query for statistics and reporting.
    /// Used by webUI statistics dashboard to display total tightening
    /// count across all modes, and by HTTP API metrics endpoints.
    #[allow(dead_code)]
    pub fn tightening_sequence(&self) -> u32 {
        self.tightening_sequence
    }

    /// Increment the batch counter without a tightening result (MID 0128).
    /// Used to skip a bolt position (e.g., after max retries on integrator side).
    /// Returns the new counter value, or 0 if not in batch mode.
    pub fn increment_batch(&mut self) -> u32 {
        match &mut self.mode {
            TighteningMode::Single => 0, // No-op in single mode
            TighteningMode::Batch(batch_manager) => batch_manager.increment(),
        }
    }

    /// Reset the batch counter (MID 0020).
    /// Resets the counter to 0 without changing batch size.
    /// Returns true if in batch mode, false if in single mode.
    pub fn reset_batch(&mut self) -> bool {
        match &mut self.mode {
            TighteningMode::Single => false, // No-op in single mode
            TighteningMode::Batch(batch_manager) => {
                batch_manager.reset();
                true
            }
        }
    }
}

impl Default for TighteningTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_single_mode() {
        let tracker = TighteningTracker::new();
        assert!(!tracker.is_batch_mode());
        assert_eq!(tracker.batch_size(), 0);
        assert_eq!(tracker.counter(), 0);
        assert_eq!(tracker.remaining_work(), None);
        assert!(!tracker.should_wait_for_config());
    }

    #[test]
    fn test_single_mode_tightening() {
        let mut tracker = TighteningTracker::new();

        let info1 = tracker.add_tightening(true);
        assert_eq!(info1.counter, 0);
        assert_eq!(info1.batch_status, BatchStatus::NotUsed);
        assert_eq!(info1.tightening_id, 1);

        let info2 = tracker.add_tightening(false);
        assert_eq!(info2.counter, 0);
        assert_eq!(info2.batch_status, BatchStatus::NotUsed);
        assert_eq!(info2.tightening_id, 2);

        // Counter stays 0 in single mode
        assert_eq!(tracker.counter(), 0);
        assert_eq!(tracker.batch_size(), 0);
    }

    #[test]
    fn test_enable_batch_mode() {
        let mut tracker = TighteningTracker::new();
        tracker.enable_batch(4);

        assert!(tracker.is_batch_mode());
        assert_eq!(tracker.batch_size(), 4);
        assert_eq!(tracker.counter(), 0);
        assert_eq!(tracker.remaining_work(), Some(4));
    }

    #[test]
    fn test_batch_mode_tightening() {
        let mut tracker = TighteningTracker::new();
        tracker.enable_batch(3);

        let info1 = tracker.add_tightening(true);
        assert_eq!(info1.counter, 1);
        assert_eq!(info1.tightening_id, 1);
        assert_eq!(info1.batch_status, BatchStatus::NotFinished);
        assert_eq!(tracker.remaining_work(), Some(2));

        let info2 = tracker.add_tightening(true);
        assert_eq!(info2.counter, 2);
        assert_eq!(info2.tightening_id, 2);
        assert_eq!(tracker.remaining_work(), Some(1));

        let info3 = tracker.add_tightening(true);
        assert_eq!(info3.counter, 3);
        assert_eq!(info3.tightening_id, 3);
        assert_eq!(info3.batch_status, BatchStatus::CompletedOk);
        assert_eq!(tracker.remaining_work(), Some(0));
        assert!(tracker.is_complete());
        assert!(tracker.should_wait_for_config());
    }

    #[test]
    fn test_batch_mode_with_nok() {
        let mut tracker = TighteningTracker::new();
        tracker.enable_batch(2);

        tracker.add_tightening(true); // counter: 1
        tracker.add_tightening(false); // counter: 1 (stays)

        assert_eq!(tracker.counter(), 1);
        assert_eq!(tracker.remaining_work(), Some(1));
        assert!(!tracker.is_complete());
        assert!(!tracker.should_wait_for_config());
    }

    #[test]
    fn test_switch_from_single_to_batch() {
        let mut tracker = TighteningTracker::new();

        // Do some single-mode tightenings
        tracker.add_tightening(true);
        tracker.add_tightening(true);
        assert_eq!(tracker.tightening_sequence(), 2);

        // Switch to batch mode
        tracker.enable_batch(3);

        let info = tracker.add_tightening(true);
        assert_eq!(info.tightening_id, 3); // Sequence continues
        assert_eq!(info.counter, 1); // Batch counter starts fresh
    }

    #[test]
    fn test_enable_batch_while_in_batch_resets() {
        let mut tracker = TighteningTracker::new();
        tracker.enable_batch(4);

        tracker.add_tightening(true);
        tracker.add_tightening(true);
        assert_eq!(tracker.counter(), 2);

        // Enable new batch
        tracker.enable_batch(6);
        assert_eq!(tracker.counter(), 0); // Reset
        assert_eq!(tracker.batch_size(), 6);
    }

    #[test]
    fn test_single_mode_never_waits() {
        let mut tracker = TighteningTracker::new();

        // Do many tightenings in single mode
        for _ in 0..10 {
            tracker.add_tightening(true);
        }

        // Should never signal to wait
        assert!(!tracker.should_wait_for_config());
        assert_eq!(tracker.remaining_work(), None);
    }

    #[test]
    fn test_batch_completion_signals_wait() {
        let mut tracker = TighteningTracker::new();
        tracker.enable_batch(2);

        tracker.add_tightening(true);
        assert!(!tracker.should_wait_for_config());

        tracker.add_tightening(true);
        assert!(tracker.should_wait_for_config()); // Batch complete, should wait
    }
}
