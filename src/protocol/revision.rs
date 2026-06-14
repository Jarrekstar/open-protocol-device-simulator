use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub type MidRevision = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionPolicy {
    Exact,
    AnyImplemented,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionSelection {
    pub enabled: bool,
    pub policy: RevisionPolicy,
    pub revision: MidRevision,
}

impl RevisionSelection {
    fn exact(revision: MidRevision) -> Self {
        Self {
            enabled: true,
            policy: RevisionPolicy::Exact,
            revision,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionFeature {
    pub revision: MidRevision,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidFamilyDefinition {
    pub id: String,
    pub name: String,
    pub mids: Vec<u16>,
    pub supported_revisions: Vec<MidRevision>,
    pub implemented_revisions: Vec<MidRevision>,
    pub default_revision: MidRevision,
    pub features: Vec<RevisionFeature>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProtocolSampleData {
    pub open_protocol_version: String,
    pub controller_software_version: String,
    pub tool_software_version: String,
    pub rbu_type: String,
    pub controller_serial_number: String,
    pub system_type: u8,
    pub system_subtype: u8,
    pub supports_sequence_number: bool,
    pub supports_message_linking: bool,
    pub station_id: u8,
    pub identifier_part_1: String,
    pub identifier_part_2: String,
    pub identifier_part_3: String,
    pub identifier_part_4: String,
    pub job_sequence_number: u16,
    pub job_tightening_status: u8,
    pub tool_serial_number: String,
    pub tightening_strategy: u8,
    pub tightening_strategy_options: u32,
    pub tightening_error_status: u32,
    pub tightening_error_status_2: u32,
    pub rundown_angle_min: u32,
    pub rundown_angle_max: u32,
    pub rundown_angle: u32,
    pub current_monitoring_min: u16,
    pub current_monitoring_max: u16,
    pub current_monitoring_value: u16,
    pub self_tap_min: u32,
    pub self_tap_max: u32,
    pub self_tap_torque: u32,
    pub prevail_torque_min: u32,
    pub prevail_torque_max: u32,
    pub prevail_torque: u32,
    pub prevail_torque_compensate: u32,
    pub torque_unit: u8,
    pub tightening_result_type: u8,
    pub customer_tightening_error_code: String,
    pub compensated_angle: u32,
    pub final_angle_decimal: u32,
    pub multistage_count: u8,
    pub multistage_torque: u32,
    pub multistage_angle: u32,
    pub multi_spindle_data_number: u64,
    pub multi_spindle_send_only_new: bool,
}

impl Default for ProtocolSampleData {
    fn default() -> Self {
        Self {
            open_protocol_version: "2.8.0".to_string(),
            controller_software_version: "SIM-1.0.0".to_string(),
            tool_software_version: "TOOL-1.0.0".to_string(),
            rbu_type: "SIM-RBU".to_string(),
            controller_serial_number: "SIM0000000000000001".to_string(),
            system_type: 0,
            system_subtype: 1,
            supports_sequence_number: false,
            supports_message_linking: false,
            station_id: 1,
            identifier_part_1: "VIN-SIM-00000000001".to_string(),
            identifier_part_2: "WORKORDER-0001".to_string(),
            identifier_part_3: "MODEL-SIMULATOR".to_string(),
            identifier_part_4: "BODY-00000001".to_string(),
            job_sequence_number: 1,
            job_tightening_status: 1,
            tool_serial_number: "SIMTOOL0000001".to_string(),
            tightening_strategy: 1,
            tightening_strategy_options: 7,
            tightening_error_status: 0,
            tightening_error_status_2: 0,
            rundown_angle_min: 10,
            rundown_angle_max: 30,
            rundown_angle: 20,
            current_monitoring_min: 80,
            current_monitoring_max: 120,
            current_monitoring_value: 100,
            self_tap_min: 500,
            self_tap_max: 900,
            self_tap_torque: 700,
            prevail_torque_min: 300,
            prevail_torque_max: 600,
            prevail_torque: 450,
            prevail_torque_compensate: 25,
            torque_unit: 1,
            tightening_result_type: 1,
            customer_tightening_error_code: "0000".to_string(),
            compensated_angle: 3950,
            final_angle_decimal: 3950,
            multistage_count: 1,
            multistage_torque: 1230,
            multistage_angle: 40,
            multi_spindle_data_number: 0,
            multi_spindle_send_only_new: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolProfile {
    pub version: u64,
    pub families: BTreeMap<String, RevisionSelection>,
    pub samples: ProtocolSampleData,
}

impl Default for ProtocolProfile {
    fn default() -> Self {
        let families = revision_catalog()
            .into_iter()
            .map(|family| (family.id, RevisionSelection::exact(family.default_revision)))
            .collect();
        Self {
            version: 1,
            families,
            samples: ProtocolSampleData::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolConfiguration {
    profile: Arc<RwLock<ProtocolProfile>>,
    storage_path: Option<Arc<PathBuf>>,
}

impl Default for ProtocolConfiguration {
    fn default() -> Self {
        Self::new(ProtocolProfile::default())
    }
}

impl ProtocolConfiguration {
    pub fn new(profile: ProtocolProfile) -> Self {
        Self {
            profile: Arc::new(RwLock::new(profile)),
            storage_path: None,
        }
    }

    pub fn persistent(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        let connection = Connection::open(&path)
            .map_err(|error| format!("Failed to open protocol profile database: {error}"))?;
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS protocol_profile (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    profile_json TEXT NOT NULL,
                    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )
            .map_err(|error| format!("Failed to initialize protocol profile storage: {error}"))?;

        let stored = connection
            .query_row(
                "SELECT profile_json FROM protocol_profile WHERE id = 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| format!("Failed to read protocol profile: {error}"))?;
        let profile = match stored {
            Some(json) => {
                let profile: ProtocolProfile = serde_json::from_str(&json)
                    .map_err(|error| format!("Stored protocol profile is invalid: {error}"))?;
                validate_profile(&profile)?;
                profile
            }
            None => {
                let profile = ProtocolProfile::default();
                persist_profile(&connection, &profile)?;
                profile
            }
        };

        Ok(Self {
            profile: Arc::new(RwLock::new(profile)),
            storage_path: Some(Arc::new(path)),
        })
    }

    pub fn profile(&self) -> ProtocolProfile {
        self.profile.read().unwrap().clone()
    }

    pub fn samples(&self) -> ProtocolSampleData {
        self.profile.read().unwrap().samples.clone()
    }

    pub fn update(&self, mut profile: ProtocolProfile) -> Result<ProtocolProfile, String> {
        validate_profile(&profile)?;
        let mut current = self.profile.write().unwrap();
        if profile.version != current.version {
            return Err(format!(
                "Protocol profile version {} is stale; current version is {}",
                profile.version, current.version
            ));
        }
        profile.version = current.version.saturating_add(1);
        if let Some(path) = &self.storage_path {
            let connection = Connection::open(path.as_ref())
                .map_err(|error| format!("Failed to open protocol profile database: {error}"))?;
            persist_profile(&connection, &profile)?;
        }
        *current = profile.clone();
        Ok(profile)
    }

    pub fn accepts(&self, mid: u16, revision: MidRevision) -> bool {
        let Some(family) = family_for_mid(mid) else {
            return true;
        };
        let profile = self.profile.read().unwrap();
        let Some(selection) = profile.families.get(&family.id) else {
            return false;
        };
        if !selection.enabled || !family.implemented_revisions.contains(&revision) {
            return false;
        }
        match selection.policy {
            RevisionPolicy::Exact => selection.revision == revision,
            RevisionPolicy::AnyImplemented => true,
        }
    }
}

fn persist_profile(connection: &Connection, profile: &ProtocolProfile) -> Result<(), String> {
    let json = serde_json::to_string(profile)
        .map_err(|error| format!("Failed to serialize protocol profile: {error}"))?;
    connection
        .execute(
            "INSERT INTO protocol_profile (id, profile_json, updated_at)
             VALUES (1, ?1, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                profile_json = excluded.profile_json,
                updated_at = CURRENT_TIMESTAMP",
            params![json],
        )
        .map_err(|error| format!("Failed to persist protocol profile: {error}"))?;
    Ok(())
}

pub fn validate_profile(profile: &ProtocolProfile) -> Result<(), String> {
    for family in revision_catalog() {
        let selection = profile
            .families
            .get(&family.id)
            .ok_or_else(|| format!("Missing protocol family '{}'", family.id))?;
        if !family.implemented_revisions.contains(&selection.revision) {
            return Err(format!(
                "Revision {} is not implemented for {}",
                selection.revision, family.name
            ));
        }
    }
    validate_ascii_width(
        "Open Protocol version",
        &profile.samples.open_protocol_version,
        19,
    )?;
    validate_ascii_width(
        "Controller software version",
        &profile.samples.controller_software_version,
        19,
    )?;
    validate_ascii_width(
        "Tool software version",
        &profile.samples.tool_software_version,
        19,
    )?;
    validate_ascii_width("RBU type", &profile.samples.rbu_type, 19)?;
    validate_ascii_width(
        "Controller serial number",
        &profile.samples.controller_serial_number,
        25,
    )?;
    for (name, value) in [
        ("Identifier part 1", &profile.samples.identifier_part_1),
        ("Identifier part 2", &profile.samples.identifier_part_2),
        ("Identifier part 3", &profile.samples.identifier_part_3),
        ("Identifier part 4", &profile.samples.identifier_part_4),
    ] {
        validate_ascii_width(name, value, 25)?;
    }
    validate_ascii_width(
        "Tool serial number",
        &profile.samples.tool_serial_number,
        14,
    )?;
    validate_ascii_width(
        "Customer tightening error code",
        &profile.samples.customer_tightening_error_code,
        4,
    )?;
    if profile.samples.system_type > 1 {
        return Err("System type must be 0 (tightening) or 1 (press)".to_string());
    }
    if profile.samples.system_subtype > 2 {
        return Err("System subtype must be in the range 0-2".to_string());
    }
    if profile.samples.station_id > 99 {
        return Err("Station ID must be in the range 0-99".to_string());
    }
    if profile.samples.job_tightening_status > 10 {
        return Err("Job tightening status must be in the range 0-10".to_string());
    }
    if profile.samples.tightening_strategy > 99 {
        return Err("Tightening strategy must be in the range 0-99".to_string());
    }
    if profile.samples.tightening_strategy_options > 99_999 {
        return Err("Tightening strategy options must fit in five digits".to_string());
    }
    if profile.samples.torque_unit == 0 || profile.samples.torque_unit > 8 {
        return Err("Torque unit must be in the range 1-8".to_string());
    }
    if profile.samples.tightening_result_type == 0 || profile.samples.tightening_result_type > 8 {
        return Err("Tightening result type must be in the range 1-8".to_string());
    }
    if profile.samples.multistage_count == 0 || profile.samples.multistage_count > 99 {
        return Err("Multistage count must be in the range 1-99".to_string());
    }
    for (name, value, maximum) in [
        (
            "Rundown angle min",
            profile.samples.rundown_angle_min,
            99_999,
        ),
        (
            "Rundown angle max",
            profile.samples.rundown_angle_max,
            99_999,
        ),
        ("Rundown angle", profile.samples.rundown_angle, 99_999),
        ("Self-tap min", profile.samples.self_tap_min, 999_999),
        ("Self-tap max", profile.samples.self_tap_max, 999_999),
        ("Self-tap torque", profile.samples.self_tap_torque, 999_999),
        (
            "Prevail torque min",
            profile.samples.prevail_torque_min,
            999_999,
        ),
        (
            "Prevail torque max",
            profile.samples.prevail_torque_max,
            999_999,
        ),
        ("Prevail torque", profile.samples.prevail_torque, 999_999),
        (
            "Prevail torque compensate",
            profile.samples.prevail_torque_compensate,
            999_999,
        ),
        (
            "Multistage torque",
            profile.samples.multistage_torque,
            999_999,
        ),
        ("Multistage angle", profile.samples.multistage_angle, 99_999),
    ] {
        if value > maximum {
            return Err(format!("{name} must be in the range 0-{maximum}"));
        }
    }
    for (name, value) in [
        (
            "Current monitoring min",
            profile.samples.current_monitoring_min,
        ),
        (
            "Current monitoring max",
            profile.samples.current_monitoring_max,
        ),
        (
            "Current monitoring value",
            profile.samples.current_monitoring_value,
        ),
    ] {
        if value > 999 {
            return Err(format!("{name} must be in the range 0-999"));
        }
    }
    if profile.samples.compensated_angle > 9_999_999
        || profile.samples.final_angle_decimal > 9_999_999
    {
        return Err("Decimal angle samples must fit in seven digits".to_string());
    }
    if profile.samples.multi_spindle_data_number > 9_999_999_999 {
        return Err("Multi-spindle data number must fit in ten digits".to_string());
    }
    Ok(())
}

fn validate_ascii_width(name: &str, value: &str, width: usize) -> Result<(), String> {
    if !value.is_ascii() {
        return Err(format!("{name} must contain ASCII characters only"));
    }
    if value.len() > width {
        return Err(format!("{name} must be no longer than {width} bytes"));
    }
    Ok(())
}

pub fn family_for_mid(mid: u16) -> Option<MidFamilyDefinition> {
    revision_catalog()
        .into_iter()
        .find(|family| family.mids.contains(&mid))
}

pub fn revision_catalog() -> Vec<MidFamilyDefinition> {
    vec![
        family(
            "communication",
            "Communication start",
            &[1, 2],
            &[1, 2, 3, 4, 5, 6],
            &[1, 2, 3, 4, 5, 6],
            &[
                (1, "Controller identity"),
                (2, "Supplier code"),
                (3, "Protocol and software versions"),
                (4, "RBU type and controller serial number"),
                (5, "System type and subtype"),
                (
                    6,
                    "Sequence number, message linking, and station ID support",
                ),
            ],
        ),
        family(
            "pset_selection",
            "PSET selection",
            &[14, 15, 16, 17],
            &[1, 2],
            &[1, 2],
            &[(1, "Selected PSET ID"), (2, "Extended PSET metadata")],
        ),
        family(
            "pset_commands",
            "PSET and batch commands",
            &[18, 19, 20, 128],
            &[1],
            &[1],
            &[(1, "PSET selection and batch control")],
        ),
        family(
            "job_ids",
            "Job ID upload",
            &[30, 31],
            &[1, 2],
            &[1, 2],
            &[(1, "Two-digit Job IDs"), (2, "Four-digit Job IDs")],
        ),
        family(
            "job_data",
            "Job data upload",
            &[32, 33],
            &[1, 2, 3],
            &[1, 2, 3],
            &[
                (1, "Basic Job definition"),
                (2, "Extended Job and step fields"),
                (3, "Additional Job metadata"),
            ],
        ),
        family(
            "job_info",
            "Job information",
            &[34, 35, 36, 37],
            &[1, 2, 3, 4, 5],
            &[1, 2, 3, 4, 5],
            &[
                (1, "Basic Job progress"),
                (2, "Four-digit Job IDs"),
                (3, "Current step, total steps, and step type"),
                (4, "Job tightening status"),
                (5, "Identifiers and sequence data"),
            ],
        ),
        family(
            "job_commands",
            "Job control",
            &[38, 39],
            &[1, 2],
            &[1, 2],
            &[(1, "Two-digit Job IDs"), (2, "Four-digit Job IDs")],
        ),
        family(
            "tool_control",
            "Tool control",
            &[42, 43],
            &[1],
            &[1],
            &[(1, "Enable and disable tool")],
        ),
        family(
            "vehicle_id",
            "Vehicle ID",
            &[50, 51, 52, 53, 54],
            &[1, 2],
            &[1, 2],
            &[
                (1, "Primary 25-character identifier"),
                (2, "Four configurable identifier parts"),
            ],
        ),
        family(
            "tightening_result",
            "Tightening result",
            &[60, 61, 62, 63],
            &[1, 2, 3, 4, 5, 6, 7, 998, 999],
            &[1, 2, 3, 4, 5, 6, 7, 998, 999],
            &[
                (1, "Core tightening result"),
                (2, "Strategy, monitoring, and extended result fields"),
                (3, "PSET name, torque unit, and result type"),
                (4, "Multiple identifier result parts"),
                (5, "Customer tightening error code"),
                (6, "PVT compensation and second error bit field"),
                (7, "Compensated and decimal angle values"),
                (998, "Multi-stage tightening result"),
                (999, "Compact tightening result"),
            ],
        ),
        family(
            "multi_spindle_status",
            "Multi-spindle status",
            &[90, 91, 92, 93],
            &[1],
            &[1],
            &[(1, "Sync tightening status")],
        ),
        family(
            "multi_spindle_result",
            "Multi-spindle result",
            &[100, 101, 102, 103],
            &[1, 2, 3, 4, 5],
            &[1, 2, 3, 4, 5],
            &[
                (1, "Per-spindle result"),
                (2, "Historical result rewind subscription"),
                (3, "Send-only-new subscription option"),
                (4, "System subtype for spindle or press data"),
                (5, "Job sequence number"),
            ],
        ),
        family(
            "keep_alive",
            "Keep alive",
            &[9999],
            &[1],
            &[1],
            &[(1, "Heartbeat")],
        ),
    ]
}

fn family(
    id: &str,
    name: &str,
    mids: &[u16],
    supported_revisions: &[MidRevision],
    implemented_revisions: &[MidRevision],
    features: &[(MidRevision, &str)],
) -> MidFamilyDefinition {
    MidFamilyDefinition {
        id: id.to_string(),
        name: name.to_string(),
        mids: mids.to_vec(),
        supported_revisions: supported_revisions.to_vec(),
        implemented_revisions: implemented_revisions.to_vec(),
        default_revision: 1,
        features: features
            .iter()
            .map(|(revision, summary)| RevisionFeature {
                revision: *revision,
                summary: (*summary).to_string(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_selects_revision_one() {
        let profile = ProtocolProfile::default();
        assert!(
            profile
                .families
                .values()
                .all(|selection| selection.revision == 1)
        );
        validate_profile(&profile).unwrap();
    }

    #[test]
    fn exact_and_any_implemented_policies_are_enforced() {
        let configuration = ProtocolConfiguration::default();
        assert!(configuration.accepts(1, 1));
        assert!(!configuration.accepts(1, 2));

        let mut profile = configuration.profile();
        let communication = profile.families.get_mut("communication").unwrap();
        communication.policy = RevisionPolicy::AnyImplemented;
        configuration.update(profile).unwrap();

        assert!(configuration.accepts(1, 6));
        assert!(!configuration.accepts(1, 7));
    }

    #[test]
    fn every_revision_for_existing_mid_families_has_a_codec() {
        for family in revision_catalog() {
            assert_eq!(
                family.implemented_revisions, family.supported_revisions,
                "{} still has unimplemented revisions",
                family.name
            );
            assert_eq!(
                family.features.len(),
                family.supported_revisions.len(),
                "{} is missing revision feature descriptions",
                family.name
            );
        }
    }

    #[test]
    fn persistent_configuration_round_trips_and_rejects_stale_updates() {
        let path = std::env::temp_dir().join(format!(
            "protocol-profile-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let configuration = ProtocolConfiguration::persistent(&path).unwrap();
        let stale = configuration.profile();
        let mut updated = stale.clone();
        updated.families.get_mut("communication").unwrap().revision = 6;
        configuration.update(updated).unwrap();
        assert!(configuration.update(stale).is_err());

        let reloaded = ProtocolConfiguration::persistent(&path).unwrap();
        assert_eq!(
            reloaded
                .profile()
                .families
                .get("communication")
                .unwrap()
                .revision,
            6
        );
        let _ = std::fs::remove_file(path);
    }
}
