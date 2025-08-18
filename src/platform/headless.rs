use crate::platform::traits::*;
use anyhow::{Result, anyhow};

/// Headless platform implementation for environments without GUI
/// Useful for testing or server environments
pub struct HeadlessPlatform {
    enable_logging: bool,
}

impl HeadlessPlatform {
    pub fn new() -> Self {
        Self {
            enable_logging: true,
        }
    }

    pub fn new_silent() -> Self {
        Self {
            enable_logging: false,
        }
    }

    fn log_action(&self, action: &str) {
        if self.enable_logging {
            tracing::info!("Headless platform: {}", action);
        }
    }
}

impl Default for HeadlessPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformController for HeadlessPlatform {
    fn mouse_move(&self, x: i32, y: i32) -> Result<()> {
        self.log_action(&format!("Mouse move to ({}, {})", x, y));
        // In headless mode, we simulate success but don't actually move the mouse
        Ok(())
    }

    fn mouse_click(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        self.log_action(&format!("Mouse click {:?} at ({}, {})", button, x, y));
        Ok(())
    }

    fn mouse_scroll(&self, x: i32, y: i32) -> Result<()> {
        self.log_action(&format!("Mouse scroll ({}, {})", x, y));
        Ok(())
    }

    fn key_press(&self, key: String) -> Result<()> {
        self.log_action(&format!("Key press: {}", key));
        Ok(())
    }

    fn key_release(&self, key: String) -> Result<()> {
        self.log_action(&format!("Key release: {}", key));
        Ok(())
    }

    fn type_text(&self, text: String) -> Result<()> {
        self.log_action(&format!("Type text: {}", text));
        Ok(())
    }

    fn capture_screen(&self, display_id: u32) -> Result<Vec<u8>> {
        self.log_action(&format!("Screen capture for display {}", display_id));
        // Return a minimal PNG header for testing
        Ok(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) // PNG signature
    }

    fn get_displays(&self) -> Result<Vec<DisplayInfo>> {
        self.log_action("Get displays");
        // Return a mock display for testing
        Ok(vec![DisplayInfo {
            id: 0,
            name: "Headless Display".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            is_primary: true,
        }])
    }

    fn get_window_at_position(&self, x: i32, y: i32) -> Result<Option<WindowInfo>> {
        self.log_action(&format!("Get window at ({}, {})", x, y));
        // Return a mock window for testing
        Ok(Some(WindowInfo {
            id: 12345,
            title: "Headless Window".to_string(),
            x,
            y,
            width: 800,
            height: 600,
            process_name: "headless".to_string(),
        }))
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        self.log_action("List windows");
        // Return mock windows for testing
        Ok(vec![
            WindowInfo {
                id: 1,
                title: "Terminal".to_string(),
                x: 0,
                y: 0,
                width: 800,
                height: 600,
                process_name: "terminal".to_string(),
            },
            WindowInfo {
                id: 2,
                title: "Editor".to_string(),
                x: 800,
                y: 0,
                width: 1120,
                height: 1080,
                process_name: "editor".to_string(),
            },
        ])
    }

    fn get_active_window(&self) -> Result<Option<WindowInfo>> {
        self.log_action("Get active window");
        // Return the first mock window as active
        Ok(Some(WindowInfo {
            id: 1,
            title: "Active Terminal".to_string(),
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            process_name: "terminal".to_string(),
        }))
    }
}

// Mark as thread-safe
unsafe impl Send for HeadlessPlatform {}
unsafe impl Sync for HeadlessPlatform {}