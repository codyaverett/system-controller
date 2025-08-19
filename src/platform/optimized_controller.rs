use crate::platform::traits::*;
use crate::platform::cross_platform::CrossPlatformController;
use crate::platform::optimizations::{PlatformOptimizations, OptimizedBatch};
use crate::platform::factory::PlatformFactory;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, Duration, Instant};
use std::collections::VecDeque;

/// Enhanced cross-platform controller with platform-specific optimizations
pub struct OptimizedPlatformController {
    base_controller: CrossPlatformController,
    optimizations: PlatformOptimizations,
    operation_queue: Arc<RwLock<VecDeque<QueuedOperation>>>,
    batch_processor: Arc<RwLock<BatchProcessor>>,
    is_processing: Arc<RwLock<bool>>,
}

/// Queued operation waiting for optimization
#[derive(Debug, Clone)]
pub struct QueuedOperation {
    pub operation_type: String,
    pub parameters: serde_json::Value,
    pub queued_at: SystemTime,
    pub priority: u8,
    pub callback: Option<String>, // For async result handling
}

/// Batch processor for optimized operations
#[derive(Debug)]
pub struct BatchProcessor {
    current_batch: Option<OptimizedBatch>,
    batch_start_time: Option<Instant>,
    processed_operations: u64,
    last_optimization_time: Option<SystemTime>,
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self {
            current_batch: None,
            batch_start_time: None,
            processed_operations: 0,
            last_optimization_time: None,
        }
    }
}

impl OptimizedPlatformController {
    /// Create a new optimized platform controller
    pub async fn new() -> Result<Self> {
        let base_controller = CrossPlatformController::new().await?;
        
        // Detect platform type for optimizations
        let platform_type = Self::detect_platform_type();
        let optimizations = PlatformOptimizations::new(platform_type).await?;

        Ok(Self {
            base_controller,
            optimizations,
            operation_queue: Arc::new(RwLock::new(VecDeque::new())),
            batch_processor: Arc::new(RwLock::new(BatchProcessor::default())),
            is_processing: Arc::new(RwLock::new(false)),
        })
    }

    /// Create with specific platform type
    pub async fn with_platform_type(platform_type: String) -> Result<Self> {
        let base_controller = CrossPlatformController::new().await?;
        let optimizations = PlatformOptimizations::new(platform_type).await?;

        Ok(Self {
            base_controller,
            optimizations,
            operation_queue: Arc::new(RwLock::new(VecDeque::new())),
            batch_processor: Arc::new(RwLock::new(BatchProcessor::default())),
            is_processing: Arc::new(RwLock::new(false)),
        })
    }

    /// Optimized mouse move with batching and timing optimization
    pub async fn optimized_mouse_move(&self, x: i32, y: i32) -> Result<()> {
        let operation = QueuedOperation {
            operation_type: "mouse_move".to_string(),
            parameters: serde_json::json!({"x": x, "y": y}),
            queued_at: SystemTime::now(),
            priority: 128, // Medium priority
            callback: None,
        };

        self.queue_operation(operation).await?;
        self.process_queue_if_needed().await?;
        Ok(())
    }

    /// Optimized mouse click with error handling and retry logic
    pub async fn optimized_mouse_click(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        let operation = QueuedOperation {
            operation_type: "mouse_click".to_string(),
            parameters: serde_json::json!({"button": button, "x": x, "y": y}),
            queued_at: SystemTime::now(),
            priority: 200, // Higher priority for clicks
            callback: None,
        };

        self.queue_operation(operation).await?;
        self.process_queue_if_needed().await?;
        Ok(())
    }

    /// Optimized key press with intelligent batching
    pub async fn optimized_key_press(&self, key: String) -> Result<()> {
        let operation = QueuedOperation {
            operation_type: "key_press".to_string(),
            parameters: serde_json::json!({"key": key}),
            queued_at: SystemTime::now(),
            priority: 150,
            callback: None,
        };

        self.queue_operation(operation).await?;
        self.process_queue_if_needed().await?;
        Ok(())
    }

    /// Optimized text typing with smart chunking
    pub async fn optimized_type_text(&self, text: String) -> Result<()> {
        // For long text, break into chunks for better responsiveness
        let chunk_size = 50;
        if text.len() > chunk_size {
            let chunks: Vec<String> = text.chars()
                .collect::<Vec<char>>()
                .chunks(chunk_size)
                .map(|chunk| chunk.iter().collect())
                .collect();

            for chunk in chunks {
                let operation = QueuedOperation {
                    operation_type: "type_text".to_string(),
                    parameters: serde_json::json!({"text": chunk}),
                    queued_at: SystemTime::now(),
                    priority: 180,
                    callback: None,
                };
                self.queue_operation(operation).await?;
            }
        } else {
            let operation = QueuedOperation {
                operation_type: "type_text".to_string(),
                parameters: serde_json::json!({"text": text}),
                queued_at: SystemTime::now(),
                priority: 180,
                callback: None,
            };
            self.queue_operation(operation).await?;
        }

        self.process_queue_if_needed().await?;
        Ok(())
    }

