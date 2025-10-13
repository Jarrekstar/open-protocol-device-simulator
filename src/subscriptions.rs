use serde::Serialize;

/// Manages client subscription state for various event types
#[derive(Debug, Clone, Default, Serialize)]
pub struct Subscriptions {
    /// Subscribed to tightening result events (MID 0061)
    pub tightening_result: bool,

    /// Subscribed to parameter set selection events (MID 0015)
    pub pset_selection: bool,

    /// Subscribed to alarm events (future support)
    pub alarm: bool,

    /// Subscribed to job info events (future support)
    pub job_info: bool,
}

impl Subscriptions {
    /// Create a new subscription manager with all subscriptions disabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to tightening result events
    pub fn subscribe_tightening_result(&mut self) {
        self.tightening_result = true;
    }

    /// Unsubscribe from tightening result events
    pub fn unsubscribe_tightening_result(&mut self) {
        self.tightening_result = false;
    }

    /// Subscribe to parameter set selection events
    pub fn subscribe_pset_selection(&mut self) {
        self.pset_selection = true;
    }

    /// Unsubscribe from parameter set selection events
    pub fn unsubscribe_pset_selection(&mut self) {
        self.pset_selection = false;
    }

    /// Check if subscribed to tightening results
    pub fn is_subscribed_to_tightening_result(&self) -> bool {
        self.tightening_result
    }

    /// Check if subscribed to pset selection
    pub fn is_subscribed_to_pset_selection(&self) -> bool {
        self.pset_selection
    }

    /// Get count of active subscriptions
    pub fn active_count(&self) -> usize {
        let mut count = 0;
        if self.tightening_result { count += 1; }
        if self.pset_selection { count += 1; }
        if self.alarm { count += 1; }
        if self.job_info { count += 1; }
        count
    }

    /// Check if any subscriptions are active
    pub fn has_any_subscription(&self) -> bool {
        self.active_count() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_no_subscriptions() {
        let subs = Subscriptions::new();
        assert!(!subs.is_subscribed_to_tightening_result());
        assert!(!subs.is_subscribed_to_pset_selection());
        assert_eq!(subs.active_count(), 0);
        assert!(!subs.has_any_subscription());
    }

    #[test]
    fn test_subscribe_tightening_result() {
        let mut subs = Subscriptions::new();
        subs.subscribe_tightening_result();

        assert!(subs.is_subscribed_to_tightening_result());
        assert_eq!(subs.active_count(), 1);
        assert!(subs.has_any_subscription());
    }

    #[test]
    fn test_unsubscribe_tightening_result() {
        let mut subs = Subscriptions::new();
        subs.subscribe_tightening_result();
        subs.unsubscribe_tightening_result();

        assert!(!subs.is_subscribed_to_tightening_result());
        assert_eq!(subs.active_count(), 0);
    }

    #[test]
    fn test_multiple_subscriptions() {
        let mut subs = Subscriptions::new();
        subs.subscribe_tightening_result();
        subs.subscribe_pset_selection();

        assert!(subs.is_subscribed_to_tightening_result());
        assert!(subs.is_subscribed_to_pset_selection());
        assert_eq!(subs.active_count(), 2);
    }

    #[test]
    fn test_subscribe_idempotent() {
        let mut subs = Subscriptions::new();
        subs.subscribe_tightening_result();
        subs.subscribe_tightening_result();

        assert!(subs.is_subscribed_to_tightening_result());
        assert_eq!(subs.active_count(), 1);
    }
}
