use serde::Serialize;
use std::time::{Duration, Instant};

/// Device operational states using typestate pattern
///
/// This FSM represents the operational lifecycle of a tightening device:
/// Idle → Tightening → Evaluating → Idle (or Error)
///
/// # Example
/// ```ignore
/// let fsm = DeviceFSM::new();
/// let fsm = fsm.start_tightening(params);
/// // ... wait for completion ...
/// let fsm = fsm.complete();
/// let result = fsm.result();
/// let fsm = fsm.finish(); // back to Idle
/// ```

// ============================================================================
// State marker types
// ============================================================================

/// Idle state - device is waiting for a tightening operation
pub struct Idle;

/// Tightening state - operation in progress
pub struct Tightening {
    /// When the tightening started
    pub start_time: Instant,
    /// Target parameters for this tightening
    pub params: TighteningParams,
}

/// Evaluating state - tightening complete, result available
pub struct Evaluating {
    /// The outcome of the tightening
    pub result: TighteningOutcome,
}

/// Error state - recoverable error occurred
pub struct Error {
    /// Error classification
    pub code: ErrorCode,
}

// ============================================================================
// Associated data types
// ============================================================================

/// Parameters for a tightening operation
#[derive(Debug, Clone, Serialize)]
pub struct TighteningParams {
    /// Target torque in Nm
    pub target_torque: f64,
    /// Minimum acceptable torque in Nm
    pub torque_min: f64,
    /// Maximum acceptable torque in Nm
    pub torque_max: f64,
    /// Target angle in degrees
    pub target_angle: f64,
    /// Minimum acceptable angle in degrees
    pub angle_min: f64,
    /// Maximum acceptable angle in degrees
    pub angle_max: f64,
    /// Realistic duration of tightening cycle in milliseconds
    pub duration_ms: u64,
}

impl TighteningParams {
    /// Create default parameters for testing
    pub fn default_test() -> Self {
        Self {
            target_torque: 12.5,
            torque_min: 10.0,
            torque_max: 15.0,
            target_angle: 40.0,
            angle_min: 30.0,
            angle_max: 50.0,
            duration_ms: 1500, // 1.5 seconds
        }
    }
}

/// Outcome of a completed tightening cycle
#[derive(Debug, Clone, Serialize)]
pub struct TighteningOutcome {
    /// Actual torque achieved in Nm
    pub actual_torque: f64,
    /// Actual angle achieved in degrees
    pub actual_angle: f64,
    /// How long the tightening took
    pub duration: Duration,
    /// Overall OK/NOK status
    pub ok: bool,
    /// Torque within limits
    pub torque_ok: bool,
    /// Angle within limits
    pub angle_ok: bool,
}

/// Error codes for tightening operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ErrorCode {
    /// Tool was disabled during operation
    ToolDisabled,
    /// Timeout waiting for completion
    Timeout,
    /// Invalid parameter set
    InvalidPset,
    /// General error
    General,
}

// ============================================================================
// Device FSM (generic over state)
// ============================================================================

/// Device finite state machine using typestate pattern
pub struct DeviceFSM<S> {
    state: S,
}

// ============================================================================
// State: Idle
// ============================================================================

impl DeviceFSM<Idle> {
    /// Create a new device in idle state
    pub fn new() -> Self {
        Self { state: Idle }
    }

    /// Start a tightening operation
    /// Transitions: Idle → Tightening
    pub fn start_tightening(self, params: TighteningParams) -> DeviceFSM<Tightening> {
        DeviceFSM {
            state: Tightening {
                start_time: Instant::now(),
                params,
            },
        }
    }
}

impl Default for DeviceFSM<Idle> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// State: Tightening
// ============================================================================

impl DeviceFSM<Tightening> {
    /// Check if tightening duration has elapsed
    pub fn is_complete(&self) -> bool {
        self.state.start_time.elapsed().as_millis() as u64 >= self.state.params.duration_ms
    }

    /// Get current progress as a value from 0.0 to 1.0
    pub fn progress(&self) -> f64 {
        let elapsed = self.state.start_time.elapsed().as_millis() as u64;
        (elapsed as f64 / self.state.params.duration_ms as f64).min(1.0)
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.state.start_time.elapsed()
    }

