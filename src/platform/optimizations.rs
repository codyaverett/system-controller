use crate::platform::traits::*;
use crate::platform::factory::{PlatformFactory, PlatformCapabilities};
use crate::platform::cross_platform::CrossPlatformController;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Platform-specific optimization manager
pub struct PlatformOptimizations {
    platform_type: String,
    capabilities: PlatformCapabilities,
    optimization_config: OptimizationConfig,
    performance_cache: Arc<RwLock<PerformanceCache>>,
    operation_stats: Arc<RwLock<OperationStatistics>>,
}

/// Configuration for platform-specific optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub enable_batch_operations: bool,
    pub batch_size: usize,
    pub enable_caching: bool,
    pub cache_ttl: Duration,
    pub enable_adaptive_timing: bool,
    pub min_operation_interval: Duration,
    pub enable_memory_optimization: bool,
    pub max_memory_usage_mb: usize,
    pub enable_cpu_optimization: bool,
    pub max_cpu_usage_percent: f32,
}

/// Performance cache for optimization decisions
#[derive(Debug, Default)]
pub struct PerformanceCache {
    pub command_timings: HashMap<String, Vec<Duration>>,
    pub memory_usage_history: Vec<(SystemTime, usize)>,
    pub cpu_usage_history: Vec<(SystemTime, f32)>,
    pub error_counts: HashMap<String, usize>,
    pub last_optimization_time: Option<SystemTime>,
}

/// Operation statistics for performance tracking
#[derive(Debug, Default, Clone)]
pub struct OperationStatistics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_operation_time: Duration,
    pub peak_operations_per_second: f64,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f32,
    pub optimization_applied_count: u64,
    pub last_stats_update: Option<SystemTime>,
}

/// Optimized operation batch
#[derive(Debug, Clone)]
pub struct OptimizedBatch {
    pub operations: Vec<OptimizedOperation>,
    pub estimated_duration: Duration,
    pub memory_requirement: usize,
    pub priority: u8,
}

/// Individual optimized operation
#[derive(Debug, Clone)]
pub struct OptimizedOperation {
    pub operation_type: String,
    pub parameters: serde_json::Value,
    pub estimated_time: Duration,
    pub retry_count: u8,
}

/// Platform-specific optimization strategies
#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    /// High-performance strategy for GUI platforms
    HighPerformance {
        enable_hardware_acceleration: bool,
        parallel_operations: usize,
    },
    /// Conservative strategy for headless environments
    Conservative {
        minimize_resource_usage: bool,
        enable_graceful_degradation: bool,
    },
    /// Adaptive strategy that adjusts based on performance
    Adaptive {
        performance_threshold: f64,
        adaptation_interval: Duration,
    },
    /// Memory-optimized strategy for resource-constrained environments
    MemoryOptimized {
        max_cache_size: usize,
        enable_compression: bool,
    },
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_batch_operations: true,
            batch_size: 10,
            enable_caching: true,
            cache_ttl: Duration::from_secs(60),
            enable_adaptive_timing: true,
            min_operation_interval: Duration::from_millis(10),
            enable_memory_optimization: true,
            max_memory_usage_mb: 128,
            enable_cpu_optimization: true,
            max_cpu_usage_percent: 80.0,
        }
    }
}

impl PlatformOptimizations {
    /// Create optimizations for a platform
    pub async fn new(platform_type: String) -> Result<Self> {
        let capabilities = PlatformFactory::get_platform_capabilities();
        let optimization_config = Self::create_optimal_config(&platform_type, &capabilities);

        Ok(Self {
            platform_type,
            capabilities,
            optimization_config,
            performance_cache: Arc::new(RwLock::new(PerformanceCache::default())),
            operation_stats: Arc::new(RwLock::new(OperationStatistics::default())),
        })
    }

