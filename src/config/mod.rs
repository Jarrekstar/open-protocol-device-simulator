//! Configuration loading and management for the Open Protocol Device Simulator.
//!
//! This module implements a layered configuration system with the following priority:
//! 1. CLI arguments (highest priority)
//! 2. Environment variables
//! 3. Configuration file (TOML)
//! 4. Hardcoded defaults (lowest priority)

mod cli;
mod settings;

pub use cli::CliArgs;
pub use settings::{DatabaseConfig, DefaultsConfig, DeviceConfig, ServerConfig, Settings};

use config::{Config, File, FileFormat};
use std::path::Path;
use thiserror::Error;

/// Error type for configuration loading failures.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to read or parse configuration file
    #[error("Configuration file error: {0}")]
    FileError(String),
    /// Failed to deserialize configuration
    #[error("Configuration parse error: {0}")]
    ParseError(String),
}

/// Load configuration from all sources with proper layering.
///
/// Priority order (highest to lowest):
/// 1. CLI arguments
/// 2. Environment variables (handled by clap)
/// 3. Configuration file (if specified or default exists)
/// 4. Hardcoded defaults
///
/// # Returns
///
/// Returns the merged configuration settings, or exits the process if `--print-config` was specified.
///
/// # Errors
///
/// Returns `ConfigError` if a specified configuration file cannot be read or parsed.
pub fn load_config() -> Result<Settings, ConfigError> {
    let cli = CliArgs::parse_args();

    // Start with defaults
    let mut settings = Settings::default();

    // Load config file if specified or if default exists
    if let Some(config_path) = &cli.config {
        settings = load_config_file(config_path)?;
    } else {
        // Try loading default config files in order
        for default_path in ["config.toml", "simulator.toml"] {
            if Path::new(default_path).exists() {
                match load_config_file(Path::new(default_path)) {
                    Ok(file_settings) => {
                        settings = file_settings;
                        println!("Loaded configuration from {}", default_path);
                        break;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load {}: {}", default_path, e);
                    }
                }
            }
        }
    }

    // Apply CLI overrides (highest priority)
    apply_cli_overrides(&mut settings, &cli);

    // Handle --print-config
    if cli.print_config {
        print_config(&settings);
        std::process::exit(0);
    }

    Ok(settings)
}

/// Load settings from a TOML configuration file.
fn load_config_file(path: &Path) -> Result<Settings, ConfigError> {
    let config = Config::builder()
        .add_source(File::new(
            path.to_str().unwrap_or("config.toml"),
            FileFormat::Toml,
        ))
        .build()
        .map_err(|e| ConfigError::FileError(e.to_string()))?;

    config
        .try_deserialize()
        .map_err(|e| ConfigError::ParseError(e.to_string()))
}

/// Apply CLI argument overrides to settings.
fn apply_cli_overrides(settings: &mut Settings, cli: &CliArgs) {
    // Server overrides
    if let Some(port) = cli.tcp_port {
        settings.server.tcp_port = port;
    }
    if let Some(port) = cli.http_port {
        settings.server.http_port = port;
    }
    if let Some(ref addr) = cli.bind_address {
        settings.server.bind_address = addr.clone();
    }

    // Database overrides
    if let Some(ref path) = cli.db_path {
        settings.database.path = path.clone();
    }

    // Device overrides
    if let Some(cell_id) = cli.cell_id {
        settings.device.cell_id = cell_id;
    }
    if let Some(channel_id) = cli.channel_id {
        settings.device.channel_id = channel_id;
    }
    if let Some(ref name) = cli.controller_name {
        settings.device.controller_name = name.clone();
    }
    if let Some(ref code) = cli.supplier_code {
        settings.device.supplier_code = code.clone();
    }
}

/// Print configuration in a readable format.
fn print_config(settings: &Settings) {
    println!("Current Configuration:");
    println!("======================");
    println!();
    println!("[server]");
    println!("  tcp_port = {}", settings.server.tcp_port);
    println!("  http_port = {}", settings.server.http_port);
    println!("  bind_address = \"{}\"", settings.server.bind_address);
    println!(
        "  event_channel_capacity = {}",
        settings.server.event_channel_capacity
    );
    println!();
    println!("[device]");
    println!("  cell_id = {}", settings.device.cell_id);
    println!("  channel_id = {}", settings.device.channel_id);
    println!(
        "  controller_name = \"{}\"",
        settings.device.controller_name
    );
    println!("  supplier_code = \"{}\"", settings.device.supplier_code);
    println!();
    println!("[database]");
    println!("  path = \"{}\"", settings.database.path.display());
    println!();
    println!("[defaults]");
    println!(
        "  auto_tightening_interval_ms = {}",
        settings.defaults.auto_tightening_interval_ms
    );
    println!(
        "  auto_tightening_duration_ms = {}",
        settings.defaults.auto_tightening_duration_ms
    );
    println!("  failure_rate = {}", settings.defaults.failure_rate);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let settings = Settings::default();
        assert_eq!(settings.server.tcp_port, 8080);
        assert_eq!(settings.server.http_port, 8081);
    }

    #[test]
    fn test_cli_overrides() {
        let mut settings = Settings::default();
        let cli = CliArgs {
            config: None,
            tcp_port: Some(9080),
            http_port: Some(9081),
            bind_address: Some("127.0.0.1".to_string()),
            db_path: None,
            cell_id: Some(5),
            channel_id: None,
            controller_name: Some("TestController".to_string()),
            supplier_code: None,
            print_config: false,
        };

        apply_cli_overrides(&mut settings, &cli);

        assert_eq!(settings.server.tcp_port, 9080);
        assert_eq!(settings.server.http_port, 9081);
        assert_eq!(settings.server.bind_address, "127.0.0.1");
        assert_eq!(settings.device.cell_id, 5);
        assert_eq!(settings.device.controller_name, "TestController");
        // Unchanged values should remain at defaults
        assert_eq!(settings.device.channel_id, 1);
        assert_eq!(settings.device.supplier_code, "SIM");
    }
}