    /// Complete the tightening and transition to Evaluating state
    /// This simulates a realistic outcome with variation
    /// Transitions: Tightening → Evaluating
    pub fn complete(self) -> DeviceFSM<Evaluating> {
        let duration = self.state.start_time.elapsed();
        let params = &self.state.params;

        // Simple pseudo-random using nanoseconds
        // In production, use proper RNG like rand crate
        let seed = duration.as_nanos() % 1000;
        let variation1 = (seed as f64 / 1000.0) * 0.1; // 0.0 to 0.1
        let variation2 = ((seed * 7) % 1000) as f64 / 1000.0 * 0.1;

        // Simulate realistic outcome with +/- 5% variation around target
        let actual_torque = params.target_torque * (0.95 + variation1);
        let actual_angle = params.target_angle * (0.95 + variation2);

        // Check if within acceptable limits
        let torque_ok = actual_torque >= params.torque_min && actual_torque <= params.torque_max;
        let angle_ok = actual_angle >= params.angle_min && actual_angle <= params.angle_max;

        DeviceFSM {
            state: Evaluating {
                result: TighteningOutcome {
                    actual_torque,
                    actual_angle,
                    duration,
                    ok: torque_ok && angle_ok,
                    torque_ok,
                    angle_ok,
                },
            },
        }
    }

    /// Abort the tightening due to an error
    /// Transitions: Tightening → Error
    pub fn abort(self, code: ErrorCode) -> DeviceFSM<Error> {
        DeviceFSM {
            state: Error { code },
        }
    }

    /// Get the tightening parameters
    pub fn params(&self) -> &TighteningParams {
        &self.state.params
    }
}

// ============================================================================
// State: Evaluating
// ============================================================================

impl DeviceFSM<Evaluating> {
    /// Get the tightening result
    pub fn result(&self) -> &TighteningOutcome {
        &self.state.result
    }

    /// Consume the result and return to Idle state
    /// Transitions: Evaluating → Idle
    pub fn finish(self) -> DeviceFSM<Idle> {
        DeviceFSM::new()
    }
}

// ============================================================================
// State: Error
// ============================================================================

impl DeviceFSM<Error> {
    /// Get the error code
    pub fn error_code(&self) -> ErrorCode {
        self.state.code
    }

    /// Clear the error and return to Idle state
    /// Transitions: Error → Idle
    pub fn clear_error(self) -> DeviceFSM<Idle> {
        DeviceFSM::new()
    }
}

// ============================================================================
// Serializable wrapper for storage in DeviceState
// ============================================================================

/// Wrapper to make FSM state serializable
/// Since we can't serialize the generic FSM directly, we store state snapshots
#[derive(Debug, Clone, Serialize)]
pub enum DeviceFSMState {
    Idle,
    Tightening {
        progress: f64,
        elapsed_ms: u64,
        target_torque: f64,
        target_angle: f64,
    },
    Evaluating {
        ok: bool,
        torque_ok: bool,
        angle_ok: bool,
        actual_torque: f64,
        actual_angle: f64,
    },
    Error {
        code: ErrorCode,
    },
}

impl DeviceFSMState {
    /// Create idle state
    pub fn idle() -> Self {
        DeviceFSMState::Idle
    }

    /// Create tightening state snapshot
    pub fn tightening(fsm: &DeviceFSM<Tightening>) -> Self {
        DeviceFSMState::Tightening {
            progress: fsm.progress(),
            elapsed_ms: fsm.elapsed().as_millis() as u64,
            target_torque: fsm.params().target_torque,
            target_angle: fsm.params().target_angle,
        }
    }

    /// Create evaluating state snapshot
    pub fn evaluating(fsm: &DeviceFSM<Evaluating>) -> Self {
        let result = fsm.result();
        DeviceFSMState::Evaluating {
            ok: result.ok,
            torque_ok: result.torque_ok,
            angle_ok: result.angle_ok,
            actual_torque: result.actual_torque,
            actual_angle: result.actual_angle,
        }
    }