    /// Create optimal configuration based on platform type and capabilities
    fn create_optimal_config(platform_type: &str, capabilities: &PlatformCapabilities) -> OptimizationConfig {
        let mut config = OptimizationConfig::default();

        match platform_type {
            "headless" | "headless-silent" => {
                // Optimize for minimal resource usage
                config.batch_size = 50; // Larger batches for efficiency
                config.cache_ttl = Duration::from_secs(300); // Longer cache
                config.min_operation_interval = Duration::from_millis(1); // Minimal delay
                config.max_memory_usage_mb = 64; // Lower memory limit
                config.max_cpu_usage_percent = 50.0; // Conservative CPU usage
            }
            "enigo" => {
                // Optimize for real-time performance
                config.batch_size = 5; // Smaller batches for responsiveness
                config.cache_ttl = Duration::from_secs(30); // Shorter cache
                config.min_operation_interval = Duration::from_millis(16); // ~60fps
                config.max_memory_usage_mb = 256; // Higher memory for performance
                config.max_cpu_usage_percent = 90.0; // Allow high CPU usage
            }
            _ => {
                // Default balanced configuration
            }
        }

        // Adjust based on capabilities
        if !capabilities.has_gui {
            config.enable_batch_operations = true; // More batching without GUI
            config.batch_size *= 2; // Even larger batches
        }

        if !capabilities.supports_real_input {
            config.min_operation_interval = Duration::from_millis(1); // No need for timing
        }

        config
    }

    /// Get the optimization strategy for this platform
    pub fn get_optimization_strategy(&self) -> OptimizationStrategy {
        match self.platform_type.as_str() {
            "headless" | "headless-silent" => OptimizationStrategy::Conservative {
                minimize_resource_usage: true,
                enable_graceful_degradation: true,
            },
            "enigo" => OptimizationStrategy::HighPerformance {
                enable_hardware_acceleration: true,
                parallel_operations: 4,
            },
            _ => OptimizationStrategy::Adaptive {
                performance_threshold: 0.8,
                adaptation_interval: Duration::from_secs(60),
            },
        }
    }

    /// Optimize a batch of operations based on platform characteristics
    pub async fn optimize_operations(
        &self,
        operations: Vec<(String, serde_json::Value)>,
    ) -> Result<OptimizedBatch> {
        let start_time = Instant::now();

        // Analyze operations for optimization opportunities
        let mut optimized_ops = Vec::new();
        let mut total_estimated_time = Duration::ZERO;
        let mut memory_requirement = 0;

        for (op_type, params) in operations {
            let estimated_time = self.estimate_operation_time(&op_type).await;
            let memory_req = self.estimate_memory_requirement(&op_type);

            optimized_ops.push(OptimizedOperation {
                operation_type: op_type.clone(),
                parameters: params,
                estimated_time,
                retry_count: 0,
            });

            total_estimated_time += estimated_time;
            memory_requirement += memory_req;
        }

        // Apply platform-specific optimizations
        match self.get_optimization_strategy() {
            OptimizationStrategy::HighPerformance { enable_hardware_acceleration, parallel_operations } => {
                if enable_hardware_acceleration {
                    // Reduce estimated time for hardware acceleration
                    total_estimated_time = total_estimated_time.mul_f32(0.7);
                }
                if parallel_operations > 1 {
                    // Adjust for parallel execution
                    total_estimated_time = total_estimated_time.div_f32(parallel_operations.min(optimized_ops.len()) as f32);
                }
            }
            OptimizationStrategy::Conservative { minimize_resource_usage, .. } => {
                if minimize_resource_usage {
                    // Increase estimated time for conservative execution
                    total_estimated_time = total_estimated_time.mul_f32(1.2);
                    memory_requirement = (memory_requirement as f32 * 0.8) as usize;
                }
            }
            OptimizationStrategy::MemoryOptimized { enable_compression, .. } => {
                if enable_compression {
                    memory_requirement = (memory_requirement as f32 * 0.6) as usize;
                }
            }
            OptimizationStrategy::Adaptive { .. } => {
                // Apply adaptive optimizations based on current performance
                let stats = self.operation_stats.read().await;
                if stats.cpu_usage_percent > 80.0 {
                    total_estimated_time = total_estimated_time.mul_f32(1.1);
                }
            }
        }

        // Update statistics
        {
            let mut stats = self.operation_stats.write().await;
            stats.optimization_applied_count += 1;
        }

        Ok(OptimizedBatch {
            operations: optimized_ops,
            estimated_duration: total_estimated_time,
            memory_requirement,
            priority: self.calculate_priority(&start_time.elapsed()),
        })
    }

