use crate::subscriptions::Subscriptions;
use std::net::SocketAddr;
use std::time::Instant;

/// Initial state - no connection established
pub struct Disconnected;

/// Connected state - TCP connection established, awaiting authentication
pub struct Connected {
    /// Remote client address
    pub addr: SocketAddr,
    /// When the connection was established
    #[allow(dead_code)]
    pub connected_at: Instant,
}

/// Ready state - authenticated and ready for normal operations
pub struct Ready {
    /// Remote client address
    pub addr: SocketAddr,
    /// When the connection was established (for connection duration tracking)
    #[allow(dead_code)]
    pub connected_at: Instant,
    /// When we last received a message (for keep-alive tracking)
    pub last_activity: Instant,
    /// Active subscriptions for this connection
    pub subscriptions: Subscriptions,
}

// ============================================================================
// Connection Session (generic over state)
// ============================================================================

/// Represents a connection session in a specific state
///
/// The generic parameter S ensures only valid state transitions are possible
pub struct ConnectionSession<S> {
    state: S,
}

// ============================================================================
// State: Disconnected
// ============================================================================

impl ConnectionSession<Disconnected> {
    /// Create a new disconnected session
    pub fn new() -> Self {
        Self {
            state: Disconnected,
        }
    }

    /// Transition to Connected state when TCP connection is established
    pub fn connect(self, addr: SocketAddr) -> ConnectionSession<Connected> {
        ConnectionSession {
            state: Connected {
                addr,
                connected_at: Instant::now(),
            },
        }
    }
}

impl Default for ConnectionSession<Disconnected> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// State: Connected
// ============================================================================

impl ConnectionSession<Connected> {
    /// Get the client's socket address
    #[allow(dead_code)]
    pub fn addr(&self) -> SocketAddr {
        self.state.addr
    }

    /// Get connection timestamp
    #[allow(dead_code)]
    pub fn connected_at(&self) -> Instant {
        self.state.connected_at
    }

    /// Transition to Ready state after successful authentication (MID 0001/0002)
    pub fn authenticate(self) -> ConnectionSession<Ready> {
        ConnectionSession {
            state: Ready {
                addr: self.state.addr,
                connected_at: self.state.connected_at,
                last_activity: Instant::now(),
                subscriptions: Subscriptions::new(),
            },
        }
    }

    /// Disconnect and return to initial state
    #[allow(dead_code)]
    pub fn disconnect(self) -> ConnectionSession<Disconnected> {
        ConnectionSession::new()
    }
}

// ============================================================================
// State: Ready
// ============================================================================

impl ConnectionSession<Ready> {
    /// Get the client's socket address
    pub fn addr(&self) -> SocketAddr {
        self.state.addr
    }

    /// Get connection timestamp
    #[allow(dead_code)]
    pub fn connected_at(&self) -> Instant {
        self.state.connected_at
    }

    /// Get last activity timestamp
    #[allow(dead_code)]
    pub fn last_activity(&self) -> Instant {
        self.state.last_activity
    }

    /// Update last activity timestamp (call on every received message)
    pub fn update_keep_alive(&mut self) {
        self.state.last_activity = Instant::now();
    }

    /// Check if connection has timed out (Open Protocol: 15 second idle timeout)
    #[allow(dead_code)]
    pub fn is_timed_out(&self, timeout_secs: u64) -> bool {
        self.state.last_activity.elapsed().as_secs() >= timeout_secs
    }

    /// Get mutable reference to subscriptions
    #[allow(dead_code)]
    pub fn subscriptions_mut(&mut self) -> &mut Subscriptions {
        &mut self.state.subscriptions
    }

    /// Get immutable reference to subscriptions
    pub fn subscriptions(&self) -> &Subscriptions {
        &self.state.subscriptions
    }

    /// Subscribe to tightening result events (MID 60)
    pub fn subscribe_tightening_result(&mut self) {
        self.state.subscriptions.subscribe_tightening_result();
    }

    /// Unsubscribe from tightening result events (MID 63)
    pub fn unsubscribe_tightening_result(&mut self) {
        self.state.subscriptions.unsubscribe_tightening_result();
    }

    /// Subscribe to parameter set selection events (MID 14)
    pub fn subscribe_pset_selection(&mut self) {
        self.state.subscriptions.subscribe_pset_selection();
    }

    /// Unsubscribe from parameter set selection events (MID 16)
    pub fn unsubscribe_pset_selection(&mut self) {
        self.state.subscriptions.unsubscribe_pset_selection();
    }

    /// Subscribe to vehicle ID events (MID 51)
    pub fn subscribe_vehicle_id(&mut self) {
        self.state.subscriptions.subscribe_vehicle_id();
    }

    /// Unsubscribe from vehicle ID events (MID 54)
    pub fn unsubscribe_vehicle_id(&mut self) {
        self.state.subscriptions.unsubscribe_vehicle_id();
    }

    /// Subscribe to multi-spindle status events (MID 90)
    pub fn subscribe_multi_spindle_status(&mut self) {
        self.state.subscriptions.subscribe_multi_spindle_status();
    }

    /// Unsubscribe from multi-spindle status events (MID 92)
    pub fn unsubscribe_multi_spindle_status(&mut self) {
        self.state.subscriptions.unsubscribe_multi_spindle_status();
    }

    /// Subscribe to multi-spindle result events (MID 100)
    pub fn subscribe_multi_spindle_result(&mut self) {
        self.state.subscriptions.subscribe_multi_spindle_result();
    }

    /// Unsubscribe from multi-spindle result events (MID 102)
    pub fn unsubscribe_multi_spindle_result(&mut self) {
        self.state.subscriptions.unsubscribe_multi_spindle_result();
    }

