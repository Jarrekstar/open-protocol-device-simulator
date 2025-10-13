use serde::Serialize;

/// Represents the status of a batch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BatchStatus {
    /// Batch is not yet finished
    NotFinished,
    /// Batch completed with all tightenings OK
    CompletedOk,
    /// Batch completed but has one or more NOK tightenings
    CompletedNok,
}

/// Information about a tightening operation within a batch
#[derive(Debug, Clone)]
pub struct TighteningInfo {
    /// Current position in the batch (1-based)
    pub counter: u32,
    /// Unique tightening identifier
    pub tightening_id: u32,
    /// Current batch status
    pub batch_status: BatchStatus,
}

/// Manages batch state and lifecycle
#[derive(Debug, Clone, Serialize)]
pub struct BatchManager {
    /// Current tightening counter within the batch
    counter: u32,
    /// Target number of tightenings for this batch
    target_size: u32,
    /// Whether the batch is completed
    completed: bool,
    /// Whether any tightening in this batch has been NOK
    has_nok: bool,
    /// Global tightening ID sequence counter
    tightening_sequence: u32,
}

impl BatchManager {
    /// Create a new batch manager with the specified target size
    pub fn new(target_size: u32) -> Self {
        Self {
            counter: 0,
            target_size,
            completed: false,
            has_nok: false,
            tightening_sequence: 0,
        }
    }

    /// Add a tightening result to the batch
    /// Returns information about the tightening and current batch status
    ///
    /// Note: Counter only increments on OK tightenings. NOK tightenings
    /// don't advance position - this allows integrator to retry same position.
    pub fn add_tightening(&mut self, result_ok: bool) -> TighteningInfo {
        // Always increment tightening ID (unique sequence)
        self.tightening_sequence += 1;

        // Only increment counter on OK (NOK allows retry at same position)
        if result_ok {
            self.counter += 1;
        } else {
            self.has_nok = true;
        }

        // Check if batch is complete (based on OK count)
        let batch_status = if self.counter >= self.target_size {
            self.completed = true;
            if self.has_nok {
                BatchStatus::CompletedNok
            } else {
                BatchStatus::CompletedOk
            }
        } else {
            BatchStatus::NotFinished
        };

        TighteningInfo {
            counter: self.counter,
            tightening_id: self.tightening_sequence,
            batch_status,
        }
    }

    /// Reset the batch for a new cycle
    /// Keeps the tightening_sequence counter but resets batch state
    pub fn reset(&mut self) {
        self.counter = 0;
        self.completed = false;
        self.has_nok = false;
    }

    /// Set a new target batch size
    /// Resets the current batch if the size changes
    pub fn set_target_size(&mut self, new_size: u32) {
        if new_size != self.target_size {
            self.target_size = new_size;
            self.reset();
        }
    }

    /// Get the batch status value for MID 0061 parameter 22
    /// Returns: 0 (NOK), 1 (OK), or 2 (not finished)
    pub fn get_batch_status_value(&self) -> i32 {
        if !self.completed {
            2  // Not finished
        } else if self.has_nok {
            0  // Batch NOK
        } else {
            1  // Batch OK
        }
    }

    /// Check if the batch is complete
    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Get the current counter value
    pub fn counter(&self) -> u32 {
        self.counter
    }