    /// Estimate operation time based on historical data
    async fn estimate_operation_time(&self, operation_type: &str) -> Duration {
        let cache = self.performance_cache.read().await;
        
        if let Some(timings) = cache.command_timings.get(operation_type) {
            if !timings.is_empty() {
                // Calculate average from recent timings
                let recent_timings: Vec<_> = timings.iter().rev().take(10).collect();
                let total: Duration = recent_timings.iter().copied().sum();
                return total / recent_timings.len() as u32;
            }
        }

        // Default estimates based on operation type
        match operation_type {
            "mouse_move" => Duration::from_millis(5),
            "mouse_click" => Duration::from_millis(10),
            "key_press" => Duration::from_millis(8),
            "type_text" => Duration::from_millis(50),
            "capture_screen" => Duration::from_millis(100),
            "get_displays" => Duration::from_millis(20),
            _ => Duration::from_millis(25),
        }
    }

    /// Estimate memory requirement for operation
    fn estimate_memory_requirement(&self, operation_type: &str) -> usize {
        match operation_type {
            "capture_screen" => 1024 * 1024 * 4, // ~4MB for screen data
            "get_displays" => 1024, // 1KB for display info
            "list_windows" => 4096, // 4KB for window list
            _ => 256, // 256B for simple operations
        }
    }

    /// Calculate operation priority based on optimization time
    fn calculate_priority(&self, optimization_time: &Duration) -> u8 {
        if optimization_time < &Duration::from_millis(10) {
            255 // High priority for fast optimizations
        } else if optimization_time < &Duration::from_millis(50) {
            128 // Medium priority
        } else {
            64 // Lower priority for slow optimizations
        }
    }

    /// Record operation performance for future optimizations
    pub async fn record_operation_performance(
        &self,
        operation_type: &str,
        duration: Duration,
        success: bool,
    ) {
        // Update performance cache
        {
            let mut cache = self.performance_cache.write().await;
            
            // Record timing
            let timings = cache.command_timings.entry(operation_type.to_string()).or_insert_with(Vec::new);
            timings.push(duration);
            
            // Keep only recent timings (last 100)
            if timings.len() > 100 {
                timings.drain(0..timings.len() - 100);
            }
            
            // Record errors
            if !success {
                let count = cache.error_counts.entry(operation_type.to_string()).or_insert(0);
                *count += 1;
            }
        }

        // Update operation statistics
        {
            let mut stats = self.operation_stats.write().await;
            stats.total_operations += 1;
            
            if success {
                stats.successful_operations += 1;
            } else {
                stats.failed_operations += 1;
            }
            
            // Update average operation time
            if stats.successful_operations > 0 {
                let total_time = stats.average_operation_time.as_millis() * stats.successful_operations.saturating_sub(1) as u128;
                let new_total = total_time + duration.as_millis();
                stats.average_operation_time = Duration::from_millis((new_total / stats.successful_operations as u128) as u64);
            } else {
                stats.average_operation_time = duration;
            }
            
            stats.last_stats_update = Some(SystemTime::now());
        }
    }

    /// Get current operation statistics
    pub async fn get_operation_statistics(&self) -> OperationStatistics {
        self.operation_stats.read().await.clone()
    }

    /// Apply adaptive optimizations based on current performance
    pub async fn apply_adaptive_optimizations(&mut self) -> Result<()> {
        let stats = self.operation_stats.read().await.clone();
        
        // Check if adaptation is needed
        let should_adapt = match stats.last_stats_update {
            Some(last_update) => {
                SystemTime::now().duration_since(last_update).unwrap_or(Duration::ZERO) 
                    > Duration::from_secs(60)
            }
            None => true,
        };

        if !should_adapt {
            return Ok(());
        }

        // Adapt configuration based on performance
        if stats.failed_operations > stats.successful_operations / 10 {
            // High error rate - be more conservative
            self.optimization_config.batch_size = self.optimization_config.batch_size.max(1) - 1;
            self.optimization_config.min_operation_interval = 
                self.optimization_config.min_operation_interval.mul_f32(1.2);
        } else if stats.average_operation_time < Duration::from_millis(10) {
            // Fast operations - can be more aggressive
            self.optimization_config.batch_size = (self.optimization_config.batch_size + 1).min(100);
            self.optimization_config.min_operation_interval = 
                self.optimization_config.min_operation_interval.mul_f32(0.9);
        }

        // Memory adaptation
        if stats.memory_usage_bytes > (self.optimization_config.max_memory_usage_mb * 1024 * 1024) as u64 {
            self.optimization_config.enable_caching = false;
            self.optimization_config.cache_ttl = self.optimization_config.cache_ttl.mul_f32(0.8);
        }

        tracing::info!("Applied adaptive optimizations for platform: {}", self.platform_type);
        Ok(())
    }

