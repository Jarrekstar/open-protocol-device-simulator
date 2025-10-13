# Multi-Device Simulation Architecture (Shelved)

**Status**: Shelved for future consideration
**Date**: 2025-10-13
**Context**: Discussion about whether multiple TCP clients should share one device or represent separate devices

---

## Current Architecture: Single Shared Device

### Design
All TCP clients connecting to port 8080 share the **same simulated device state**:
- Shared batch counter
- Shared parameter set (pset)
- Shared vehicle ID
- Shared tool enabled/disabled state
- Independent subscriptions per client

### Behavior Example
```
Client A connects → sets batch size to 5
Client B connects → sees same batch (counter starts from current position)
HTTP triggers tightening → both clients receive MID 0061 (if subscribed)
```

### Why This Design?
**Matches real hardware behavior**: A physical Atlas Copco controller allows multiple TCP connections, but all connections interact with the same physical device. Common scenario:
- Primary integrator (MES system)
- Secondary monitoring dashboard
- Data logger
- Diagnostic tool

All see the same device state because there's only ONE physical controller.

---

## Use Cases Analysis

### ✅ When Current Design Makes Sense

#### 1. Single Integrator Development
- Developer building one integrator application
- Testing without physical hardware
- May have multiple connections from different components (UI + backend)

#### 2. Multi-Client Coordination Testing
- Testing that MES and monitoring dashboard don't conflict
- Verifying subscription isolation (each client gets only what they subscribed to)
- Testing command collision scenarios

#### 3. Load Testing
- Simulating 50 clients connected to ONE device
- Testing broadcaster performance
- Realistic production scenario

---

### ❌ When Current Design Has Limitations

#### 4. Multi-Device Assembly Line
```
Real factory layout:
Station 1 → Controller A (device_id=1)
Station 2 → Controller B (device_id=2)
Station 3 → Controller C (device_id=3)
```
**Problem**: Current simulator represents only ONE device. Cannot simulate multiple independent stations in one process.

#### 5. Isolated Testing (Multiple Developers)
```
Developer A: Testing batch workflow (batch size = 10)
Developer B: Testing pset selection (needs clean state)
Developer C: Load testing with auto-tightening
```
**Problem**: All developers interfere with each other. State changes affect all connections.

#### 6. Parallel Integration Tests (CI/CD)
```
Test Suite A: Batch completion scenarios
Test Suite B: Tool enable/disable logic
Test Suite C: Retry workflows
```
**Problem**: Test suites running concurrently corrupt each other's state.

---

## Potential Solutions

### Option 1: Multiple Process Instances ⭐ Simplest

Run separate simulator processes for each device:

```bash
# Station 1
cargo run --release &  # Ports 8080/8081

# Station 2
PORT=8082 HTTP_PORT=8083 cargo run --release &

# Station 3
PORT=8084 HTTP_PORT=8085 cargo run --release &
```

**Docker Compose Example:**
```yaml
services:
  station1:
    build: .
    ports: ["8080:8080", "8081:8081"]
    environment:
      - DEVICE_ID=1

  station2:
    build: .
    ports: ["8082:8080", "8083:8081"]
    environment:
      - DEVICE_ID=2

  station3:
    build: .
    ports: ["8084:8080", "8085:8081"]
    environment:
      - DEVICE_ID=3
```

**Pros:**
- No code changes required
- Each process is truly isolated
- Realistic: separate processes = separate physical devices
- Simple to understand and maintain

**Cons:**
- Resource overhead (3 processes = 3x memory)
- Manual port management
- More complex orchestration

---

### Option 2: Multi-Device Support with Device ID Routing

Modify simulator to support multiple virtual devices in one process:

```rust
// New architecture
struct SimulatorState {
    devices: HashMap<u32, Arc<RwLock<DeviceState>>>,  // device_id → state
}

// Routing logic
async fn handle_connection(stream: TcpStream, state: Arc<RwLock<SimulatorState>>) {
    // Extract device_id from MID 0001 handshake or station_id field
    let device_id = extract_device_id_from_handshake(&first_message);

    // Route all subsequent messages to that device's state
    let device_state = state.read().unwrap()
        .devices.get(&device_id)
        .cloned()
        .expect("Invalid device_id");

    // Continue with per-device state...
}
```

**Protocol Changes:**
- Use `station_id` field (2 digits) in Open Protocol header as `device_id`
- Or extend MID 0001 to include device selection
- Each device has independent: batch counter, pset, vehicle ID, etc.