    /// Get the target batch size
    pub fn target_size(&self) -> u32 {
        self.target_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_batch_completion() {
        let mut manager = BatchManager::new(3);

        let info1 = manager.add_tightening(true);
        assert_eq!(info1.counter, 1);
        assert_eq!(info1.batch_status, BatchStatus::NotFinished);

        let info2 = manager.add_tightening(true);
        assert_eq!(info2.counter, 2);
        assert_eq!(info2.batch_status, BatchStatus::NotFinished);

        let info3 = manager.add_tightening(true);
        assert_eq!(info3.counter, 3);
        assert_eq!(info3.batch_status, BatchStatus::CompletedOk);
        assert!(manager.is_complete());
    }

    #[test]
    fn test_batch_with_nok() {
        let mut manager = BatchManager::new(3);

        let info1 = manager.add_tightening(true);
        assert_eq!(info1.counter, 1);

        let info2 = manager.add_tightening(false);  // NOK
        assert_eq!(info2.counter, 1);  // Counter doesn't increment on NOK

        let info3 = manager.add_tightening(true);
        assert_eq!(info3.counter, 2);  // Now increments

        let info4 = manager.add_tightening(true);
        assert_eq!(info4.counter, 3);
        assert_eq!(info4.batch_status, BatchStatus::CompletedNok);
        assert_eq!(manager.get_batch_status_value(), 0);  // NOK = 0
    }

    #[test]
    fn test_batch_status_values() {
        let mut manager = BatchManager::new(2);

        manager.add_tightening(true);
        assert_eq!(manager.get_batch_status_value(), 2);  // Not finished

        manager.add_tightening(true);
        assert_eq!(manager.get_batch_status_value(), 1);  // OK = 1

        manager.reset();
        manager.add_tightening(false);  // NOK, counter stays 0
        manager.add_tightening(true);   // OK, counter = 1
        manager.add_tightening(true);   // OK, counter = 2, complete
        assert_eq!(manager.get_batch_status_value(), 0);  // NOK = 0 (had NOK during batch)
    }

    #[test]
    fn test_batch_reset() {
        let mut manager = BatchManager::new(2);

        manager.add_tightening(true);   // counter: 1
        manager.add_tightening(true);   // counter: 2
        assert!(manager.is_complete());

        manager.reset();
        assert!(!manager.is_complete());
        assert_eq!(manager.counter(), 0);

        let info = manager.add_tightening(true);
        assert_eq!(info.counter, 1);
        assert_eq!(info.batch_status, BatchStatus::NotFinished);
    }

    #[test]
    fn test_batch_size_change() {
        let mut manager = BatchManager::new(3);

        manager.add_tightening(true);
        manager.add_tightening(true);

        // Change batch size - should reset
        manager.set_target_size(4);
        assert_eq!(manager.counter(), 0);
        assert!(!manager.is_complete());
    }

    #[test]
    fn test_tightening_id_sequence() {
        let mut manager = BatchManager::new(2);

        let info1 = manager.add_tightening(true);
        let info2 = manager.add_tightening(true);

        assert_eq!(info1.tightening_id, 1);
        assert_eq!(info2.tightening_id, 2);

        // Reset batch but tightening ID continues
        manager.reset();
        let info3 = manager.add_tightening(true);
        assert_eq!(info3.tightening_id, 3);
    }

    #[test]
    fn test_single_tightening_batch() {
        let mut manager = BatchManager::new(1);

        let info = manager.add_tightening(true);
        assert_eq!(info.counter, 1);
        assert_eq!(info.batch_status, BatchStatus::CompletedOk);
        assert!(manager.is_complete());
    }

    #[test]
    fn test_nok_doesnt_increment_counter() {
        let mut manager = BatchManager::new(3);

        let info1 = manager.add_tightening(true);
        assert_eq!(info1.counter, 1);

        // NOK doesn't increment counter
        let info2 = manager.add_tightening(false);
        assert_eq!(info2.counter, 1);  // Still 1!
        assert_eq!(info2.tightening_id, 2);  // But ID still increments

        // Retry same position - now OK
        let info3 = manager.add_tightening(true);
        assert_eq!(info3.counter, 2);  // Now 2

        let info4 = manager.add_tightening(true);
        assert_eq!(info4.counter, 3);
        assert_eq!(info4.batch_status, BatchStatus::CompletedNok);
    }

    #[test]
    fn test_tightening_id_always_increments() {
        let mut manager = BatchManager::new(2);

        let info1 = manager.add_tightening(true);
        assert_eq!(info1.tightening_id, 1);

        // Even NOK increments tightening ID
        let info2 = manager.add_tightening(false);
        assert_eq!(info2.tightening_id, 2);

        // Retry
        let info3 = manager.add_tightening(true);
        assert_eq!(info3.tightening_id, 3);

        let info4 = manager.add_tightening(true);
        assert_eq!(info4.tightening_id, 4);
    }

    #[test]
    fn test_multiple_nok_retries() {
        let mut manager = BatchManager::new(2);

        // Position 1: OK first try
        let info1 = manager.add_tightening(true);
        assert_eq!(info1.counter, 1);
        assert_eq!(info1.tightening_id, 1);

        // Position 2: NOK, retry twice before OK
        let info2 = manager.add_tightening(false);
        assert_eq!(info2.counter, 1);  // Stays at 1
        assert_eq!(info2.tightening_id, 2);

        let info3 = manager.add_tightening(false);
        assert_eq!(info3.counter, 1);  // Still 1
        assert_eq!(info3.tightening_id, 3);

        let info4 = manager.add_tightening(true);
        assert_eq!(info4.counter, 2);  // Finally advances
        assert_eq!(info4.tightening_id, 4);
        assert_eq!(info4.batch_status, BatchStatus::CompletedNok);
    }

    #[test]
    fn test_batch_completion_with_retry() {
        let mut manager = BatchManager::new(4);

        // Simulate: OK, NOK, OK (retry), OK, OK
        manager.add_tightening(true);   // counter: 1
        manager.add_tightening(false);  // counter: 1 (stays)
        manager.add_tightening(true);   // counter: 2 (retry succeeds)
        manager.add_tightening(true);   // counter: 3

        let final_info = manager.add_tightening(true);  // counter: 4
        assert_eq!(final_info.counter, 4);
        assert_eq!(final_info.tightening_id, 5);  // 5 total attempts
        assert_eq!(final_info.batch_status, BatchStatus::CompletedNok);
        assert!(manager.is_complete());
    }
}
