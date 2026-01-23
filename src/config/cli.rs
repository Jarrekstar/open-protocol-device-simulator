//! Command-line interface argument parsing.
//!
//! This module defines CLI arguments using clap with environment variable support.

use clap::Parser;
use std::path::PathBuf;

/// Open Protocol Device Simulator
///
/// A configurable simulator for testing Open Protocol integrations.
#[derive(Parser, Debug)]
#[command(name = "open-protocol-device-simulator")]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Path to TOML configuration file
    #[arg(short, long, env = "SIMULATOR_CONFIG")]
    pub config: Option<PathBuf>,

    /// TCP port for Open Protocol connections
    #[arg(long, env = "SIMULATOR_TCP_PORT")]
    pub tcp_port: Option<u16>,

    /// HTTP port for REST API and WebSocket
    #[arg(long, env = "SIMULATOR_HTTP_PORT")]
    pub http_port: Option<u16>,

    /// Bind address for all listeners
    #[arg(long, env = "SIMULATOR_BIND_ADDRESS")]
    pub bind_address: Option<String>,

    /// Path to SQLite database file
    #[arg(long, env = "SIMULATOR_DB_PATH")]
    pub db_path: Option<PathBuf>,

    /// Cell ID for the simulated device
    #[arg(long, env = "SIMULATOR_CELL_ID")]
    pub cell_id: Option<u32>,

    /// Channel ID for the simulated device
    #[arg(long, env = "SIMULATOR_CHANNEL_ID")]
    pub channel_id: Option<u32>,

    /// Controller name reported in Open Protocol messages
    #[arg(long, env = "SIMULATOR_CONTROLLER_NAME")]
    pub controller_name: Option<String>,

    /// Supplier code reported in Open Protocol messages
    #[arg(long, env = "SIMULATOR_SUPPLIER_CODE")]
    pub supplier_code: Option<String>,

    /// Print the loaded configuration and exit
    #[arg(long)]
    pub print_config: bool,
}

impl CliArgs {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        CliArgs::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_defaults() {
        // Test that parsing with no args works (uses defaults/env)
        let args = CliArgs::try_parse_from(["test"]).unwrap();
        assert!(args.config.is_none());
        assert!(args.tcp_port.is_none());
        assert!(!args.print_config);
    }

    #[test]
    fn test_cli_with_args() {
        let args = CliArgs::try_parse_from([
            "test",
            "--tcp-port",
            "9080",
            "--http-port",
            "9081",
            "--cell-id",
            "5",
        ])
        .unwrap();

        assert_eq!(args.tcp_port, Some(9080));
        assert_eq!(args.http_port, Some(9081));
        assert_eq!(args.cell_id, Some(5));
    }

    #[test]
    fn test_cli_print_config() {
        let args = CliArgs::try_parse_from(["test", "--print-config"]).unwrap();
        assert!(args.print_config);
    }
}
