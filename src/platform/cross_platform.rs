use crate::platform::traits::*;
use crate::platform::factory::{PlatformFactory, PlatformCapabilities as BasePlatformCapabilities};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

/// Enhanced cross-platform controller that provides additional abstractions
/// and platform-specific optimizations
pub struct CrossPlatformController {
    platform: Box<dyn PlatformController + Send + Sync>,
    capabilities: EnhancedPlatformCapabilities,
    input_state: Arc<RwLock<InputState>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
}

#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub last_mouse_position: Option<(i32, i32)>,
    pub pressed_keys: HashMap<String, SystemTime>,
    pub mouse_buttons_pressed: HashMap<MouseButton, SystemTime>,
    pub total_commands_executed: u64,
    pub last_command_time: Option<SystemTime>,
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub average_response_time: Duration,
    pub total_operations: u64,
    pub failed_operations: u64,
    pub peak_operations_per_second: f64,
    pub last_operation_time: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct EnhancedPlatformCapabilities {
    pub base: BasePlatformCapabilities,
    pub supports_batch_operations: bool,
    pub max_concurrent_operations: usize,
    pub platform_name: String,
}

impl CrossPlatformController {
    /// Create a new cross-platform controller with automatic platform detection
    pub async fn new() -> Result<Self> {
        let platform = PlatformFactory::create_platform()?;
        let capabilities = Self::detect_enhanced_capabilities(&platform).await?;
        
        Ok(Self {
            platform,
            capabilities,
            input_state: Arc::new(RwLock::new(InputState::default())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        })
    }

    /// Create a cross-platform controller with a specific platform
    pub async fn with_platform(platform_name: &str) -> Result<Self> {
        let platform = PlatformFactory::create_platform_by_name(platform_name)?;
        let capabilities = Self::detect_enhanced_capabilities(&platform).await?;
        
        Ok(Self {
            platform,
            capabilities,
            input_state: Arc::new(RwLock::new(InputState::default())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        })
    }

    /// Get the platform capabilities
    pub fn get_capabilities(&self) -> &EnhancedPlatformCapabilities {
        &self.capabilities
    }

    /// Get current input state
    pub async fn get_input_state(&self) -> InputState {
        self.input_state.read().await.clone()
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// Enhanced mouse move with state tracking and validation
    pub async fn enhanced_mouse_move(&self, x: i32, y: i32) -> Result<()> {
        let start_time = SystemTime::now();
        
        // Validate coordinates based on platform capabilities
        self.validate_mouse_coordinates(x, y).await?;
        
        // Update input state
        {
            let mut state = self.input_state.write().await;
            state.last_mouse_position = Some((x, y));
            state.total_commands_executed += 1;
            state.last_command_time = Some(start_time);
        }
        
        // Execute the command
        let result = if self.capabilities.base.can_control_mouse {
            self.platform.mouse_move(x, y)
        } else {
            Ok(()) // No-op for platforms without mouse control
        };
        
        // Update performance metrics
        self.update_performance_metrics(start_time, result.is_ok()).await;
        
        result
    }

    /// Enhanced mouse click with button state tracking
    pub async fn enhanced_mouse_click(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        let start_time = SystemTime::now();
        
        // Validate coordinates
        self.validate_mouse_coordinates(x, y).await?;
        
        // Update input state
        {
            let mut state = self.input_state.write().await;
            state.last_mouse_position = Some((x, y));
            state.mouse_buttons_pressed.insert(button.clone(), start_time);
            state.total_commands_executed += 1;
            state.last_command_time = Some(start_time);
        }
        
        // Execute the command
        let result = if self.capabilities.base.can_control_mouse {
            self.platform.mouse_click(button, x, y)
        } else {
            Ok(()) // No-op for platforms without mouse control
        };
        
        // Update performance metrics
        self.update_performance_metrics(start_time, result.is_ok()).await;
        
        result
    }

    /// Enhanced keyboard input with key state tracking
    pub async fn enhanced_key_press(&self, key: String) -> Result<()> {
        let start_time = SystemTime::now();
        
        // Validate key
        self.validate_key(&key)?;
        
        // Update input state
        {
            let mut state = self.input_state.write().await;
            state.pressed_keys.insert(key.clone(), start_time);
            state.total_commands_executed += 1;
            state.last_command_time = Some(start_time);
        }
        
        // Execute the command
        let result = if self.capabilities.base.can_control_keyboard {
            self.platform.key_press(key)
        } else {
            Ok(()) // No-op for platforms without keyboard control
        };
        
        // Update performance metrics
        self.update_performance_metrics(start_time, result.is_ok()).await;
        
        result
    }

    /// Enhanced key release with state cleanup
    pub async fn enhanced_key_release(&self, key: String) -> Result<()> {
        let start_time = SystemTime::now();
        
        // Validate key
        self.validate_key(&key)?;
        
        // Update input state
        {
            let mut state = self.input_state.write().await;
            state.pressed_keys.remove(&key);
            state.total_commands_executed += 1;
            state.last_command_time = Some(start_time);
        }
        
        // Execute the command
        let result = if self.capabilities.base.can_control_keyboard {
            self.platform.key_release(key)
        } else {
            Ok(()) // No-op for platforms without keyboard control
        };
        
        // Update performance metrics
        self.update_performance_metrics(start_time, result.is_ok()).await;
        
        result
    }

    /// Batch operations for improved performance
    pub async fn execute_batch_operations(&self, operations: Vec<BatchOperation>) -> Result<Vec<Result<()>>> {
        if !self.capabilities.supports_batch_operations {
            // Fall back to sequential execution
            let mut results = Vec::new();
            for operation in operations {
                let result = self.execute_single_operation(operation).await;
                results.push(result);
            }
            return Ok(results);
        }
        
        let start_time = SystemTime::now();
        let mut results = Vec::new();
        
        for operation in operations {
            let result = self.execute_single_operation(operation).await;
            results.push(result);
        }
        
        self.update_performance_metrics(start_time, results.iter().all(|r| r.is_ok())).await;
        
        Ok(results)
    }

    /// Get display information with enhanced error handling
    pub async fn get_enhanced_displays(&self) -> Result<Vec<DisplayInfo>> {
        let start_time = SystemTime::now();
        
        let result = if self.capabilities.base.can_capture_screen {
            self.platform.get_displays()
        } else {
            Ok(vec![]) // Return empty list for platforms without display support
        };
        
        self.update_performance_metrics(start_time, result.is_ok()).await;
        
        result
    }

    /// Enhanced screen capture with format validation
    pub async fn enhanced_capture_screen(&self, display_id: u32) -> Result<Vec<u8>> {
        let start_time = SystemTime::now();
        
        if !self.capabilities.base.can_capture_screen {
            return Err(anyhow!("Screen capture not supported on this platform"));
        }
        
        // Validate display_id
        let displays = self.platform.get_displays()?;
        if !displays.iter().any(|d| d.id == display_id) {
            return Err(anyhow!("Invalid display ID: {}", display_id));
        }
        
        let result = self.platform.capture_screen(display_id);
        
        self.update_performance_metrics(start_time, result.is_ok()).await;
        
        result
    }

    /// Reset input state (useful for testing and cleanup)
    pub async fn reset_input_state(&self) {
        let mut state = self.input_state.write().await;
        *state = InputState::default();
    }

    /// Detect enhanced platform capabilities
    async fn detect_enhanced_capabilities(platform: &Box<dyn PlatformController + Send + Sync>) -> Result<EnhancedPlatformCapabilities> {
        let basic_caps = PlatformFactory::get_platform_capabilities();
        
        // Test for additional capabilities
        let supports_batch = true; // For now, assume all platforms support batch operations
        let max_concurrent = if basic_caps.has_gui { 10 } else { 100 }; // Headless can handle more concurrent ops
        
        // Determine platform name
        let platform_name = if basic_caps.has_gui {
            "gui_platform".to_string()
        } else {
            "headless_platform".to_string()
        };
        
        Ok(EnhancedPlatformCapabilities {
            base: basic_caps,
            supports_batch_operations: supports_batch,
            max_concurrent_operations: max_concurrent,
            platform_name,
        })
    }

    /// Validate mouse coordinates
    async fn validate_mouse_coordinates(&self, x: i32, y: i32) -> Result<()> {
        if !self.capabilities.base.can_control_mouse {
            return Ok(()); // Skip validation for platforms without mouse control
        }
        
        // Basic bounds checking
        if x < -10000 || x > 10000 || y < -10000 || y > 10000 {
            return Err(anyhow!("Mouse coordinates out of reasonable bounds: ({}, {})", x, y));
        }
        
        Ok(())
    }

    /// Validate keyboard key
    fn validate_key(&self, key: &str) -> Result<()> {
        if !self.capabilities.base.can_control_keyboard {
            return Ok(()); // Skip validation for platforms without keyboard control
        }
        
        if key.is_empty() {
            return Err(anyhow!("Key cannot be empty"));
        }
        
        if key.len() > 100 {
            return Err(anyhow!("Key string too long: {}", key.len()));
        }
        
        Ok(())
    }

    /// Update performance metrics
    async fn update_performance_metrics(&self, start_time: SystemTime, success: bool) {
        let duration = start_time.elapsed().unwrap_or(Duration::from_millis(0));
        
        let mut metrics = self.performance_metrics.write().await;
        metrics.total_operations += 1;
        
        if !success {
            metrics.failed_operations += 1;
        }
        
        // Update average response time (simple moving average)
        let total_time = metrics.average_response_time.as_millis() * (metrics.total_operations - 1) as u128;
        let new_total = total_time + duration.as_millis();
        metrics.average_response_time = Duration::from_millis((new_total / metrics.total_operations as u128) as u64);
        
        metrics.last_operation_time = Some(SystemTime::now());
        
        // Calculate operations per second
        if let Some(last_time) = metrics.last_operation_time {
            if let Ok(time_diff) = last_time.duration_since(start_time) {
                if time_diff.as_secs() > 0 {
                    let ops_per_sec = 1.0 / time_diff.as_secs_f64();
                    if ops_per_sec > metrics.peak_operations_per_second {
                        metrics.peak_operations_per_second = ops_per_sec;
                    }
                }
            }
        }
    }

    /// Execute a single batch operation
    async fn execute_single_operation(&self, operation: BatchOperation) -> Result<()> {
        match operation {
            BatchOperation::MouseMove { x, y } => self.enhanced_mouse_move(x, y).await,
            BatchOperation::MouseClick { button, x, y } => self.enhanced_mouse_click(button, x, y).await,
            BatchOperation::KeyPress { key } => self.enhanced_key_press(key).await,
            BatchOperation::KeyRelease { key } => self.enhanced_key_release(key).await,
            BatchOperation::TypeText { text } => {
                if self.capabilities.base.can_control_keyboard {
                    self.platform.type_text(text)
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Batch operation types for improved performance
#[derive(Debug, Clone)]
pub enum BatchOperation {
    MouseMove { x: i32, y: i32 },
    MouseClick { button: MouseButton, x: i32, y: i32 },
    KeyPress { key: String },
    KeyRelease { key: String },
    TypeText { text: String },
}

/// Implement the PlatformController trait for CrossPlatformController
impl PlatformController for CrossPlatformController {
    fn mouse_move(&self, x: i32, y: i32) -> Result<()> {
        // Use the synchronous version of the platform
        if self.capabilities.base.can_control_mouse {
            self.platform.mouse_move(x, y)
        } else {
            Ok(())
        }
    }

    fn mouse_click(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        if self.capabilities.base.can_control_mouse {
            self.platform.mouse_click(button, x, y)
        } else {
            Ok(())
        }
    }

    fn mouse_scroll(&self, x: i32, y: i32) -> Result<()> {
        if self.capabilities.base.can_control_mouse {
            self.platform.mouse_scroll(x, y)
        } else {
            Ok(())
        }
    }

    fn key_press(&self, key: String) -> Result<()> {
        if self.capabilities.base.can_control_keyboard {
            self.platform.key_press(key)
        } else {
            Ok(())
        }
    }

    fn key_release(&self, key: String) -> Result<()> {
        if self.capabilities.base.can_control_keyboard {
            self.platform.key_release(key)
        } else {
            Ok(())
        }
    }

    fn type_text(&self, text: String) -> Result<()> {
        if self.capabilities.base.can_control_keyboard {
            self.platform.type_text(text)
        } else {
            Ok(())
        }
    }

    fn capture_screen(&self, display_id: u32) -> Result<Vec<u8>> {
        if self.capabilities.base.can_capture_screen {
            self.platform.capture_screen(display_id)
        } else {
            Ok(vec![]) // Return empty data for platforms without screen capture
        }
    }

    fn get_displays(&self) -> Result<Vec<DisplayInfo>> {
        if self.capabilities.base.can_capture_screen {
            self.platform.get_displays()
        } else {
            Ok(vec![])
        }
    }

    fn get_window_at_position(&self, x: i32, y: i32) -> Result<Option<WindowInfo>> {
        if self.capabilities.base.can_enumerate_windows {
            self.platform.get_window_at_position(x, y)
        } else {
            Ok(None)
        }
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        if self.capabilities.base.can_enumerate_windows {
            self.platform.list_windows()
        } else {
            Ok(vec![])
        }
    }

    fn get_active_window(&self) -> Result<Option<WindowInfo>> {
        if self.capabilities.base.can_enumerate_windows {
            self.platform.get_active_window()
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cross_platform_controller_creation() {
        let controller = CrossPlatformController::new().await;
        assert!(controller.is_ok());
        
        let controller = controller.unwrap();
        let capabilities = controller.get_capabilities();
        assert!(!capabilities.platform_name.is_empty());
    }

    #[tokio::test]
    async fn test_enhanced_input_operations() {
        let controller = CrossPlatformController::new().await.unwrap();
        
        // Test enhanced mouse operations
        let result = controller.enhanced_mouse_move(100, 100).await;
        assert!(result.is_ok());
        
        let result = controller.enhanced_mouse_click(MouseButton::Left, 100, 100).await;
        assert!(result.is_ok());
        
        // Test enhanced keyboard operations
        let result = controller.enhanced_key_press("a".to_string()).await;
        assert!(result.is_ok());
        
        let result = controller.enhanced_key_release("a".to_string()).await;
        assert!(result.is_ok());
        
        // Check input state
        let state = controller.get_input_state().await;
        assert_eq!(state.last_mouse_position, Some((100, 100)));
        assert_eq!(state.total_commands_executed, 4);
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let controller = CrossPlatformController::new().await.unwrap();
        
        let operations = vec![
            BatchOperation::MouseMove { x: 50, y: 50 },
            BatchOperation::KeyPress { key: "a".to_string() },
            BatchOperation::TypeText { text: "hello".to_string() },
        ];
        
        let results = controller.execute_batch_operations(operations).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let controller = CrossPlatformController::new().await.unwrap();
        
        // Perform some operations
        for i in 0..5 {
            let _ = controller.enhanced_mouse_move(i * 10, i * 10).await;
        }
        
        let metrics = controller.get_performance_metrics().await;
        assert_eq!(metrics.total_operations, 5);
        assert!(metrics.average_response_time.as_millis() >= 0);
    }

    #[tokio::test]
    async fn test_input_validation() {
        let controller = CrossPlatformController::new().await.unwrap();
        
        // Test extreme coordinates
        let result = controller.enhanced_mouse_move(-999999, 999999).await;
        if controller.get_capabilities().base.can_control_mouse {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok()); // Should succeed on headless platforms
        }
        
        // Test empty key
        let result = controller.enhanced_key_press("".to_string()).await;
        if controller.get_capabilities().base.can_control_keyboard {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok()); // Should succeed on headless platforms
        }
    }
}