use crate::platform::traits::*;
use crate::platform::{EnigoPlatform, HeadlessPlatform};
use anyhow::{Result, anyhow};
use std::env;

/// Platform factory for automatically detecting and creating the appropriate platform implementation
pub struct PlatformFactory;

impl PlatformFactory {
    /// Create the best available platform for the current environment
    pub fn create_platform() -> Result<Box<dyn PlatformController + Send + Sync>> {
        // Check for explicit environment variable
        if let Ok(platform_type) = env::var("SYSTEM_CONTROLLER_PLATFORM") {
            return Self::create_platform_by_name(&platform_type);
        }

        // Try to detect headless environment
        if Self::is_headless_environment() {
            tracing::info!("Detected headless environment, using HeadlessPlatform");
            return Ok(Box::new(HeadlessPlatform::new()));
        }

        // Try to create EnigoPlatform for GUI environments
        match EnigoPlatform::new() {
            Ok(platform) => {
                tracing::info!("Created EnigoPlatform for GUI environment");
                Ok(Box::new(platform))
            }
            Err(e) => {
                tracing::warn!("Failed to create EnigoPlatform: {}, falling back to HeadlessPlatform", e);
                Ok(Box::new(HeadlessPlatform::new()))
            }
        }
    }

    /// Create a platform by explicit name
    pub fn create_platform_by_name(name: &str) -> Result<Box<dyn PlatformController + Send + Sync>> {
        match name.to_lowercase().as_str() {
            "enigo" => {
                let platform = EnigoPlatform::new()?;
                Ok(Box::new(platform))
            }
            "headless" => {
                Ok(Box::new(HeadlessPlatform::new()))
            }
            "headless-silent" => {
                Ok(Box::new(HeadlessPlatform::new_silent()))
            }
            _ => Err(anyhow!("Unknown platform type: {}", name)),
        }
    }

    /// Detect if we're running in a headless environment
    fn is_headless_environment() -> bool {
        // Check for common headless environment indicators
        
        // Check if DISPLAY is set (Linux/X11)
        if env::var("DISPLAY").is_err() && Self::is_unix() {
            return true;
        }

        // Check for CI environment variables
        if env::var("CI").is_ok() || 
           env::var("GITHUB_ACTIONS").is_ok() ||
           env::var("TRAVIS").is_ok() ||
           env::var("JENKINS_URL").is_ok() {
            return true;
        }

        // Check for Docker environment
        if std::path::Path::new("/.dockerenv").exists() {
            return true;
        }

        // Check for SSH session
        if env::var("SSH_CLIENT").is_ok() || env::var("SSH_TTY").is_ok() {
            return true;
        }

        // Check if we're running in a terminal without GUI
        if env::var("TERM").is_ok() && env::var("DESKTOP_SESSION").is_err() {
            return true;
        }

        false
    }

    fn is_unix() -> bool {
        cfg!(unix)
    }

    /// Get platform capabilities for the current environment
    pub fn get_platform_capabilities() -> PlatformCapabilities {
        if Self::is_headless_environment() {
            PlatformCapabilities {
                has_gui: false,
                can_control_mouse: false,
                can_control_keyboard: false,
                can_capture_screen: false,
                can_enumerate_windows: false,
                supports_real_input: false,
            }
        } else {
            PlatformCapabilities {
                has_gui: true,
                can_control_mouse: true,
                can_control_keyboard: true,
                can_capture_screen: true, // Will be implemented in Phase 3
                can_enumerate_windows: true, // Will be implemented in Phase 3
                supports_real_input: true,
            }
        }
    }
}

/// Platform capabilities structure
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformCapabilities {
    pub has_gui: bool,
    pub can_control_mouse: bool,
    pub can_control_keyboard: bool,
    pub can_capture_screen: bool,
    pub can_enumerate_windows: bool,
    pub supports_real_input: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_factory_creation() {
        // Should create some platform without error
        let result = PlatformFactory::create_platform();
        assert!(result.is_ok());
    }

    #[test]
    fn test_platform_by_name() {
        let headless = PlatformFactory::create_platform_by_name("headless");
        assert!(headless.is_ok());

        let headless_silent = PlatformFactory::create_platform_by_name("headless-silent");
        assert!(headless_silent.is_ok());

        let invalid = PlatformFactory::create_platform_by_name("invalid");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_headless_detection() {
        // Test that headless detection doesn't crash
        let _is_headless = PlatformFactory::is_headless_environment();
        // We can't assert the value since it depends on the test environment
    }

    #[test]
    fn test_platform_capabilities() {
        let caps = PlatformFactory::get_platform_capabilities();
        // Should have some meaningful capabilities
        assert!(caps.has_gui || !caps.supports_real_input);
    }
}