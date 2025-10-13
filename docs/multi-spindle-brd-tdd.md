# Multi-Spindle Mode: Business Requirements & Technical Design Document

**Status**: Planning
**Date**: 2025-10-13
**Version**: 1.0

---

## Table of Contents

1. [Business Requirements Document (BRD)](#business-requirements-document-brd)
2. [Technical Design Document (TDD)](#technical-design-document-tdd)
3. [Implementation Plan](#implementation-plan)
4. [Test Scenarios](#test-scenarios)
5. [Open Protocol Specification Reference](#open-protocol-specification-reference)

---

## Business Requirements Document (BRD)

### 1. Executive Summary

**Objective**: Extend the Open Protocol Device Simulator to support multi-spindle synchronized tightening operations, enabling realistic simulation of assembly lines with 2-10 synchronized tightening tools.

**Business Value**:
- Test integrators without access to expensive multi-spindle hardware
- Simulate complex synchronized assembly operations
- Validate integrator logic for handling multiple spindle failures
- Enable load testing with realistic multi-spindle scenarios

### 2. Background

In automotive and aerospace assembly, multiple tightening spindles often operate simultaneously in a synchronized manner. For example:
- Engine assembly: 4 spindles tightening cylinder head bolts
- Chassis assembly: 6 spindles for suspension components
- Aircraft assembly: 8-10 spindles for wing attachment

The Atlas Copco Open Protocol defines specific MIDs (0090-0103) for multi-spindle operations, but testing requires physical hardware. This simulator will enable software-only testing.

### 3. Stakeholders

- **Primary**: Integration engineers developing MES/SCADA systems
- **Secondary**: QA teams testing assembly line software
- **Tertiary**: Training personnel learning Open Protocol

### 4. Functional Requirements

#### FR-1: Multi-Spindle Configuration
- **REQ-1.1**: Support 2-10 synchronized spindles
- **REQ-1.2**: Auto-generate spindle numbers (1-N) and channel IDs (1-N)
- **REQ-1.3**: Configuration persists across simulator restarts
- **REQ-1.4**: Default mode remains single-spindle (backward compatible)

#### FR-2: Synchronized Tightening Operations
- **REQ-2.1**: All spindles tighten simultaneously (same sync ID)
- **REQ-2.2**: Each spindle has independent torque/angle values (±5% variation)
- **REQ-2.3**: Each spindle can fail independently (per-spindle failure rate)
- **REQ-2.4**: Overall status = OK only if ALL spindles succeed

#### FR-3: Open Protocol Message Support
- **REQ-3.1**: MID 0090 - Multi-spindle status subscribe
- **REQ-3.2**: MID 0091 - Multi-spindle status (lightweight: OK/NOK per spindle)
- **REQ-3.3**: MID 0092 - Multi-spindle status acknowledge
- **REQ-3.4**: MID 0093 - Multi-spindle status unsubscribe
- **REQ-3.5**: MID 0100 - Multi-spindle result subscribe
- **REQ-3.6**: MID 0101 - Multi-spindle result (full: torque/angle per spindle)
- **REQ-3.7**: MID 0102 - Multi-spindle result acknowledge
- **REQ-3.8**: MID 0103 - Multi-spindle result unsubscribe

#### FR-4: Batch Management Integration
- **REQ-4.1**: One sync tightening = one batch increment (not per spindle)
- **REQ-4.2**: If sync_overall_status = NOK → batch counter doesn't increment
- **REQ-4.3**: Retry logic: Integrator can retry failed sync operations
- **REQ-4.4**: Existing single-spindle batch logic remains unchanged

#### FR-5: Subscription Management
- **REQ-5.1**: Separate subscriptions for status (MID 0091) vs result (MID 0101)
- **REQ-5.2**: Only send subscribed message types
- **REQ-5.3**: Subscription state tracked per TCP client

#### FR-6: HTTP Configuration API
- **REQ-6.1**: Enable/disable multi-spindle mode via HTTP POST
- **REQ-6.2**: Configure spindle count (2-10)
- **REQ-6.3**: Query current multi-spindle configuration

### 5. Non-Functional Requirements

#### NFR-1: Performance
- Multi-spindle operations should not degrade performance vs single-spindle
- Target: Support 10 spindles with <5ms additional latency

#### NFR-2: Reliability
- Configuration changes should be atomic (all succeed or all fail)
- No data corruption if switching between single/multi-spindle modes

#### NFR-3: Compatibility
- Single-spindle mode remains default
- Existing integrators work without modifications
- MID 0061 (single-spindle result) still supported when multi-spindle disabled

#### NFR-4: Testability
- All multi-spindle logic unit testable
- Integration tests cover all MID combinations
- Failure scenarios reproducible (seed-based randomization)

### 6. User Stories

**US-1**: As an integration engineer, I want to enable 4-spindle mode so I can test my engine assembly integrator without physical hardware.

**US-2**: As a QA engineer, I want each spindle to fail independently so I can validate our error handling logic for partial failures.

**US-3**: As a developer, I want the batch counter to only increment when all spindles succeed so I can test retry workflows.

**US-4**: As a trainer, I want realistic multi-spindle variation so trainees understand real-world assembly operations.

### 7. Acceptance Criteria

- [ ] Can enable multi-spindle mode via HTTP API
- [ ] Auto-tightening generates synchronized results for all configured spindles
- [ ] Each spindle has independent ±5% torque/angle variation
- [ ] Each spindle can fail independently based on failure_rate
- [ ] Integrator receives MID 0091 when subscribed to status
- [ ] Integrator receives MID 0101 when subscribed to result
- [ ] Batch counter increments only when sync_overall_status = OK
- [ ] Single-spindle mode still works (backward compatible)
- [ ] Configuration persists across simulator restarts

### 8. Out of Scope (Future Enhancements)

- Press systems (force/stroke instead of torque/angle)
- Different torque targets per spindle
- Spindle-specific failure patterns (e.g., spindle 3 always fails)
- Advanced sync master/slave configuration
- Multi-station coordination

---

## Technical Design Document (TDD)

### 1. Architecture Overview

**Design Principle**: Extend existing single-spindle architecture with minimal disruption.

**Mode Toggle**: `multi_spindle_config.enabled` flag switches between single/multi-spindle operation.

**Component Diagram**:
```
┌─────────────────────────────────────────────────────────┐
│                    TCP Clients                          │
│                                                          │
│  MID 0090/0093 ──────┐                                  │
│  MID 0100/0103 ──────┼──► Handler Registry              │
│                      │                                   │
│                      └──► MultiSpindleConfig             │
│                                                          │
│  ┌────────────────────────────────────────┐             │
│  │  Auto-Tightening Loop                  │             │
│  │  ├─ Generate sync results (all spndles)│             │
│  │  ├─ Apply ±5% variation per spindle    │             │
│  │  ├─ Apply per-spindle failure logic    │             │
│  │  └─ Calculate sync_overall_status      │             │
│  └────────────────────────────────────────┘             │
│                      │                                   │
│                      ▼                                   │
│  ┌────────────────────────────────────────┐             │
│  │  Event Broadcaster                     │             │
│  │  ├─ MultiSpindleStatusCompleted (0091) │             │
│  │  └─ MultiSpindleResultCompleted (0101) │             │
│  └────────────────────────────────────────┘             │
│                      │                                   │
│                      ▼                                   │
│  Only send to subscribed clients                        │
└─────────────────────────────────────────────────────────┘
```

### 2. Data Structures

#### 2.1 Core Multi-Spindle Configuration (`src/multi_spindle.rs`)

```rust
use serde::{Deserialize, Serialize};

/// Multi-spindle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSpindleConfig {
    /// Whether multi-spindle mode is enabled
    pub enabled: bool,
    /// Number of spindles (2-10)
    pub spindle_count: u8,
    /// Sync tightening ID counter (increments after each sync operation)
    pub sync_tightening_id: u32,
}

impl MultiSpindleConfig {
    pub fn new(spindle_count: u8) -> Self {
        assert!(spindle_count >= 2 && spindle_count <= 10,
                "Spindle count must be 2-10");
        Self {
            enabled: true,
            spindle_count,
            sync_tightening_id: 0,
        }
    }

    /// Auto-generate spindles: numbers 1-N with channel IDs 1-N
    pub fn get_spindles(&self) -> Vec<SpindleInfo> {
        (1..=self.spindle_count)
            .map(|n| SpindleInfo {
                spindle_number: n,
                channel_id: n,
            })
            .collect()
    }

    /// Get next sync tightening ID (wraps at 65535)
    pub fn next_sync_id(&mut self) -> u32 {
        self.sync_tightening_id = (self.sync_tightening_id + 1) % 65536;
        self.sync_tightening_id
    }
}

impl Default for MultiSpindleConfig {
    fn default() -> Self {
        Self {
            enabled: false,  // Disabled by default (single-spindle mode)
            spindle_count: 1,
            sync_tightening_id: 0,
        }
    }
}

/// Information about a single spindle in the sync group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpindleInfo {
    pub spindle_number: u8,  // 01-99
    pub channel_id: u8,      // 01-20
}

/// Result of a single spindle in a synchronized tightening
#[derive(Debug, Clone, Serialize)]
pub struct SpindleResult {
    pub spindle_number: u8,
    pub channel_id: u8,
    pub overall_status: bool,
    pub torque_status: bool,
    pub torque: f64,
    pub angle_status: bool,
    pub angle: f64,
}

/// Complete multi-spindle tightening result
#[derive(Debug, Clone, Serialize)]
pub struct MultiSpindleResult {
    pub sync_id: u32,
    pub sync_overall_status: bool,  // OK only if ALL spindles OK
    pub spindle_results: Vec<SpindleResult>,
    // Common fields (same across all spindles)
    pub vin_number: Option<String>,
    pub job_id: u32,
    pub pset_id: u32,
    pub batch_size: u32,
    pub batch_counter: u32,
    pub batch_status: Option<bool>,
    pub torque_min: f64,
    pub torque_max: f64,
    pub torque_target: f64,
    pub angle_min: f64,
    pub angle_max: f64,
    pub angle_target: f64,
    pub timestamp: String,
    pub last_pset_change: Option<String>,
}

/// Lightweight multi-spindle status (for MID 0091)
#[derive(Debug, Clone, Serialize)]
pub struct MultiSpindleStatus {
    pub spindle_count: u8,
    pub sync_id: u32,
    pub timestamp: String,
    pub sync_overall_status: bool,
    pub spindle_results: Vec<SpindleStatusInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpindleStatusInfo {
    pub spindle_number: u8,
    pub channel_id: u8,
    pub overall_status: bool,
}

impl From<&MultiSpindleResult> for MultiSpindleStatus {
    fn from(result: &MultiSpindleResult) -> Self {
        Self {
            spindle_count: result.spindle_results.len() as u8,
            sync_id: result.sync_id,
            timestamp: result.timestamp.clone(),
            sync_overall_status: result.sync_overall_status,
            spindle_results: result.spindle_results.iter().map(|r| {
                SpindleStatusInfo {
                    spindle_number: r.spindle_number,
                    channel_id: r.channel_id,
                    overall_status: r.overall_status,
                }
            }).collect(),
        }
    }
}
```

#### 2.2 Result Generation Logic

```rust
/// Generate multi-spindle results with per-spindle variation and failures
pub fn generate_multi_spindle_results(
    config: &MultiSpindleConfig,
    base_torque: f64,
    base_angle: f64,
    torque_limits: (f64, f64, f64),  // (min, max, target)
    angle_limits: (f64, f64, f64),   // (min, max, target)
    failure_rate: f64,
    vin_number: Option<String>,
    job_id: u32,
    pset_id: u32,
    batch_info: (u32, u32, Option<bool>),  // (size, counter, status)
    timestamp: String,
    last_pset_change: Option<String>,
) -> MultiSpindleResult {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let spindles = config.get_spindles();
    let mut spindle_results = Vec::new();

    for spindle_info in spindles {
        // Each spindle gets ±5% variation from base values
        let variation = rng.gen_range(-0.05..=0.05);
        let torque = base_torque * (1.0 + variation);
        let angle = base_angle * (1.0 + variation);

        // Independent failure per spindle
        let spindle_ok = rng.gen::<f64>() > failure_rate;

        // Torque/angle status based on limits
        let torque_ok = spindle_ok && torque >= torque_limits.0 && torque <= torque_limits.1;
        let angle_ok = spindle_ok && angle >= angle_limits.0 && angle <= angle_limits.1;

        spindle_results.push(SpindleResult {
            spindle_number: spindle_info.spindle_number,
            channel_id: spindle_info.channel_id,
            overall_status: spindle_ok,
            torque_status: torque_ok,
            torque,
            angle_status: angle_ok,
            angle,
        });
    }

    // Sync overall status = OK only if ALL spindles OK
    let sync_overall_status = spindle_results.iter().all(|r| r.overall_status);

    MultiSpindleResult {
        sync_id: config.sync_tightening_id,
        sync_overall_status,
        spindle_results,
        vin_number,
        job_id,
        pset_id,
        batch_size: batch_info.0,
        batch_counter: batch_info.1,
        batch_status: batch_info.2,
        torque_min: torque_limits.0,
        torque_max: torque_limits.1,
        torque_target: torque_limits.2,
        angle_min: angle_limits.0,
        angle_max: angle_limits.1,
        angle_target: angle_limits.2,
        timestamp,
        last_pset_change,
    }
}
```

### 3. Protocol Message Serialization

#### 3.1 MID 0091 - Multi-spindle Status (Lightweight)

**Message Structure**:
```
Bytes 21-24: Parameter 01 - Number of spindles (01 + 02-10)
Bytes 25-31: Parameter 02 - Sync tightening ID (02 + 5 digits, range 00000-65535)
Bytes 32-52: Parameter 03 - Timestamp (03 + 19 chars YYYY-MM-DD:HH:MM:SS)
Bytes 53-55: Parameter 04 - Sync overall status (04 + 1 digit, 0=NOK, 1=OK)
Bytes 56+:   Parameter 05 - Spindle status (05 + 5 bytes per spindle)
             - Bytes 1-2: Spindle number (01-99)
             - Bytes 3-4: Channel ID (01-20)
             - Byte 5: Individual status (0=NOK, 1=OK)
```

**Implementation** (`src/handler/data/multi_spindle_status.rs`):
```rust
use crate::multi_spindle::MultiSpindleStatus;
use crate::protocol::field::FieldBuilder;
use crate::handler::data::Serializable;

impl Serializable for MultiSpindleStatus {
    fn serialize(&self) -> Vec<u8> {
        let mut builder = FieldBuilder::new();

        // Parameter 01: Number of spindles
        builder = builder.add_str(Some(1), &format!("{:02}", self.spindle_count), 2);

        // Parameter 02: Sync tightening ID
        builder = builder.add_str(Some(2), &format!("{:05}", self.sync_id), 5);

        // Parameter 03: Timestamp
        builder = builder.add_str(Some(3), &self.timestamp, 19);

        // Parameter 04: Sync overall status
        let sync_status = if self.sync_overall_status { "1" } else { "0" };
        builder = builder.add_str(Some(4), sync_status, 1);

        // Parameter 05: Spindle status (5 bytes per spindle)
        let mut spindle_data = String::new();
        for spindle in &self.spindle_results {
            spindle_data.push_str(&format!("{:02}", spindle.spindle_number));
            spindle_data.push_str(&format!("{:02}", spindle.channel_id));
            spindle_data.push_str(if spindle.overall_status { "1" } else { "0" });
        }
        builder = builder.add_str(Some(5), &spindle_data, spindle_data.len());

        builder.build()
    }
}
```

**Example Message** (2 spindles, both OK):
```
00670091001020200012032025-10-13:14:30:15041050120102041
│││││││││││││││││││││││││││││││││││││││││││││││││
│││││││││││││││││││││││││││││││││││││││││││││││││
│││││││││││││││││││││││││││││││││││││││││││└─ Spindle 2 OK
│││││││││││││││││││││││││││││││││││││││││└───┴─ Spindle 2 channel 02
│││││││││││││││││││││││││││││││││││││└───┴───── Spindle 2 number 02
│││││││││││││││││││││││││││││││││││└─────────── Spindle 1 OK
│││││││││││││││││││││││││││││││└───┴─────────── Spindle 1 channel 01
│││││││││││││││││││││││││││└───┴───────────────  Spindle 1 number 01
│││││││││││││││││││││││││└───────────────────── Param 05
│││││││││││││││││││││││└───────────────────────  Sync overall OK
│││││││││││││││││││││└─────────────────────────  Param 04
│││││││││││││└───────┴─────────────────────────  Timestamp
│││││││││││└───────────────────────────────────  Param 03
│││││││└───┴───────────────────────────────────  Sync ID 00001
│││││└─────────────────────────────────────────  Param 02
│││└─┴─────────────────────────────────────────  Number of spindles 02
│└─────────────────────────────────────────────  Param 01
└───────────────────────────────────────────────  MID 0091
```

#### 3.2 MID 0101 - Multi-spindle Result (Full Details)

**Message Structure** (Revision 4):
```
Parameters 1-17: Same as single-spindle (VIN, Job, Pset, Batch, Torque/Angle limits, etc.)
Parameter 18: Spindle results (18 bytes × number of spindles)
  Per spindle:
  - Bytes 1-2:   Spindle number (01-99)
  - Bytes 3-4:   Channel ID (01-99)
  - Byte 5:      Overall status (0=NOK, 1=OK)
  - Byte 6:      Torque status (0=NOK, 1=OK)
  - Bytes 7-12:  Torque value (×100, 6 ASCII digits)
  - Byte 13:     Angle status (0=NOK, 1=OK)
  - Bytes 14-18: Angle value (5 ASCII digits)
Parameter 19: System subtype (3 digits, always 001 for tightening)
```

**Implementation** (`src/handler/data/multi_spindle_result.rs`):
```rust
use crate::multi_spindle::MultiSpindleResult;
use crate::protocol::field::FieldBuilder;
use crate::handler::data::Serializable;

impl Serializable for MultiSpindleResult {
    fn serialize(&self) -> Vec<u8> {
        let mut builder = FieldBuilder::new();

        // Parameter 01: Number of spindles
        builder = builder.add_str(Some(1),
            &format!("{:02}", self.spindle_results.len()), 2);

        // Parameter 02: VIN Number
        let vin = self.vin_number.as_ref()
            .map(|v| format!("{:25}", v))
            .unwrap_or_else(|| " ".repeat(25));
        builder = builder.add_str(Some(2), &vin, 25);

        // Parameter 03: Job ID
        builder = builder.add_str(Some(3), &format!("{:02}", self.job_id), 2);

        // Parameter 04: Parameter Set ID
        builder = builder.add_str(Some(4), &format!("{:03}", self.pset_id), 3);

        // Parameter 05: Batch Size
        builder = builder.add_str(Some(5), &format!("{:04}", self.batch_size), 4);

        // Parameter 06: Batch Counter
        builder = builder.add_str(Some(6), &format!("{:04}", self.batch_counter), 4);

        // Parameter 07: Batch Status
        let batch_status = match self.batch_status {
            Some(true) => "1",   // OK
            Some(false) => "0",  // NOK
            None => "2",         // Not used
        };
        builder = builder.add_str(Some(7), batch_status, 1);

        // Parameters 08-13: Torque and Angle limits
        builder = builder.add_str(Some(8),
            &format!("{:06}", (self.torque_min * 100.0) as i32), 6);
        builder = builder.add_str(Some(9),
            &format!("{:06}", (self.torque_max * 100.0) as i32), 6);
        builder = builder.add_str(Some(10),
            &format!("{:06}", (self.torque_target * 100.0) as i32), 6);
        builder = builder.add_str(Some(11),
            &format!("{:05}", self.angle_min as u32), 5);
        builder = builder.add_str(Some(12),
            &format!("{:05}", self.angle_max as u32), 5);
        builder = builder.add_str(Some(13),
            &format!("{:05}", self.angle_target as u32), 5);

        // Parameter 14: Last Pset Change
        let pset_change = self.last_pset_change.as_ref()
            .map(|t| t.clone())
            .unwrap_or_else(|| " ".repeat(19));
        builder = builder.add_str(Some(14), &pset_change, 19);

        // Parameter 15: Timestamp
        builder = builder.add_str(Some(15), &self.timestamp, 19);

        // Parameter 16: Sync Tightening ID
        builder = builder.add_str(Some(16), &format!("{:05}", self.sync_id), 5);

        // Parameter 17: Sync Overall Status
        let sync_status = if self.sync_overall_status { "1" } else { "0" };
        builder = builder.add_str(Some(17), sync_status, 1);

        // Parameter 18: Spindle results (18 bytes per spindle)
        let mut spindle_data = String::new();
        for spindle in &self.spindle_results {
            spindle_data.push_str(&format!("{:02}", spindle.spindle_number));
            spindle_data.push_str(&format!("{:02}", spindle.channel_id));
            spindle_data.push_str(if spindle.overall_status { "1" } else { "0" });
            spindle_data.push_str(if spindle.torque_status { "1" } else { "0" });
            spindle_data.push_str(&format!("{:06}", (spindle.torque * 100.0) as i32));
            spindle_data.push_str(if spindle.angle_status { "1" } else { "0" });
            spindle_data.push_str(&format!("{:05}", spindle.angle as u32));
        }
        builder = builder.add_str(Some(18), &spindle_data, spindle_data.len());

        // Parameter 19: System subtype (001 = tightening system)
        builder = builder.add_str(Some(19), "001", 3);

        builder.build()
    }
}
```

### 4. Handler Implementations

#### 4.1 MID 0090 - Multi-spindle Status Subscribe

**File**: `src/handler/multi_spindle_status_subscribe.rs`

```rust
use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};
use crate::handler::data::command_accepted::CommandAccepted;

pub struct MultiSpindleStatusSubscribeHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl MultiSpindleStatusSubscribeHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for MultiSpindleStatusSubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        let mut state = self.state.write().unwrap();

        // Check if multi-spindle mode is enabled
        if !state.multi_spindle_config.enabled {
            return Err(HandlerError::InvalidRequest(
                "Controller is not in multi-spindle mode".to_string()
            ));
        }

        // Check if already subscribed
        if state.multi_spindle_status_subscribed {
            return Err(HandlerError::InvalidRequest(
                "Multi-spindle status subscription already exists".to_string()
            ));
        }

        state.multi_spindle_status_subscribed = true;
        println!("MID 0090: Multi-spindle status subscribed");

        let ack_data = CommandAccepted::with_mid(90);
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
```

#### 4.2 MID 0093 - Multi-spindle Status Unsubscribe

**File**: `src/handler/multi_spindle_status_unsubscribe.rs`

```rust
use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};
use crate::handler::data::command_accepted::CommandAccepted;

pub struct MultiSpindleStatusUnsubscribeHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl MultiSpindleStatusUnsubscribeHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for MultiSpindleStatusUnsubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        let mut state = self.state.write().unwrap();

        if !state.multi_spindle_status_subscribed {
            return Err(HandlerError::InvalidRequest(
                "Multi-spindle status subscription does not exist".to_string()
            ));
        }

        state.multi_spindle_status_subscribed = false;
        println!("MID 0093: Multi-spindle status unsubscribed");

        let ack_data = CommandAccepted::with_mid(93);
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
```

Similar implementations for MID 0100 (result subscribe) and MID 0103 (result unsubscribe).

### 5. DeviceState Integration

**Modify** `src/state.rs`:

```rust
use crate::multi_spindle::MultiSpindleConfig;

#[derive(Debug, Clone, Serialize)]
pub struct DeviceState {
    // Existing fields...
    pub batch_manager: BatchManager,
    pub device_fsm_state: DeviceFSMState,
    pub tool_enabled: bool,

    // Multi-spindle configuration
    pub multi_spindle_config: MultiSpindleConfig,
    pub multi_spindle_status_subscribed: bool,
    pub multi_spindle_result_subscribed: bool,
}

impl DeviceState {
    pub fn new() -> Self {
        Self {
            // Existing initialization...
            multi_spindle_config: MultiSpindleConfig::default(),
            multi_spindle_status_subscribed: false,
            multi_spindle_result_subscribed: false,
        }
    }

    /// Enable multi-spindle mode with specified spindle count
    pub fn enable_multi_spindle(&mut self, spindle_count: u8) {
        self.multi_spindle_config = MultiSpindleConfig::new(spindle_count);
    }

    /// Disable multi-spindle mode (return to single-spindle)
    pub fn disable_multi_spindle(&mut self) {
        self.multi_spindle_config.enabled = false;
    }
}
```

### 6. Event System Integration

**Modify** `src/events.rs`:

```rust
use crate::multi_spindle::{MultiSpindleStatus, MultiSpindleResult};

#[derive(Debug, Clone)]
pub enum SimulatorEvent {
    // Existing events...
    TighteningCompleted { result: TighteningResult },
    PsetChanged { pset_id: u32 },
    BatchCompleted { total: u32 },

    // Multi-spindle events
    MultiSpindleStatusCompleted { status: MultiSpindleStatus },
    MultiSpindleResultCompleted { result: MultiSpindleResult },
}
```

**Modify** `src/subscriptions.rs`:

```rust
pub struct Subscriptions {
    // Existing subscriptions...
    pset_selection: bool,
    tightening_result: bool,

    // Multi-spindle subscriptions
    multi_spindle_status: bool,
    multi_spindle_result: bool,
}

impl Subscriptions {
    pub fn subscribe_multi_spindle_status(&mut self) {
        self.multi_spindle_status = true;
    }

    pub fn unsubscribe_multi_spindle_status(&mut self) {
        self.multi_spindle_status = false;
    }

    pub fn is_subscribed_to_multi_spindle_status(&self) -> bool {
        self.multi_spindle_status
    }

    // Similar methods for multi_spindle_result
}
```

### 7. Auto-Tightening Integration

**Modify** `src/http_server.rs` - auto-tightening loop:

```rust
// After Phase 3 (tightening complete), check mode:
if state.multi_spindle_config.enabled {
    // ============================================================
    // MULTI-SPINDLE MODE
    // ============================================================

    let (multi_result, batch_counter, batch_completed) = {
        let mut s = state.write().unwrap();

        // Increment sync ID
        let sync_id = s.multi_spindle_config.next_sync_id();

        // Generate multi-spindle results
        let multi_result = generate_multi_spindle_results(
            &s.multi_spindle_config,
            outcome.actual_torque,
            outcome.actual_angle,
            (10.0, 15.0, 12.5),  // Torque limits
            (30.0, 50.0, 40.0),  // Angle limits
            failure_rate,
            s.vehicle_id.clone(),
            s.current_job_id.unwrap_or(1),
            s.current_pset_id.unwrap_or(1),
            (
                s.batch_manager.target_size(),
                s.batch_manager.counter(),
                None,  // Will be updated after batch logic
            ),
            chrono::Local::now().format("%Y-%m-%d:%H:%M:%S").to_string(),
            None,
        );

        // Use sync_overall_status for batch logic
        let info = s.batch_manager.add_tightening(multi_result.sync_overall_status);

        let batch_completed = s.batch_manager.is_complete();

        // Update batch_status in result
        let mut final_result = multi_result;
        final_result.batch_counter = info.counter;
        final_result.batch_status = match info.batch_status {
            crate::batch_manager::BatchStatus::NotFinished => None,
            crate::batch_manager::BatchStatus::CompletedOk => Some(true),
            crate::batch_manager::BatchStatus::CompletedNok => Some(false),
        };

        (final_result, info.counter, batch_completed)
    };

    // Broadcast events based on subscriptions
    if state.multi_spindle_status_subscribed {
        let status = MultiSpindleStatus::from(&multi_result);
        let event = SimulatorEvent::MultiSpindleStatusCompleted { status };
        let _ = broadcaster.send(event);
    }

    if state.multi_spindle_result_subscribed {
        let event = SimulatorEvent::MultiSpindleResultCompleted {
            result: multi_result
        };
        let _ = broadcaster.send(event);
    }

    if batch_completed {
        let batch_event = SimulatorEvent::BatchCompleted { total: batch_counter };
        let _ = broadcaster.send(batch_event);
    }

    println!(
        "Multi-spindle sync {}: {} spindles, overall {} (batch {}/{})",
        sync_id,
        state.multi_spindle_config.spindle_count,
        if multi_result.sync_overall_status { "OK" } else { "NOK" },
        batch_counter,
        state.batch_manager.target_size()
    );

} else {
    // ============================================================
    // SINGLE-SPINDLE MODE (existing logic)
    // ============================================================
    // ... existing MID 0061 logic unchanged ...
}
```

### 8. HTTP Configuration Endpoint

**Add to** `src/http_server.rs`:

```rust
#[derive(Deserialize)]
struct MultiSpindleConfigRequest {
    enabled: bool,
    #[serde(default)]
    spindle_count: Option<u8>,
}

#[derive(Serialize)]
struct MultiSpindleConfigResponse {
    success: bool,
    message: String,
    enabled: bool,
    spindle_count: u8,
    spindles: Vec<SpindleInfo>,
}

async fn configure_multi_spindle(
    AxumState(server_state): AxumState<ServerState>,
    Json(payload): Json<MultiSpindleConfigRequest>,
) -> impl IntoResponse {
    let mut state = server_state.device_state.write().unwrap();

    if payload.enabled {
        let spindle_count = payload.spindle_count.unwrap_or(2);

        if spindle_count < 2 || spindle_count > 10 {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Spindle count must be 2-10"
                })),
            );
        }

        state.enable_multi_spindle(spindle_count);
        let spindles = state.multi_spindle_config.get_spindles();

        (
            StatusCode::OK,
            Json(MultiSpindleConfigResponse {
                success: true,
                message: format!("Multi-spindle mode enabled with {} spindles", spindle_count),
                enabled: true,
                spindle_count,
                spindles,
            }),
        )
    } else {
        state.disable_multi_spindle();

        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Multi-spindle mode disabled (single-spindle mode)"
            })),
        )
    }
}

// Add to Router:
let app = Router::new()
    .route("/state", get(get_state))
    .route("/simulate/tightening", post(simulate_tightening))
    .route("/auto-tightening/start", post(start_auto_tightening))
    .route("/auto-tightening/stop", post(stop_auto_tightening))
    .route("/auto-tightening/status", get(get_auto_tightening_status))
    .route("/config/multi-spindle", post(configure_multi_spindle))  // NEW
    .with_state(server_state);
```

### 9. Main Event Loop Integration

**Modify** `src/main.rs`:

```rust
loop {
    tokio::select! {
        // Handle incoming TCP messages...
        Some(result) = framed.next() => { /* existing logic */ }

        // Handle broadcast events
        Ok(event) = event_rx.recv() => {
            match event {
                // Existing events...
                SimulatorEvent::TighteningCompleted { result } => { /* ... */ }

                // Multi-spindle status (MID 0091)
                SimulatorEvent::MultiSpindleStatusCompleted { status } => {
                    if session.subscriptions().is_subscribed_to_multi_spindle_status() {
                        println!("Broadcasting MID 0091 to subscribed client ({})",
                                 session.addr());
                        let response = protocol::Response::from_data(91, 1, status);
                        let response_bytes = protocol::serializer::serialize_response(&response);

                        if let Err(e) = framed.send(response_bytes.as_slice().into()).await {
                            eprintln!("send error during broadcast: {e}");
                            break;
                        }
                    }
                }

                // Multi-spindle result (MID 0101)
                SimulatorEvent::MultiSpindleResultCompleted { result } => {
                    if session.subscriptions().is_subscribed_to_multi_spindle_result() {
                        println!("Broadcasting MID 0101 to subscribed client ({})",
                                 session.addr());
                        let response = protocol::Response::from_data(101, 4, result);
                        let response_bytes = protocol::serializer::serialize_response(&response);

                        if let Err(e) = framed.send(response_bytes.as_slice().into()).await {
                            eprintln!("send error during broadcast: {e}");
                            break;
                        }
                    }
                }

                // Other events...
            }
        }
    }
}
```

---

## Implementation Plan

### Phase 1: Core Infrastructure (Day 1)
**Goal**: Set up data structures and basic configuration

**Tasks**:
1. Create `src/multi_spindle.rs`
   - MultiSpindleConfig struct
   - SpindleResult, MultiSpindleResult structs
   - MultiSpindleStatus struct
   - generate_multi_spindle_results function

2. Modify `src/state.rs`
   - Add multi_spindle_config field
   - Add enable_multi_spindle/disable_multi_spindle methods

3. Modify `src/events.rs`
   - Add MultiSpindleStatusCompleted event
   - Add MultiSpindleResultCompleted event

4. Modify `src/subscriptions.rs`
   - Add multi_spindle_status subscription flag
   - Add multi_spindle_result subscription flag
   - Add subscription methods

**Deliverable**: Compiles successfully, basic data structures in place

---

### Phase 2: HTTP Configuration (Day 1-2)
**Goal**: Enable/disable multi-spindle via HTTP API

**Tasks**:
1. Add configuration endpoint in `src/http_server.rs`
   - POST /config/multi-spindle
   - Enable/disable logic
   - Spindle count validation (2-10)

2. Test configuration:
   ```bash
   curl -X POST http://localhost:8081/config/multi-spindle \
     -d '{"enabled": true, "spindle_count": 4}'
   ```

**Deliverable**: Can toggle multi-spindle mode, configuration visible in GET /state

---

### Phase 3: MID 0090/0091 Status Flow (Day 2-3)
**Goal**: Subscribe and receive lightweight status messages

**Tasks**:
1. Create `src/handler/multi_spindle_status_subscribe.rs` (MID 0090)
2. Create `src/handler/multi_spindle_status_unsubscribe.rs` (MID 0093)
3. Create `src/handler/data/multi_spindle_status.rs` (MID 0091)
   - Implement Serializable trait
4. Register handlers in `src/handler/mod.rs`
5. Update session subscription tracking
6. Integrate into auto-tightening loop (status broadcast)
7. Add event handling in `src/main.rs`

**Test**:
```bash
# Subscribe to status
echo '00200090001         ' | nc localhost 8080

# Start auto-tightening (multi-spindle mode)
curl -X POST http://localhost:8081/auto-tightening/start

# Expect: Receive MID 0091 messages
```

**Deliverable**: MID 0091 broadcasting works

---

### Phase 4: MID 0100/0101 Result Flow (Day 3-4)
**Goal**: Subscribe and receive full result messages with torque/angle

**Tasks**:
1. Create `src/handler/multi_spindle_result_subscribe.rs` (MID 0100)
2. Create `src/handler/multi_spindle_result_unsubscribe.rs` (MID 0103)
3. Create `src/handler/data/multi_spindle_result.rs` (MID 0101)
   - Implement Serializable trait (complex 18-byte per spindle)
4. Register handlers
5. Integrate into auto-tightening loop (result broadcast)
6. Add event handling in `src/main.rs`

**Test**:
```bash
# Subscribe to result
echo '00200100001         ' | nc localhost 8080

# Start auto-tightening
curl -X POST http://localhost:8081/auto-tightening/start

# Expect: Receive MID 0101 messages with full details
```

**Deliverable**: MID 0101 broadcasting works

---

### Phase 5: Batch Integration & Testing (Day 4-5)
**Goal**: Verify batch counter behavior with multi-spindle

**Tasks**:
1. Test batch counter increments only when sync_overall_status = OK
2. Test retry workflow (NOK → enable tool → retry)
3. Test multiple sequential batches
4. Test failure scenarios (1 spindle fails, all fail, none fail)
5. Verify ±5% variation per spindle
6. Verify independent per-spindle failures

**Test Scenarios**:
```bash
# Scenario 1: All spindles OK
curl -X POST http://localhost:8081/config/multi-spindle \
  -d '{"enabled": true, "spindle_count": 3}'

curl -X POST http://localhost:8081/auto-tightening/start \
  -d '{"interval_ms": 2000, "failure_rate": 0.0}'

# Expect: All 3 spindles OK, batch counter increments

# Scenario 2: High failure rate
curl -X POST http://localhost:8081/auto-tightening/start \
  -d '{"interval_ms": 2000, "failure_rate": 0.5}'

# Expect: Some spindles NOK, batch counter doesn't increment
```

**Deliverable**: All batch scenarios pass

---

### Phase 6: Unit & Integration Tests (Day 5)
**Goal**: Comprehensive test coverage

**Tasks**:
1. Unit tests for multi_spindle.rs
   - MultiSpindleConfig::new validation (2-10)
   - get_spindles auto-generation
   - next_sync_id wrapping at 65535
   - generate_multi_spindle_results

2. Unit tests for serialization
   - MID 0091 field layout
   - MID 0101 field layout
   - Edge cases (1 spindle fail, all fail, none fail)

3. Integration tests
   - Subscribe → receive broadcasts
   - Batch counter behavior
   - Mode switching (single → multi → single)

**Deliverable**: All tests pass, coverage > 80%

---

### Phase 7: Documentation & Examples (Day 6)
**Goal**: Update README and create usage examples

**Tasks**:
1. Update README.md
   - Add multi-spindle section
   - Configuration examples
   - Testing workflows

2. Create example scripts
   - multi-spindle-test.sh
   - multi-spindle-batch-workflow.sh

3. Update architecture diagrams

**Deliverable**: Complete documentation

---

## Test Scenarios

### Test 1: Basic Multi-Spindle Configuration
**Objective**: Verify multi-spindle mode can be enabled and configured

**Steps**:
1. Start simulator
2. Enable multi-spindle with 4 spindles:
   ```bash
   curl -X POST http://localhost:8081/config/multi-spindle \
     -d '{"enabled": true, "spindle_count": 4}'
   ```
3. Query state:
   ```bash
   curl http://localhost:8081/state
   ```

**Expected**:
```json
{
  "multi_spindle_config": {
    "enabled": true,
    "spindle_count": 4,
    "sync_tightening_id": 0
  }
}
```

---

### Test 2: Multi-Spindle Status Subscription (MID 0091)
**Objective**: Verify status subscription and broadcast

**Steps**:
1. Enable multi-spindle (4 spindles)
2. Integrator connects:
   ```bash
   echo '00200090001         ' | nc localhost 8080
   ```
3. Expect MID 0005 acknowledgement
4. Start auto-tightening
5. Observe MID 0091 messages

**Expected MID 0091 format** (4 spindles, all OK):
```
008600910010204000012032025-10-13:15:00:00041050120102041050220104105032010410504201041
```

Breakdown:
- `0086` = Length
- `0091` = MID
- `001` = Revision
- `02` = Param 01
- `04` = 4 spindles
- `00001` = Param 02 (sync ID)
- `2025-10-13:15:00:00` = Param 03 (timestamp)
- `04` = Param 04
- `1` = Sync overall OK
- `05` = Param 05
- `01` `01` `1` = Spindle 1, channel 1, OK
- `02` `02` `1` = Spindle 2, channel 2, OK
- `03` `03` `1` = Spindle 3, channel 3, OK
- `04` `04` `1` = Spindle 4, channel 4, OK

---

### Test 3: Multi-Spindle Result Subscription (MID 0101)
**Objective**: Verify full result subscription with torque/angle details

**Steps**:
1. Enable multi-spindle (2 spindles)
2. Integrator subscribes:
   ```bash
   echo '00200100001         ' | nc localhost 8080
   ```
3. Start auto-tightening
4. Observe MID 0101 messages

**Expected**: MID 0101 with 18 bytes per spindle in parameter 18

---

### Test 4: All Spindles OK - Batch Increments
**Objective**: Verify batch counter increments when all spindles succeed

**Setup**:
- Multi-spindle: 3 spindles
- Batch size: 4
- Failure rate: 0.0 (all OK)

**Steps**:
1. Configure and start auto-tightening
2. Monitor batch counter

**Expected**:
- Sync 1: counter 0→1
- Sync 2: counter 1→2
- Sync 3: counter 2→3
- Sync 4: counter 3→4, batch complete

---

### Test 5: One Spindle NOK - Batch Doesn't Increment
**Objective**: Verify batch counter stays same when any spindle fails

**Setup**:
- Multi-spindle: 4 spindles
- Batch size: 3
- Failure rate: 0.5 (high)

**Steps**:
1. Start auto-tightening
2. Wait for a NOK sync operation (at least one spindle fails)
3. Verify batch counter doesn't change

**Expected**:
- Sync 1 (all OK): counter 0→1
- Sync 2 (1 spindle NOK): counter stays 1
- Sync 3 (retry, all OK): counter 1→2

---

### Test 6: Independent Spindle Failures
**Objective**: Verify each spindle can fail independently

**Setup**:
- Multi-spindle: 5 spindles
- Failure rate: 0.3

**Steps**:
1. Run 10 sync operations
2. Parse MID 0091 messages
3. Count failures per spindle

**Expected**: Each spindle has different failure counts (not correlated)

---

### Test 7: Torque/Angle Variation (±5%)
**Objective**: Verify each spindle has independent variation

**Setup**:
- Multi-spindle: 4 spindles
- Base torque: 12.5 Nm
- Base angle: 40°

**Steps**:
1. Run single sync operation
2. Parse MID 0101 message
3. Extract torque/angle per spindle

**Expected**:
- Spindle 1: 11.9-13.1 Nm, 38-42°
- Spindle 2: 11.9-13.1 Nm, 38-42° (different from spindle 1)
- Spindle 3: 11.9-13.1 Nm, 38-42° (different from others)
- Spindle 4: 11.9-13.1 Nm, 38-42° (different from others)

---

### Test 8: Sync ID Increments
**Objective**: Verify sync_tightening_id increments after each operation

**Steps**:
1. Run 5 sync operations
2. Parse sync IDs from MID 0091

**Expected**:
- Sync 1: ID = 1
- Sync 2: ID = 2
- Sync 3: ID = 3
- Sync 4: ID = 4
- Sync 5: ID = 5

---

### Test 9: Sync ID Wraps at 65535
**Objective**: Verify sync ID wraps correctly

**Setup**: Set sync_tightening_id to 65534 manually

**Steps**:
1. Run 3 sync operations
2. Verify IDs

**Expected**:
- Sync 1: ID = 65535
- Sync 2: ID = 0 (wrapped)
- Sync 3: ID = 1

---

### Test 10: Mode Switching (Single ↔ Multi)
**Objective**: Verify safe switching between modes

**Steps**:
1. Start in single-spindle, run tightening
2. Enable multi-spindle (4 spindles)
3. Run tightening (expect MID 0091)
4. Disable multi-spindle
5. Run tightening (expect MID 0061)

**Expected**: No crashes, correct messages per mode

---

### Test 11: Configuration Validation
**Objective**: Verify spindle count validation

**Test Cases**:
```bash
# Invalid: 0 spindles
curl -X POST .../config/multi-spindle -d '{"enabled": true, "spindle_count": 0}'
# Expected: 400 Bad Request

# Invalid: 1 spindle
curl -X POST .../config/multi-spindle -d '{"enabled": true, "spindle_count": 1}'
# Expected: 400 Bad Request

# Valid: 2 spindles
curl -X POST .../config/multi-spindle -d '{"enabled": true, "spindle_count": 2}'
# Expected: 200 OK

# Valid: 10 spindles
curl -X POST .../config/multi-spindle -d '{"enabled": true, "spindle_count": 10}'
# Expected: 200 OK

# Invalid: 11 spindles
curl -X POST .../config/multi-spindle -d '{"enabled": true, "spindle_count": 11}'
# Expected: 400 Bad Request
```

---

### Test 12: Subscription Isolation
**Objective**: Verify subscriptions are per-client

**Steps**:
1. Client A connects, subscribes to MID 0090 (status)
2. Client B connects, subscribes to MID 0100 (result)
3. Trigger sync tightening

**Expected**:
- Client A receives MID 0091 only
- Client B receives MID 0101 only

---

### Test 13: Backward Compatibility
**Objective**: Verify single-spindle mode still works

**Steps**:
1. Do NOT enable multi-spindle
2. Subscribe to MID 0060 (single-spindle result)
3. Trigger tightening

**Expected**: Receive MID 0061 (not MID 0091 or 0101)

---

### Test 14: Multiple Batches Sequential
**Objective**: Verify multi-spindle works through multiple batches

**Steps**:
1. Enable multi-spindle (3 spindles)
2. Start auto-tightening
3. Integrator sends batch 1 (size 4)
4. Wait for completion
5. Integrator sends batch 2 (size 6)
6. Wait for completion

**Expected**:
- Batch 1 completes after 4 sync OK operations
- Batch 2 starts, completes after 6 sync OK operations

---

### Test 15: Error Handling - Not Multi-Spindle Mode
**Objective**: Verify error when subscribing to multi-spindle in single-spindle mode

**Steps**:
1. Do NOT enable multi-spindle
2. Send MID 0090 (status subscribe)

**Expected**: MID 0004 error "Controller is not in multi-spindle mode"

---

## Open Protocol Specification Reference

### MID 0090 - Multi-spindle Status Subscribe
**Direction**: Integrator → Controller
**Response**: MID 0005 (accepted) or MID 0004 (error)
**Errors**:
- Controller is not a sync master
- Multi-spindle status subscription already exists

**Message Format**:
```
00200090001         \0
```

---

### MID 0091 - Multi-spindle Status
**Direction**: Controller → Integrator
**Response**: MID 0092 (acknowledge)

**Parameters**:
1. Number of spindles (2 bytes: 02-10)
2. Sync tightening ID (5 bytes: 00000-65535)
3. Timestamp (19 bytes: YYYY-MM-DD:HH:MM:SS)
4. Sync overall status (1 byte: 0=NOK, 1=OK)
5. Spindle status (5 bytes per spindle)
   - Bytes 1-2: Spindle number (01-99)
   - Bytes 3-4: Channel ID (01-20)
   - Byte 5: Status (0=NOK, 1=OK)

**Example** (2 spindles, both OK):
```
00670091001020200012032025-10-13:14:30:15041050120102041
```

---

### MID 0092 - Multi-spindle Status Acknowledge
**Direction**: Integrator → Controller
**Response**: None

**Message Format**:
```
00200092001         \0
```

---

### MID 0093 - Multi-spindle Status Unsubscribe
**Direction**: Integrator → Controller
**Response**: MID 0005 (accepted) or MID 0004 (subscription doesn't exist)

**Message Format**:
```
00200093001         \0
```

---

### MID 0100 - Multi-spindle Result Subscribe
**Direction**: Integrator → Controller
**Response**: MID 0005 (accepted) or MID 0004 (error)

**Revision 1**: No data
**Revision 2**: Data No. System (10 bytes) - rewind point
**Revision 3/4/5**: Send only new data flag (1 byte: 0=FALSE, 1=TRUE)

**Message Format** (Revision 1):
```
00200100001         \0
```

---

### MID 0101 - Multi-spindle Result
**Direction**: Controller → Integrator
**Response**: MID 0102 (acknowledge)

**Parameters** (Revision 4):
1. Number of spindles (2 bytes: 01-50)
2. VIN Number (25 bytes)
3. Job ID (2 bytes: 00-99)
4. Parameter Set ID (3 bytes: 000-999)
5. Batch Size (4 bytes: 0000-9999)
6. Batch Counter (4 bytes: 0000-9999)
7. Batch Status (1 byte: 0=NOK, 1=OK, 2=not used)
8. Torque Min (6 bytes: value × 100)
9. Torque Max (6 bytes: value × 100)
10. Torque Target (6 bytes: value × 100)
11. Angle Min (5 bytes: degrees)
12. Angle Max (5 bytes: degrees)
13. Angle Target (5 bytes: degrees)
14. Last Pset Change (19 bytes: YYYY-MM-DD:HH:MM:SS)
15. Timestamp (19 bytes: YYYY-MM-DD:HH:MM:SS)
16. Sync Tightening ID (5 bytes: 00000-65535)
17. Sync Overall Status (1 byte: 0=NOK, 1=OK)
18. Spindle Results (18 bytes per spindle)
    - Bytes 1-2: Spindle number
    - Bytes 3-4: Channel ID
    - Byte 5: Overall status
    - Byte 6: Torque status
    - Bytes 7-12: Torque value (× 100)
    - Byte 13: Angle status
    - Bytes 14-18: Angle value
19. System Subtype (3 bytes: 001=tightening, 002=press)

---

### MID 0102 - Multi-spindle Result Acknowledge
**Direction**: Integrator → Controller
**Response**: None

**Message Format**:
```
00200102001         \0
```

---

### MID 0103 - Multi-spindle Result Unsubscribe
**Direction**: Integrator → Controller
**Response**: MID 0005 (accepted) or MID 0004 (subscription doesn't exist)

**Message Format**:
```
00200103001         \0
```

---

## Summary

This document provides a complete specification for implementing multi-spindle mode in the Open Protocol Device Simulator. The design:

✅ **Extends existing architecture** with minimal disruption
✅ **Maintains backward compatibility** (single-spindle remains default)
✅ **Follows Open Protocol spec** for MIDs 0090-0103
✅ **Supports realistic scenarios** (±5% variation, independent failures)
✅ **Integrates with batch management** (sync_overall_status controls counter)
✅ **Provides HTTP configuration** (easy mode switching)
✅ **Includes comprehensive tests** (15+ test scenarios)

**Implementation Estimate**: 5-6 days for complete feature with tests and documentation.

**Next Steps**: Review this document, approve the design, and begin Phase 1 implementation.
