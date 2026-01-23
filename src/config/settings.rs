//! Configuration structures for the Open Protocol Device Simulator.
//!
//! This module defines the settings hierarchy used throughout the application.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Root configuration structure containing all settings.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Server configuration (ports, addresses)
    #[serde(default)]
    pub server: ServerConfig,

    /// Device identification configuration
    #[serde(default)]
    pub device: DeviceConfig,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Default values for various operations
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

/// Server configuration for TCP and HTTP listeners.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// TCP port for Open Protocol connections (default: 8080)
    #[serde(default = "default_tcp_port")]
    pub tcp_port: u16,

    /// HTTP port for REST API and WebSocket (default: 8081)
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// Bind address for all listeners (default: "0.0.0.0")
    #[serde(default = "default_bind_address")]
    pub bind_address: String,

    /// Capacity of the event broadcast channel (default: 100)
    #[serde(default = "default_event_channel_capacity")]
    pub event_channel_capacity: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            tcp_port: default_tcp_port(),
            http_port: default_http_port(),
            bind_address: default_bind_address(),
            event_channel_capacity: default_event_channel_capacity(),
        }
    }
}

fn default_tcp_port() -> u16 {
    8080
}

fn default_http_port() -> u16 {
    8081
}

fn default_bind_address() -> String {
    "0.0.0.0".to_string()
}

fn default_event_channel_capacity() -> usize {
    100
}

/// Device identification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// Cell ID for the simulated device (default: 1)
    #[serde(default = "default_cell_id")]
    pub cell_id: u32,

    /// Channel ID for the simulated device (default: 1)
    #[serde(default = "default_channel_id")]
    pub channel_id: u32,

    /// Controller name reported in Open Protocol messages (default: "OpenProtocolSimulator")
    #[serde(default = "default_controller_name")]
    pub controller_name: String,

    /// Supplier code reported in Open Protocol messages (default: "SIM")
    #[serde(default = "default_supplier_code")]
    pub supplier_code: String,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            cell_id: default_cell_id(),
            channel_id: default_channel_id(),
            controller_name: default_controller_name(),
            supplier_code: default_supplier_code(),
        }
    }
}

fn default_cell_id() -> u32 {
    1
}

fn default_channel_id() -> u32 {
    1
}

fn default_controller_name() -> String {
    "OpenProtocolSimulator".to_string()
}

fn default_supplier_code() -> String {
    "SIM".to_string()
}

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file (default: "simulator.db")
    #[serde(default = "default_db_path")]
    pub path: PathBuf,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
        }
    }
}

fn default_db_path() -> PathBuf {
    PathBuf::from("simulator.db")
}

/// Default values for various simulation operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default interval between auto-tightening cycles in milliseconds (default: 3000)
    #[serde(default = "default_auto_tightening_interval")]
    pub auto_tightening_interval_ms: u64,

    /// Default duration of each tightening operation in milliseconds (default: 1500)
    #[serde(default = "default_auto_tightening_duration")]
    pub auto_tightening_duration_ms: u64,

    /// Default failure rate for auto-tightening (0.0-1.0, default: 0.1)
    #[serde(default = "default_failure_rate")]
    pub failure_rate: f64,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            auto_tightening_interval_ms: default_auto_tightening_interval(),
            auto_tightening_duration_ms: default_auto_tightening_duration(),
            failure_rate: default_failure_rate(),
        }
    }
}

fn default_auto_tightening_interval() -> u64 {
    3000
}

fn default_auto_tightening_duration() -> u64 {
    1500
}

fn default_failure_rate() -> f64 {
    0.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.server.tcp_port, 8080);
        assert_eq!(settings.server.http_port, 8081);
        assert_eq!(settings.server.bind_address, "0.0.0.0");
        assert_eq!(settings.device.cell_id, 1);
        assert_eq!(settings.device.controller_name, "OpenProtocolSimulator");
        assert_eq!(settings.database.path, PathBuf::from("simulator.db"));
        assert_eq!(settings.defaults.auto_tightening_interval_ms, 3000);
    }
}
