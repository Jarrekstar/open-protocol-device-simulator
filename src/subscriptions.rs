use crate::protocol::revision::MidRevision;
use serde::Serialize;

/// Manages client subscription state for various event types
#[derive(Debug, Clone, Default, Serialize)]
pub struct Subscriptions {
    /// Subscribed to tightening result events (MID 0061)
    pub tightening_result: bool,

    /// Subscribed to parameter set selection events (MID 0015)
    pub pset_selection: bool,

    /// Subscribed to vehicle ID events (MID 0052)
    pub vehicle_id: bool,

    /// Subscribed to multi-spindle status events (MID 0091)
    pub multi_spindle_status: bool,

    /// Subscribed to multi-spindle result events (MID 0101)
    pub multi_spindle_result: bool,

    /// Subscribed to alarm events (not yet implemented)
    pub alarm: bool,

    #[serde(skip)]
    tightening_result_revision: Option<MidRevision>,
    #[serde(skip)]
    pset_selection_revision: Option<MidRevision>,
    #[serde(skip)]
    vehicle_id_revision: Option<MidRevision>,
    #[serde(skip)]
    multi_spindle_status_revision: Option<MidRevision>,
    #[serde(skip)]
    multi_spindle_result_revision: Option<MidRevision>,

    /// Requested MID 0035 revision for this connection.
    pub job_info_revision: Option<MidRevision>,
}

impl Subscriptions {
    /// Create a new subscription manager with all subscriptions disabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to tightening result events
    pub fn subscribe_tightening_result(&mut self) {
        self.subscribe_tightening_result_revision(1);
    }

    pub fn subscribe_tightening_result_revision(&mut self, revision: MidRevision) {
        self.tightening_result = true;
        self.tightening_result_revision = Some(revision);
    }

    /// Unsubscribe from tightening result events
    pub fn unsubscribe_tightening_result(&mut self) {
        self.tightening_result = false;
        self.tightening_result_revision = None;
    }

    /// Subscribe to parameter set selection events
    pub fn subscribe_pset_selection(&mut self) {
        self.subscribe_pset_selection_revision(1);
    }

    pub fn subscribe_pset_selection_revision(&mut self, revision: MidRevision) {
        self.pset_selection = true;
        self.pset_selection_revision = Some(revision);
    }

    /// Unsubscribe from parameter set selection events
    pub fn unsubscribe_pset_selection(&mut self) {
        self.pset_selection = false;
        self.pset_selection_revision = None;
    }

    /// Check if subscribed to tightening results
    pub fn is_subscribed_to_tightening_result(&self) -> bool {
        self.tightening_result
    }

    /// Check if subscribed to pset selection
    pub fn is_subscribed_to_pset_selection(&self) -> bool {
        self.pset_selection
    }

    pub fn tightening_result_revision(&self) -> Option<MidRevision> {
        self.tightening_result_revision
    }

    pub fn pset_selection_revision(&self) -> Option<MidRevision> {
        self.pset_selection_revision
    }

    /// Subscribe to vehicle ID events
    pub fn subscribe_vehicle_id(&mut self) {
        self.subscribe_vehicle_id_revision(1);
    }

    pub fn subscribe_vehicle_id_revision(&mut self, revision: MidRevision) {
        self.vehicle_id = true;
        self.vehicle_id_revision = Some(revision);
    }

    /// Unsubscribe from vehicle ID events
    pub fn unsubscribe_vehicle_id(&mut self) {
        self.vehicle_id = false;
        self.vehicle_id_revision = None;
    }

    /// Check if subscribed to vehicle ID
    pub fn is_subscribed_to_vehicle_id(&self) -> bool {
        self.vehicle_id
    }

    pub fn vehicle_id_revision(&self) -> Option<MidRevision> {
        self.vehicle_id_revision
    }

    /// Subscribe to multi-spindle status events
    pub fn subscribe_multi_spindle_status(&mut self) {
        self.subscribe_multi_spindle_status_revision(1);
    }

    pub fn subscribe_multi_spindle_status_revision(&mut self, revision: MidRevision) {
        self.multi_spindle_status = true;
        self.multi_spindle_status_revision = Some(revision);
    }

    /// Unsubscribe from multi-spindle status events
    pub fn unsubscribe_multi_spindle_status(&mut self) {
        self.multi_spindle_status = false;
        self.multi_spindle_status_revision = None;
    }

    /// Check if subscribed to multi-spindle status
    pub fn is_subscribed_to_multi_spindle_status(&self) -> bool {
        self.multi_spindle_status
    }

    pub fn multi_spindle_status_revision(&self) -> Option<MidRevision> {
        self.multi_spindle_status_revision
    }

    /// Subscribe to multi-spindle result events
    pub fn subscribe_multi_spindle_result(&mut self) {
        self.subscribe_multi_spindle_result_revision(1);
    }

    pub fn subscribe_multi_spindle_result_revision(&mut self, revision: MidRevision) {
        self.multi_spindle_result = true;
        self.multi_spindle_result_revision = Some(revision);
    }

    /// Unsubscribe from multi-spindle result events
    pub fn unsubscribe_multi_spindle_result(&mut self) {
        self.multi_spindle_result = false;
        self.multi_spindle_result_revision = None;
    }

    /// Check if subscribed to multi-spindle result
    pub fn is_subscribed_to_multi_spindle_result(&self) -> bool {
        self.multi_spindle_result
    }

    pub fn multi_spindle_result_revision(&self) -> Option<MidRevision> {
        self.multi_spindle_result_revision
    }

    pub fn subscribe_job_info(&mut self, revision: MidRevision) -> bool {
        if self.job_info_revision.is_some() {
            false
        } else {
            self.job_info_revision = Some(revision);
            true
        }
    }

    pub fn unsubscribe_job_info(&mut self) -> bool {
        self.job_info_revision.take().is_some()
    }

    pub fn job_info_revision(&self) -> Option<MidRevision> {
        self.job_info_revision
    }

    /// Get count of active subscriptions
    ///
    /// Diagnostic method for subscription statistics.
    /// Used by webUI connection dashboard to display per-client
    /// subscription counts and by monitoring/metrics endpoints.
    #[allow(dead_code)]
    pub fn active_count(&self) -> usize {
        let mut count = 0;
        if self.tightening_result {
            count += 1;
        }
        if self.pset_selection {
            count += 1;
        }
        if self.vehicle_id {
            count += 1;
        }
        if self.multi_spindle_status {
            count += 1;
        }
        if self.multi_spindle_result {
            count += 1;
        }
        if self.alarm {
            count += 1;
        }
        if self.job_info_revision.is_some() {
            count += 1;
        }
        count
    }

    /// Check if any subscriptions are active
    ///
    /// Convenience method for subscription status checks.
    /// Used by connection lifecycle management to determine whether to
    /// keep idle connections alive, and by webUI for client status display.
    #[allow(dead_code)]
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

    #[test]
    fn stores_revision_for_each_subscription_family() {
        let mut subs = Subscriptions::new();
        subs.subscribe_tightening_result_revision(7);
        subs.subscribe_vehicle_id_revision(2);

        assert_eq!(subs.tightening_result_revision(), Some(7));
        assert_eq!(subs.vehicle_id_revision(), Some(2));

        subs.unsubscribe_vehicle_id();
        assert_eq!(subs.vehicle_id_revision(), None);
    }
}