    /// Get optimization recommendations for the current platform
    pub async fn get_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        let stats = self.operation_stats.read().await;
        let cache = self.performance_cache.read().await;

        // Analyze error patterns
        for (operation, error_count) in &cache.error_counts {
            if *error_count > 10 {
                recommendations.push(OptimizationRecommendation {
                    category: "Error Reduction".to_string(),
                    description: format!("High error rate for {} operations", operation),
                    impact: "High".to_string(),
                    suggested_action: format!("Consider increasing retry count or timeout for {} operations", operation),
                });
            }
        }

        // Analyze performance patterns
        if stats.average_operation_time > Duration::from_millis(100) {
            recommendations.push(OptimizationRecommendation {
                category: "Performance".to_string(),
                description: "Average operation time is high".to_string(),
                impact: "Medium".to_string(),
                suggested_action: "Consider enabling batch operations or reducing operation complexity".to_string(),
            });
        }

        // Memory recommendations
        if stats.memory_usage_bytes > (64 * 1024 * 1024) as u64 {
            recommendations.push(OptimizationRecommendation {
                category: "Memory".to_string(),
                description: "High memory usage detected".to_string(),
                impact: "Medium".to_string(),
                suggested_action: "Enable memory optimization and consider reducing cache size".to_string(),
            });
        }

        recommendations
    }

    /// Clear performance cache and reset statistics
    pub async fn reset_performance_data(&self) {
        {
            let mut cache = self.performance_cache.write().await;
            *cache = PerformanceCache::default();
        }
        {
            let mut stats = self.operation_stats.write().await;
            *stats = OperationStatistics::default();
        }
    }
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub category: String,
    pub description: String,
    pub impact: String,
    pub suggested_action: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_platform_optimizations_creation() {
        let optimizations = PlatformOptimizations::new("headless".to_string()).await;
        assert!(optimizations.is_ok());
        
        let opt = optimizations.unwrap();
        assert_eq!(opt.platform_type, "headless");
    }

    #[tokio::test]
    async fn test_operation_optimization() {
        let optimizations = PlatformOptimizations::new("headless".to_string()).await.unwrap();
        
        let operations = vec![
            ("mouse_move".to_string(), serde_json::json!({"x": 100, "y": 100})),
            ("key_press".to_string(), serde_json::json!({"key": "a"})),
        ];
        
        let result = optimizations.optimize_operations(operations).await;
        assert!(result.is_ok());
        
        let batch = result.unwrap();
        assert_eq!(batch.operations.len(), 2);
        assert!(batch.estimated_duration > Duration::ZERO);
    }

    #[tokio::test]
    async fn test_performance_recording() {
        let optimizations = PlatformOptimizations::new("enigo".to_string()).await.unwrap();
        
        optimizations.record_operation_performance(
            "test_op",
            Duration::from_millis(50),
            true
        ).await;
        
        let stats = optimizations.get_operation_statistics().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.successful_operations, 1);
    }

    #[tokio::test]
    async fn test_adaptive_optimizations() {
        let mut optimizations = PlatformOptimizations::new("headless".to_string()).await.unwrap();
        
        // Record some operations
        for i in 0..20 {
            optimizations.record_operation_performance(
                "test_op",
                Duration::from_millis(i * 5),
                i % 10 != 0 // Some failures
            ).await;
        }
        
        let result = optimizations.apply_adaptive_optimizations().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_optimization_recommendations() {
        let optimizations = PlatformOptimizations::new("enigo".to_string()).await.unwrap();
        
        // Record some high-error operations
        for _ in 0..15 {
            optimizations.record_operation_performance(
                "problematic_op",
                Duration::from_millis(10),
                false
            ).await;
        }
        
        let recommendations = optimizations.get_optimization_recommendations().await;
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.category == "Error Reduction"));
    }

    #[tokio::test]
    async fn test_optimization_strategies() {
        let headless_opt = PlatformOptimizations::new("headless".to_string()).await.unwrap();
        let enigo_opt = PlatformOptimizations::new("enigo".to_string()).await.unwrap();
        
        match headless_opt.get_optimization_strategy() {
            OptimizationStrategy::Conservative { .. } => {}, // Expected
            _ => panic!("Expected Conservative strategy for headless"),
        }
        
        match enigo_opt.get_optimization_strategy() {
            OptimizationStrategy::HighPerformance { .. } => {}, // Expected
            _ => panic!("Expected HighPerformance strategy for enigo"),
        }
    }
}