    /// Disconnect and return to initial state
    #[allow(dead_code)]
    pub fn disconnect(self) -> ConnectionSession<Disconnected> {
        ConnectionSession::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use std::thread;
    use std::time::Duration;

    fn test_addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
    }

    #[test]
    fn test_state_transition_disconnected_to_connected() {
        let session = ConnectionSession::new();
        let addr = test_addr();

        let session = session.connect(addr);
        assert_eq!(session.addr(), addr);
    }

    #[test]
    fn test_state_transition_connected_to_ready() {
        let session = ConnectionSession::new();
        let session = session.connect(test_addr());
        let session = session.authenticate();

        assert!(!session.subscriptions().is_subscribed_to_tightening_result());
        assert_eq!(session.subscriptions().active_count(), 0);
    }

    #[test]
    fn test_full_connection_lifecycle() {
        let session = ConnectionSession::new();
        let session = session.connect(test_addr());
        let session = session.authenticate();
        let _session = session.disconnect();

        // Successfully returned to disconnected state
    }

    #[test]
    fn test_keep_alive_tracking() {
        let session = ConnectionSession::new();
        let session = session.connect(test_addr());
        let mut session = session.authenticate();

        thread::sleep(Duration::from_millis(100));

        // Should not be timed out yet (100ms < 15s)
        assert!(!session.is_timed_out(15));

        // Update keep alive
        session.update_keep_alive();

        // Should still not be timed out
        assert!(!session.is_timed_out(15));
    }

    #[test]
    fn test_subscription_management() {
        let session = ConnectionSession::new();
        let session = session.connect(test_addr());
        let mut session = session.authenticate();

        // Initially no subscriptions
        assert!(!session.subscriptions().is_subscribed_to_tightening_result());
        assert!(!session.subscriptions().is_subscribed_to_pset_selection());

        // Subscribe to tightening results
        session.subscribe_tightening_result();
        assert!(session.subscriptions().is_subscribed_to_tightening_result());
        assert_eq!(session.subscriptions().active_count(), 1);

        // Subscribe to pset selection
        session.subscribe_pset_selection();
        assert!(session.subscriptions().is_subscribed_to_pset_selection());
        assert_eq!(session.subscriptions().active_count(), 2);

        // Unsubscribe from tightening results
        session.unsubscribe_tightening_result();
        assert!(!session.subscriptions().is_subscribed_to_tightening_result());
        assert_eq!(session.subscriptions().active_count(), 1);
    }

    #[test]
    fn test_timeout_detection() {
        let session = ConnectionSession::new();
        let session = session.connect(test_addr());
        let mut session = session.authenticate();

        // Artificially set last activity to past
        session.state.last_activity = Instant::now() - Duration::from_secs(20);

        // Should be timed out with 15 second timeout
        assert!(session.is_timed_out(15));

        // Update keep alive
        session.update_keep_alive();

        // Should no longer be timed out
        assert!(!session.is_timed_out(15));
    }

    #[test]
    fn test_disconnect_from_connected() {
        let session = ConnectionSession::new();
        let session = session.connect(test_addr());
        let _session = session.disconnect();

        // Successfully returned to disconnected state
    }

    #[test]
    fn test_disconnect_from_ready() {
        let session = ConnectionSession::new();
        let session = session.connect(test_addr());
        let session = session.authenticate();
        let _session = session.disconnect();

        // Successfully returned to disconnected state
    }

    #[test]
    fn test_complete_session_lifecycle_with_subscriptions() {
        // Phase 1: Disconnected
        let session = ConnectionSession::new();

        // Phase 2: Connect
        let addr = test_addr();
        let session = session.connect(addr);
        assert_eq!(session.addr(), addr);

        // Phase 3: Authenticate and transition to Ready
        let mut session = session.authenticate();

        // Phase 4: Manage subscriptions in Ready state
        assert_eq!(session.subscriptions().active_count(), 0);

        session.subscribe_tightening_result();
        assert!(session.subscriptions().is_subscribed_to_tightening_result());

        session.subscribe_pset_selection();
        assert!(session.subscriptions().is_subscribed_to_pset_selection());
        assert_eq!(session.subscriptions().active_count(), 2);

        // Phase 5: Keep-alive management
        session.update_keep_alive();
        assert!(!session.is_timed_out(15));

        // Phase 6: Unsubscribe
        session.unsubscribe_tightening_result();
        assert!(!session.subscriptions().is_subscribed_to_tightening_result());
        assert_eq!(session.subscriptions().active_count(), 1);

        // Phase 7: Disconnect
        let _session = session.disconnect();

        // Successfully completed full lifecycle
    }

    #[test]
    fn test_session_address_tracking() {
        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 12345);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 101)), 12346);

        let session1 = ConnectionSession::new().connect(addr1);
        let session2 = ConnectionSession::new().connect(addr2);

        assert_eq!(session1.addr(), addr1);
        assert_eq!(session2.addr(), addr2);
        assert_ne!(session1.addr(), session2.addr());
    }

    #[test]
    fn test_subscription_isolation() {
        // Create two independent sessions
        let mut session1 = ConnectionSession::new().connect(test_addr()).authenticate();

        let session2 = ConnectionSession::new().connect(test_addr()).authenticate();

        // Subscribe session1 but not session2
        session1.subscribe_tightening_result();

        // Verify isolation
        assert!(
            session1
                .subscriptions()
                .is_subscribed_to_tightening_result()
        );
        assert!(
            !session2
                .subscriptions()
                .is_subscribed_to_tightening_result()
        );
    }
}