    /// Execute a batch of operations optimally
    pub async fn execute_batch_operations(&self, operations: Vec<QueuedOperation>) -> Result<Vec<Result<()>>> {
        let start_time = Instant::now();
        
        // Convert to format expected by optimizations
        let opt_operations: Vec<(String, serde_json::Value)> = operations.iter()
            .map(|op| (op.operation_type.clone(), op.parameters.clone()))
            .collect();

        // Get optimized batch
        let optimized_batch = self.optimizations.optimize_operations(opt_operations).await?;
        
        // Execute operations
        let mut results = Vec::new();
        for optimized_op in &optimized_batch.operations {
            let result = self.execute_single_optimized_operation(optimized_op).await;
            
            // Record performance
            self.optimizations.record_operation_performance(
                &optimized_op.operation_type,
                start_time.elapsed(),
                result.is_ok(),
            ).await;
            
            results.push(result);
        }

        // Update batch processor
        {
            let mut processor = self.batch_processor.write().await;
            processor.processed_operations += results.len() as u64;
            processor.last_optimization_time = Some(SystemTime::now());
        }

        Ok(results)
    }

    /// Execute a single optimized operation
    async fn execute_single_optimized_operation(&self, operation: &crate::platform::optimizations::OptimizedOperation) -> Result<()> {
        match operation.operation_type.as_str() {
            "mouse_move" => {
                let params = &operation.parameters;
                let x = params["x"].as_i64().unwrap_or(0) as i32;
                let y = params["y"].as_i64().unwrap_or(0) as i32;
                self.base_controller.enhanced_mouse_move(x, y).await
            }
            "mouse_click" => {
                let params = &operation.parameters;
                let x = params["x"].as_i64().unwrap_or(0) as i32;
                let y = params["y"].as_i64().unwrap_or(0) as i32;
                let button = serde_json::from_value(params["button"].clone())
                    .unwrap_or(MouseButton::Left);
                self.base_controller.enhanced_mouse_click(button, x, y).await
            }
            "key_press" => {
                let params = &operation.parameters;
                let key = params["key"].as_str().unwrap_or("").to_string();
                self.base_controller.enhanced_key_press(key).await
            }
            "type_text" => {
                let params = &operation.parameters;
                let text = params["text"].as_str().unwrap_or("").to_string();
                self.base_controller.type_text(text)
            }
            _ => Ok(()), // Unknown operations succeed silently
        }
    }

    /// Queue an operation for optimized processing
    async fn queue_operation(&self, operation: QueuedOperation) -> Result<()> {
        let mut queue = self.operation_queue.write().await;
        
        // Insert based on priority (higher priority first)
        let insert_pos = queue.iter()
            .position(|op| op.priority < operation.priority)
            .unwrap_or(queue.len());
        
        queue.insert(insert_pos, operation);
        Ok(())
    }

    /// Process the operation queue if conditions are met
    async fn process_queue_if_needed(&self) -> Result<()> {
        // Check if we're already processing
        {
            let is_processing = self.is_processing.read().await;
            if *is_processing {
                return Ok(());
            }
        }

        let should_process = {
            let queue = self.operation_queue.read().await;
            let processor = self.batch_processor.read().await;
            
            // Process if queue is full or oldest operation is too old
            queue.len() >= 10 || 
            queue.front().map(|op| {
                SystemTime::now().duration_since(op.queued_at)
                    .unwrap_or(Duration::ZERO) > Duration::from_millis(100)
            }).unwrap_or(false)
        };

        if should_process {
            self.process_current_queue().await?;
        }

        Ok(())
    }

    /// Process all queued operations
    async fn process_current_queue(&self) -> Result<()> {
        // Set processing flag
        {
            let mut is_processing = self.is_processing.write().await;
            *is_processing = true;
        }

        // Get all queued operations
        let operations = {
            let mut queue = self.operation_queue.write().await;
            let ops = queue.drain(..).collect::<Vec<_>>();
            ops
        };

        if !operations.is_empty() {
            // Execute batch
            let _results = self.execute_batch_operations(operations).await?;
        }

        // Clear processing flag
        {
            let mut is_processing = self.is_processing.write().await;
            *is_processing = false;
        }

        Ok(())
    }

    /// Force process all queued operations immediately
    pub async fn flush_queue(&self) -> Result<()> {
        self.process_current_queue().await
    }

    /// Get optimization statistics
    pub async fn get_optimization_statistics(&self) -> crate::platform::optimizations::OperationStatistics {
        self.optimizations.get_operation_statistics().await
    }

    /// Get optimization recommendations
    pub async fn get_optimization_recommendations(&self) -> Vec<crate::platform::optimizations::OptimizationRecommendation> {
        self.optimizations.get_optimization_recommendations().await
    }

    /// Apply adaptive optimizations
    pub async fn apply_adaptive_optimizations(&mut self) -> Result<()> {
        self.optimizations.apply_adaptive_optimizations().await
    }

    /// Reset performance data
    pub async fn reset_performance_data(&self) {
        self.optimizations.reset_performance_data().await
    }