**HTTP API Changes:**
```bash
# Target specific device
curl http://localhost:8081/device/1/state
curl http://localhost:8081/device/2/simulate/tightening

# Or use query parameter
curl http://localhost:8081/state?device_id=2
```

**Pros:**
- Simulate entire assembly line in one process
- Lower resource overhead than multiple processes
- Realistic: each device has independent state
- Single HTTP API endpoint

**Cons:**
- Significant code changes required
- More complex state management
- Need to define device_id selection protocol
- Potential confusion if device_id not specified

**Implementation Complexity:** Medium-High

**Files to Modify:**
- `src/main.rs` - Add device routing logic
- `src/state.rs` - Wrap devices in HashMap
- `src/http_server.rs` - Add device_id parameter to all endpoints
- `src/protocol/parser.rs` - Extract device_id from messages
- `src/handler/*` - Pass device_id through handler chain

---

### Option 3: Isolated Mode Configuration Flag

Add a configuration flag to change behavior:

```rust
enum SimulatorMode {
    Shared,    // Current: all clients share one device (default)
    Isolated,  // Each TCP connection gets own DeviceState
}

// In main.rs
let mode = std::env::var("SIMULATOR_MODE")
    .unwrap_or("shared".to_string());

match mode.as_str() {
    "isolated" => {
        // Create new DeviceState per connection
        tokio::spawn(async move {
            let device_state = DeviceState::new_shared();  // Per-connection
            // ...
        });
    }
    "shared" => {
        // Current behavior
    }
}
```

**Usage:**
```bash
# For parallel testing
SIMULATOR_MODE=isolated cargo test

# For integration testing (default)
cargo run
```

**Pros:**
- Solves test isolation problem
- Relatively simple implementation
- Backward compatible (default = current behavior)

**Cons:**
- **NOT realistic** - doesn't match real hardware behavior
- Confusing: clients think they're connected to same device but aren't
- No shared state for legitimate multi-client scenarios
- HTTP API ambiguity: which device state does `/state` return?

**Implementation Complexity:** Low-Medium

---

## Industrial Automation Context

In real factory environments:
- Each **workstation** typically has its own controller
- Integrator (MES/SCADA) connects to **multiple controllers** (one per station)
- Each controller maintains **independent state**
- Each controller allows **multiple TCP connections** to its single device

**Key Question**: "How many physical controllers are you replacing?"
- **One controller** → Current design is perfect ✅
- **Multiple controllers** → Consider Option 1 (multiple processes) or Option 2 (multi-device routing)

---

## Recommendation

**Keep current design as default** - it correctly simulates one physical device with multiple client connections.

### For Multi-Device Scenarios:
1. **Short-term**: Use Option 1 (multiple processes on different ports)
2. **Long-term** (if needed frequently): Implement Option 2 (multi-device routing)

### For Test Isolation:
- Use Docker containers (one simulator per container)
- Use separate machines/VMs
- Run tests serially instead of parallel

---

## Decision: Shelved

**Reason**: Current single-device design is correct for primary use case (testing one integrator against one device). Multi-device support adds complexity for uncertain benefit.

**Revisit If**:
- Users frequently request multi-station simulation
- Assembly line simulation becomes common requirement
- Test isolation becomes critical pain point

**Alternative**: Document the single-device behavior clearly in README and provide Docker Compose examples for multi-device scenarios.

---

## README Addition (If Implemented)

```markdown
## Multi-Device Simulation

This simulator represents **one physical controller device**. All TCP clients connected to port 8080 share the same device state (batch counter, pset, vehicle ID, etc.). This matches real hardware behavior where multiple systems connect to a single controller.

### Simulating Multiple Devices

To simulate multiple controllers (e.g., multi-station assembly line), run multiple instances:

**Using Multiple Processes:**
```bash
cargo run --release &  # Device 1: ports 8080/8081
PORT=8082 HTTP_PORT=8083 cargo run --release &  # Device 2
PORT=8084 HTTP_PORT=8085 cargo run --release &  # Device 3
```

**Using Docker Compose:**
```yaml
services:
  station1:
    build: .
    ports: ["8080:8080", "8081:8081"]
  station2:
    build: .
    ports: ["8082:8080", "8083:8081"]
  station3:
    build: .
    ports: ["8084:8080", "8085:8081"]
```

Each instance is an independent device with its own state.
```

---

## Related Discussions

- Connection session FSM (per-client): Already implemented ✅
- Device operational FSM (global): Already implemented ✅
- Batch manager (shared): Already implemented ✅
- Subscription isolation (per-client): Already implemented ✅

The architecture already has good separation of concerns. Multi-device support would primarily affect the `main.rs` connection handling and state initialization.
