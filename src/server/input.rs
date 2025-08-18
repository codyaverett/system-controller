use crate::platform::traits::*;
use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Input validation utilities
#[derive(Debug, Clone)]
pub struct InputValidator {
    max_text_length: usize,
}

impl InputValidator {
    pub fn new() -> Self {
        Self {
            max_text_length: 1000,
        }
    }

    pub fn validate_coordinates(&self, x: i32, y: i32) -> Result<()> {
        if x < 0 || y < 0 {
            return Err(anyhow!("Coordinates cannot be negative: ({}, {})", x, y));
        }
        Ok(())
    }

    pub fn validate_key_name(&self, key: &str) -> Result<()> {
        if key.is_empty() {
            return Err(anyhow!("Key name cannot be empty"));
        }
        
        // Check for null bytes or other invalid characters
        if key.contains('\0') {
            return Err(anyhow!("Key name contains invalid characters"));
        }
        
        Ok(())
    }

    pub fn validate_text_input(&self, text: &str) -> Result<()> {
        if text.len() > self.max_text_length {
            return Err(anyhow!("Text input too long: {} characters (max: {})", 
                text.len(), self.max_text_length));
        }
        Ok(())
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiting for input operations
#[derive(Debug)]
struct RateLimiter {
    max_requests: usize,
    window_duration: Duration,
    requests: VecDeque<Instant>,
}

impl RateLimiter {
    fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            requests: VecDeque::new(),
        }
    }

    fn check_rate_limit(&mut self) -> Result<()> {
        let now = Instant::now();
        
        // Remove old requests outside the window
        while let Some(&front) = self.requests.front() {
            if now.duration_since(front) > self.window_duration {
                self.requests.pop_front();
            } else {
                break;
            }
        }
        
        // Check if we're at the limit
        if self.requests.len() >= self.max_requests {
            return Err(anyhow!("Rate limit exceeded: {} requests per {:?}", 
                self.max_requests, self.window_duration));
        }
        
        // Add current request
        self.requests.push_back(now);
        Ok(())
    }
}

/// Main input controller for handling mouse and keyboard operations
pub struct InputController {
    platform: Box<dyn PlatformController + Send + Sync>,
    validator: InputValidator,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
}

impl InputController {
    pub fn new(platform: Box<dyn PlatformController + Send + Sync>) -> Self {
        Self {
            platform,
            validator: InputValidator::new(),
            rate_limiter: None,
        }
    }

    pub fn with_rate_limit(
        platform: Box<dyn PlatformController + Send + Sync>,
        max_requests: usize,
        window_duration: Duration,
    ) -> Self {
        Self {
            platform,
            validator: InputValidator::new(),
            rate_limiter: Some(Arc::new(Mutex::new(
                RateLimiter::new(max_requests, window_duration)
            ))),
        }
    }

    fn check_rate_limit(&self) -> Result<()> {
        if let Some(rate_limiter) = &self.rate_limiter {
            let mut limiter = rate_limiter.lock()
                .map_err(|_| anyhow!("Failed to acquire rate limiter lock"))?;
            limiter.check_rate_limit()?;
        }
        Ok(())
    }

    // Mouse control methods
    pub async fn move_mouse(&self, x: i32, y: i32) -> Result<()> {
        self.check_rate_limit()?;
        self.validator.validate_coordinates(x, y)
            .map_err(|e| anyhow!("Invalid mouse coordinates: {}", e))?;
        
        self.platform.mouse_move(x, y)
            .map_err(|e| anyhow!("Failed to move mouse: {}", e))
    }

    pub async fn click_mouse(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        self.check_rate_limit()?;
        self.validator.validate_coordinates(x, y)
            .map_err(|e| anyhow!("Invalid click coordinates: {}", e))?;
        
        self.platform.mouse_click(button, x, y)
            .map_err(|e| anyhow!("Failed to click mouse: {}", e))
    }

    pub async fn double_click_mouse(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        self.check_rate_limit()?;
        self.validator.validate_coordinates(x, y)?;
        
        // Perform two rapid clicks
        self.platform.mouse_click(button.clone(), x, y)?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.platform.mouse_click(button, x, y)?;
        
        Ok(())
    }

    pub async fn scroll_mouse(&self, x: i32, y: i32) -> Result<()> {
        self.check_rate_limit()?;
        
        self.platform.mouse_scroll(x, y)
            .map_err(|e| anyhow!("Failed to scroll mouse: {}", e))
    }

    // Keyboard control methods
    pub async fn press_key(&self, key: &str) -> Result<()> {
        self.check_rate_limit()?;
        self.validator.validate_key_name(key)
            .map_err(|e| anyhow!("Invalid key name: {}", e))?;
        
        self.platform.key_press(key.to_string())
            .map_err(|e| anyhow!("Failed to press key: {}", e))
    }

    pub async fn release_key(&self, key: &str) -> Result<()> {
        self.check_rate_limit()?;
        self.validator.validate_key_name(key)?;
        
        self.platform.key_release(key.to_string())
            .map_err(|e| anyhow!("Failed to release key: {}", e))
    }

    pub async fn type_text(&self, text: &str) -> Result<()> {
        self.check_rate_limit()?;
        self.validator.validate_text_input(text)
            .map_err(|e| anyhow!("Invalid text input: {}", e))?;
        
        self.platform.type_text(text.to_string())
            .map_err(|e| anyhow!("Failed to type text: {}", e))
    }

    pub async fn key_combination(&self, keys: &[&str]) -> Result<()> {
        self.check_rate_limit()?;
        
        if keys.is_empty() {
            return Err(anyhow!("Key combination cannot be empty"));
        }

        // Validate all keys first
        for key in keys {
            self.validator.validate_key_name(key)?;
        }

        // Press all keys in order
        for key in keys {
            self.platform.key_press(key.to_string())?;
        }

        // Small delay between press and release
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Release all keys in reverse order
        for key in keys.iter().rev() {
            self.platform.key_release(key.to_string())?;
        }

        Ok(())
    }
}

// Make InputController thread-safe for concurrent operations
unsafe impl Send for InputController {}
unsafe impl Sync for InputController {}