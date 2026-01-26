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
    use std::fs;
    use std::path::PathBuf;

    /// Helper struct that cleans up a temporary file on drop
    struct TempFile {
        path: PathBuf,
    }

    impl TempFile {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(name);
            Self { path }
        }

        fn write(&self, content: &str) {
            fs::write(&self.path, content).expect("Failed to write temp file");
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

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

    #[test]
    fn test_load_valid_toml_file() {
        let temp_file = TempFile::new("test_valid_config.toml");
        temp_file.write(
            r#"
[server]
tcp_port = 9000
http_port = 9001
bind_address = "192.168.1.1"
event_channel_capacity = 200

[device]
cell_id = 42
channel_id = 7
controller_name = "TestSimulator"
supplier_code = "TST"

[database]
path = "/tmp/test.db"

[defaults]
auto_tightening_interval_ms = 5000
auto_tightening_duration_ms = 2000
failure_rate = 0.25
"#,
        );

        let settings = load_config_file(temp_file.path()).expect("Should load valid config");

        assert_eq!(settings.server.tcp_port, 9000);
        assert_eq!(settings.server.http_port, 9001);
        assert_eq!(settings.server.bind_address, "192.168.1.1");
        assert_eq!(settings.server.event_channel_capacity, 200);
        assert_eq!(settings.device.cell_id, 42);
        assert_eq!(settings.device.channel_id, 7);
        assert_eq!(settings.device.controller_name, "TestSimulator");
        assert_eq!(settings.device.supplier_code, "TST");
        assert_eq!(settings.database.path, PathBuf::from("/tmp/test.db"));
        assert_eq!(settings.defaults.auto_tightening_interval_ms, 5000);
        assert_eq!(settings.defaults.auto_tightening_duration_ms, 2000);
        assert!((settings.defaults.failure_rate - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let path = Path::new("/nonexistent/path/to/config.toml");
        let result = load_config_file(path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::FileError(_)));
    }

    #[test]
    fn test_load_invalid_toml_syntax() {
        let temp_file = TempFile::new("test_invalid_syntax.toml");
        temp_file.write(
            r#"
[server
tcp_port = 9000
this is not valid toml syntax!!!
"#,
        );

        let result = load_config_file(temp_file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::FileError(_)));
    }

    #[test]
    fn test_load_toml_wrong_types() {
        let temp_file = TempFile::new("test_wrong_types.toml");
        // tcp_port should be u16, not a string
        temp_file.write(
            r#"
[server]
tcp_port = "not a number"
"#,
        );

        let result = load_config_file(temp_file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::ParseError(_)));
    }

    #[test]
    fn test_config_error_display() {
        let file_error = ConfigError::FileError("file not found".to_string());
        let parse_error = ConfigError::ParseError("invalid type".to_string());

        let file_msg = format!("{}", file_error);
        let parse_msg = format!("{}", parse_error);

        assert!(file_msg.contains("Configuration file error"));
        assert!(file_msg.contains("file not found"));
        assert!(parse_msg.contains("Configuration parse error"));
        assert!(parse_msg.contains("invalid type"));
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        let temp_file = TempFile::new("test_partial_config.toml");
        // Only specify a few fields, rest should use defaults
        temp_file.write(
            r#"
[server]
tcp_port = 7777

[device]
controller_name = "PartialConfig"
"#,
        );

        let settings = load_config_file(temp_file.path()).expect("Should load partial config");

        // Specified values
        assert_eq!(settings.server.tcp_port, 7777);
        assert_eq!(settings.device.controller_name, "PartialConfig");

        // Default values for unspecified fields
        assert_eq!(settings.server.http_port, 8081);
        assert_eq!(settings.server.bind_address, "0.0.0.0");
        assert_eq!(settings.server.event_channel_capacity, 100);
        assert_eq!(settings.device.cell_id, 1);
        assert_eq!(settings.device.channel_id, 1);
        assert_eq!(settings.device.supplier_code, "SIM");
        assert_eq!(settings.database.path, PathBuf::from("simulator.db"));
        assert_eq!(settings.defaults.auto_tightening_interval_ms, 3000);
        assert_eq!(settings.defaults.auto_tightening_duration_ms, 1500);
        assert!((settings.defaults.failure_rate - 0.1).abs() < f64::EPSILON);
    }
}