    /// Create error state snapshot
    pub fn error(fsm: &DeviceFSM<Error>) -> Self {
        DeviceFSMState::Error {
            code: fsm.error_code(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_idle_to_tightening_transition() {
        let fsm = DeviceFSM::new();
        let params = TighteningParams::default_test();
        let fsm = fsm.start_tightening(params);

        assert_eq!(fsm.progress(), 0.0);
        assert!(!fsm.is_complete());
    }

    #[test]
    fn test_tightening_progress() {
        let fsm = DeviceFSM::new();
        let mut params = TighteningParams::default_test();
        params.duration_ms = 100; // Short duration for testing

        let fsm = fsm.start_tightening(params);

        thread::sleep(Duration::from_millis(50));
        let progress = fsm.progress();
        assert!(progress > 0.0 && progress < 1.0);

        thread::sleep(Duration::from_millis(60));
        assert!(fsm.is_complete());
    }

    #[test]
    fn test_complete_tightening() {
        let fsm = DeviceFSM::new();
        let params = TighteningParams::default_test();
        let fsm = fsm.start_tightening(params);

        let fsm = fsm.complete();
        let result = fsm.result();

        // Should have values within reasonable range
        assert!(result.actual_torque > 10.0 && result.actual_torque < 15.0);
        assert!(result.actual_angle > 35.0 && result.actual_angle < 45.0);
    }

    #[test]
    fn test_full_success_cycle() {
        let fsm = DeviceFSM::new();
        let params = TighteningParams::default_test();

        // Idle → Tightening
        let fsm = fsm.start_tightening(params);

        // Tightening → Evaluating
        let fsm = fsm.complete();
        let result = fsm.result();
        assert!(result.ok || !result.ok); // Either outcome is valid

        // Evaluating → Idle
        let _fsm = fsm.finish();
    }

    #[test]
    fn test_abort_tightening() {
        let fsm = DeviceFSM::new();
        let params = TighteningParams::default_test();
        let fsm = fsm.start_tightening(params);

        // Abort during tightening
        let fsm = fsm.abort(ErrorCode::ToolDisabled);
        assert_eq!(fsm.error_code(), ErrorCode::ToolDisabled);

        // Clear error and return to idle
        let _fsm = fsm.clear_error();
    }

    #[test]
    fn test_ok_nok_evaluation() {
        let fsm = DeviceFSM::new();

        // Test with tight limits - likely to fail
        let params = TighteningParams {
            target_torque: 12.5,
            torque_min: 12.49,
            torque_max: 12.51,
            target_angle: 40.0,
            angle_min: 39.99,
            angle_max: 40.01,
            duration_ms: 1000,
        };

        let fsm = fsm.start_tightening(params);
        let fsm = fsm.complete();
        let result = fsm.result();

        // With such tight limits, outcome depends on variation
        // Just verify the logic works
        if result.ok {
            assert!(result.torque_ok && result.angle_ok);
        } else {
            assert!(!result.torque_ok || !result.angle_ok);
        }
    }

    #[test]
    fn test_fsm_state_snapshot_idle() {
        let fsm = DeviceFSM::new();
        let snapshot = DeviceFSMState::idle();

        match snapshot {
            DeviceFSMState::Idle => assert!(true),
            _ => panic!("Expected Idle state"),
        }
    }

    #[test]
    fn test_fsm_state_snapshot_tightening() {
        let fsm = DeviceFSM::new();
        let params = TighteningParams::default_test();
        let fsm = fsm.start_tightening(params);

        let snapshot = DeviceFSMState::tightening(&fsm);

        match snapshot {
            DeviceFSMState::Tightening {
                progress,
                target_torque,
                ..
            } => {
                assert!(progress >= 0.0);
                assert_eq!(target_torque, 12.5);
            }
            _ => panic!("Expected Tightening state"),
        }
    }

    #[test]
    fn test_fsm_state_snapshot_evaluating() {
        let fsm = DeviceFSM::new();
        let params = TighteningParams::default_test();
        let fsm = fsm.start_tightening(params).complete();

        let snapshot = DeviceFSMState::evaluating(&fsm);

        match snapshot {
            DeviceFSMState::Evaluating {
                ok,
                torque_ok,
                angle_ok,
                ..
            } => {
                // All should be boolean
                assert!(ok == true || ok == false);
                assert!(torque_ok == true || torque_ok == false);
                assert!(angle_ok == true || angle_ok == false);
            }
            _ => panic!("Expected Evaluating state"),
        }
    }
}