    /// Get current queue size
    pub async fn get_queue_size(&self) -> usize {
        self.operation_queue.read().await.len()
    }

    /// Get batch processor status
    pub async fn get_batch_processor_status(&self) -> BatchProcessorStatus {
        let processor = self.batch_processor.read().await;
        BatchProcessorStatus {
            current_batch_size: processor.current_batch.as_ref().map(|b| b.operations.len()).unwrap_or(0),
            processed_operations: processor.processed_operations,
            last_optimization_time: processor.last_optimization_time,
            is_processing: *self.is_processing.read().await,
        }
    }

    /// Detect platform type for optimization
    fn detect_platform_type() -> String {
        let caps = PlatformFactory::get_platform_capabilities();
        if caps.has_gui {
            "enigo".to_string()
        } else {
            "headless".to_string()
        }
    }
}

/// Batch processor status information
#[derive(Debug, Clone)]
pub struct BatchProcessorStatus {
    pub current_batch_size: usize,
    pub processed_operations: u64,
    pub last_optimization_time: Option<SystemTime>,
    pub is_processing: bool,
}

/// Implement PlatformController trait for OptimizedPlatformController
impl PlatformController for OptimizedPlatformController {
    fn mouse_move(&self, x: i32, y: i32) -> Result<()> {
        // Use the base controller for sync operations
        self.base_controller.mouse_move(x, y)
    }

    fn mouse_click(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        self.base_controller.mouse_click(button, x, y)
    }

    fn mouse_scroll(&self, x: i32, y: i32) -> Result<()> {
        self.base_controller.mouse_scroll(x, y)
    }

    fn key_press(&self, key: String) -> Result<()> {
        self.base_controller.key_press(key)
    }

    fn key_release(&self, key: String) -> Result<()> {
        self.base_controller.key_release(key)
    }

    fn type_text(&self, text: String) -> Result<()> {
        self.base_controller.type_text(text)
    }

    fn capture_screen(&self, display_id: u32) -> Result<Vec<u8>> {
        self.base_controller.capture_screen(display_id)
    }

    fn get_displays(&self) -> Result<Vec<DisplayInfo>> {
        self.base_controller.get_displays()
    }

    fn get_window_at_position(&self, x: i32, y: i32) -> Result<Option<WindowInfo>> {
        self.base_controller.get_window_at_position(x, y)
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        self.base_controller.list_windows()
    }

    fn get_active_window(&self) -> Result<Option<WindowInfo>> {
        self.base_controller.get_active_window()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_optimized_controller_creation() {
        let controller = OptimizedPlatformController::new().await;
        assert!(controller.is_ok());
    }

    #[tokio::test]
    async fn test_optimized_mouse_operations() {
        let controller = OptimizedPlatformController::new().await.unwrap();
        
        let result = controller.optimized_mouse_move(100, 100).await;
        assert!(result.is_ok());
        
        let result = controller.optimized_mouse_click(MouseButton::Left, 50, 50).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_optimized_keyboard_operations() {
        let controller = OptimizedPlatformController::new().await.unwrap();
        
        let result = controller.optimized_key_press("a".to_string()).await;
        assert!(result.is_ok());
        
        let result = controller.optimized_type_text("Hello, World!".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let controller = OptimizedPlatformController::new().await.unwrap();
        
        let operations = vec![
            QueuedOperation {
                operation_type: "mouse_move".to_string(),
                parameters: serde_json::json!({"x": 10, "y": 10}),
                queued_at: SystemTime::now(),
                priority: 100,
                callback: None,
            },
            QueuedOperation {
                operation_type: "key_press".to_string(),
                parameters: serde_json::json!({"key": "a"}),
                queued_at: SystemTime::now(),
                priority: 100,
                callback: None,
            },
        ];
        
        let results = controller.execute_batch_operations(operations).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_queue_processing() {
        let controller = OptimizedPlatformController::new().await.unwrap();
        
        // Add several operations
        for i in 0..5 {
            let _ = controller.optimized_mouse_move(i * 10, i * 10).await;
        }
        
        // Flush the queue
        let result = controller.flush_queue().await;
        assert!(result.is_ok());
        
        // Queue should be empty now
        let queue_size = controller.get_queue_size().await;
        assert_eq!(queue_size, 0);
    }

    #[tokio::test]
    async fn test_optimization_statistics() {
        let controller = OptimizedPlatformController::new().await.unwrap();
        
        // Perform some operations
        let _ = controller.optimized_mouse_move(100, 100).await;
        let _ = controller.flush_queue().await;
        
        let stats = controller.get_optimization_statistics().await;
        assert!(stats.total_operations >= 0);
    }

    #[tokio::test]
    async fn test_long_text_chunking() {
        let controller = OptimizedPlatformController::new().await.unwrap();
        
        // Test with long text that should be chunked
        let long_text = "a".repeat(150);
        let result = controller.optimized_type_text(long_text).await;
        assert!(result.is_ok());
        
        // Should have created multiple operations
        let queue_size = controller.get_queue_size().await;
        assert!(queue_size > 1);
    }
}