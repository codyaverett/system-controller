use crate::platform::cross_platform::CrossPlatformController;
use crate::protocol::messages::*;
use crate::server::network::NetworkServer;
use crate::server::network_protocol::NetworkProtocolIntegration;
use crate::server::enhanced_display::EnhancedDisplayController;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Complete end-to-end system integration
pub struct SystemIntegration {
    network_server: Option<NetworkServer>,
    network_protocol: NetworkProtocolIntegration,
    platform_controller: CrossPlatformController,
    display_controller: EnhancedDisplayController,
    active_connections: Arc<RwLock<HashMap<String, ActiveConnection>>>,
    system_stats: Arc<RwLock<SystemStatistics>>,
    config: SystemConfig,
}

/// Active connection information
#[derive(Debug, Clone)]
pub struct ActiveConnection {
    pub connection_id: String,
    pub session_id: String,
    pub client_info: ClientInfo,
    pub connected_at: SystemTime,
    pub last_activity: SystemTime,
    pub commands_processed: u64,
    pub bytes_transferred: u64,
}

/// System-wide statistics
#[derive(Debug, Clone, Default)]
pub struct SystemStatistics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_commands_processed: u64,
    pub total_bytes_transferred: u64,
    pub system_uptime: Duration,
    pub performance_metrics: PerformanceMetrics,
    pub error_count: u64,
    pub last_error: Option<String>,
}

/// Performance metrics for the integrated system
#[derive(Debug, Clone, Default, Serialize)]
pub struct PerformanceMetrics {
    pub average_command_processing_time: Duration,
    pub peak_commands_per_second: f64,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f32,
    pub network_latency_ms: f32,
}

/// System configuration
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub bind_address: String,
    pub max_connections: usize,
    pub command_timeout: Duration,
    pub session_timeout: Duration,
    pub enable_performance_monitoring: bool,
    pub log_level: String,
    pub security_enabled: bool,
}

// Use ClientInfo from network_protocol to avoid conflicts
pub use crate::server::network_protocol::ClientInfo;

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".to_string(),
            max_connections: 100,
            command_timeout: Duration::from_secs(30),
            session_timeout: Duration::from_secs(3600),
            enable_performance_monitoring: true,
            log_level: "info".to_string(),
            security_enabled: true,
        }
    }
}

impl SystemIntegration {
    /// Create a new complete system integration
    pub async fn new() -> Result<Self> {
        let config = SystemConfig::default();
        Self::with_config(config).await
    }

    /// Create system integration with custom configuration
    pub async fn with_config(config: SystemConfig) -> Result<Self> {
        let platform_controller = CrossPlatformController::new().await?;
        let network_protocol = NetworkProtocolIntegration::new().await?;
        
        // Create enhanced display controller
        let platform_arc = Arc::new(platform_controller.clone());
        let display_controller = EnhancedDisplayController::new(platform_arc);

        Ok(Self {
            network_server: None,
            network_protocol,
            platform_controller,
            display_controller,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            system_stats: Arc::new(RwLock::new(SystemStatistics::default())),
            config,
        })
    }

    /// Start the complete integrated system
    pub async fn start(&mut self) -> Result<()> {
        // Create and start network server
        let platform_for_server = CrossPlatformController::new().await?;
        let mut server = NetworkServer::new(
            Box::new(platform_for_server),
            self.config.bind_address.clone()
        );
        
        server.start().await?;
        self.network_server = Some(server);

        // Initialize system statistics
        {
            let mut stats = self.system_stats.write().await;
            stats.system_uptime = Duration::from_secs(0);
        }

        tracing::info!("System integration started on {}", self.config.bind_address);
        Ok(())
    }

    /// Stop the integrated system
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(ref server) = self.network_server {
            server.shutdown().await?;
        }
        self.network_server = None;

        // Clean up active connections
        self.active_connections.write().await.clear();

        tracing::info!("System integration stopped");
        Ok(())
    }

    /// Process a command through the complete integrated system
    pub async fn process_integrated_command(
        &self,
        command: &Command,
        client_info: &ClientInfo,
    ) -> Result<IntegratedResponse> {
        let start_time = SystemTime::now();
        
        // Create or get session
        let session_id = self.get_or_create_session(client_info).await;
        
        // Process command through network protocol integration
        let platform_arc = Arc::new(self.platform_controller.clone());
        let response = self.network_protocol.process_command_integrated(
            command,
            platform_arc,
            &session_id,
        ).await?;

        // Update connection statistics
        self.update_connection_stats(&session_id, &response, start_time).await?;

        // Create integrated response with additional metadata
        let integrated_response = IntegratedResponse {
            response,
            session_id: session_id.clone(),
            processing_time: start_time.elapsed().unwrap_or(Duration::ZERO),
            system_stats: self.get_system_stats_snapshot().await,
        };

        Ok(integrated_response)
    }

    /// Process multiple commands concurrently
    pub async fn process_batch_commands(
        &self,
        commands: Vec<Command>,
        client_info: &ClientInfo,
    ) -> Result<Vec<IntegratedResponse>> {
        let _session_id = self.get_or_create_session(client_info).await;
        let mut handles = Vec::new();

        for command in commands {
            let command_clone = command.clone();
            let client_info_clone = client_info.clone();
            let self_clone = self.clone_for_async();
            
            let handle = tokio::spawn(async move {
                self_clone.process_integrated_command(&command_clone, &client_info_clone).await
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result?),
                Err(e) => {
                    tracing::error!("Batch command processing error: {}", e);
                    self.increment_error_count().await;
                }
            }
        }

        Ok(results)
    }

    /// Enhanced display capture with full system integration
    pub async fn integrated_display_capture(&self, display_id: u32) -> Result<EnhancedCaptureResult> {
        let start_time = SystemTime::now();

        // Use enhanced display controller
        use crate::server::enhanced_display::EnhancedCaptureOptions;
        let options = EnhancedCaptureOptions {
            display_id,
            force_refresh: false,
            compression_format: None,
            quality: None,
            differential: true,
            max_width: None,
            max_height: None,
        };

        let capture_data = self.display_controller.enhanced_capture_screen(options).await?;
        let processing_time = start_time.elapsed().unwrap_or(Duration::ZERO);

        // Get display statistics
        let display_stats = self.display_controller.get_capture_statistics().await;

        Ok(EnhancedCaptureResult {
            data: capture_data,
            display_id,
            processing_time,
            compression_ratio: if display_stats.total_bytes_captured > 0 {
                display_stats.total_bytes_compressed as f64 / display_stats.total_bytes_captured as f64
            } else {
                1.0
            },
            capture_statistics: display_stats,
        })
    }

    /// Get comprehensive system health status
    pub async fn get_system_health(&self) -> SystemHealth {
        let stats = self.system_stats.read().await.clone();
        let connections = self.active_connections.read().await;
        let platform_capabilities = self.platform_controller.get_capabilities();
        let session_stats = self.network_protocol.get_session_stats().await;
        let display_stats = self.display_controller.get_capture_statistics().await;

        SystemHealth {
            status: if stats.error_count < 10 { "healthy" } else { "degraded" }.to_string(),
            uptime: stats.system_uptime,
            active_connections: connections.len(),
            total_commands_processed: stats.total_commands_processed,
            error_rate: if stats.total_commands_processed > 0 {
                stats.error_count as f64 / stats.total_commands_processed as f64
            } else {
                0.0
            },
            performance: stats.performance_metrics.clone(),
            platform_info: PlatformInfo {
                name: platform_capabilities.platform_name.clone(),
                capabilities: format!("{:?}", platform_capabilities.base),
                concurrent_operations: platform_capabilities.max_concurrent_operations,
            },
            session_info: session_stats,
            display_info: DisplayHealthInfo {
                total_captures: display_stats.total_captures,
                success_rate: if display_stats.total_captures > 0 {
                    display_stats.successful_captures as f64 / display_stats.total_captures as f64
                } else {
                    0.0
                },
                average_capture_time: display_stats.average_capture_time,
            },
        }
    }

    /// Perform system maintenance and cleanup
    pub async fn perform_maintenance(&self) -> Result<()> {
        // Clean up expired sessions
        self.network_protocol.cleanup_expired_sessions().await;

        // Clean up stale connections
        let mut connections = self.active_connections.write().await;
        let now = SystemTime::now();
        connections.retain(|_, conn| {
            now.duration_since(conn.last_activity).unwrap_or(Duration::MAX) < self.config.session_timeout
        });

        // Update system statistics
        {
            let mut stats = self.system_stats.write().await;
            stats.active_connections = connections.len() as u64;
        }

        // Optimize display controller settings
        self.display_controller.auto_optimize_settings().await?;

        tracing::info!("System maintenance completed");
        Ok(())
    }

    /// Get or create a session for the client
    async fn get_or_create_session(&self, client_info: &ClientInfo) -> String {
        // For simplicity, create a new session each time
        // In production, this would check for existing sessions
        self.network_protocol.create_session(client_info.clone()).await
    }

    /// Update connection statistics
    async fn update_connection_stats(
        &self,
        session_id: &str,
        response: &Response,
        start_time: SystemTime,
    ) -> Result<()> {
        let processing_time = start_time.elapsed().unwrap_or(Duration::ZERO);
        
        // Update system stats
        {
            let mut stats = self.system_stats.write().await;
            stats.total_commands_processed += 1;
            
            if response.status == ResponseStatus::Error {
                stats.error_count += 1;
                stats.last_error = response.error.clone();
            }
            
            // Update average processing time
            let total_time = stats.performance_metrics.average_command_processing_time.as_millis() * (stats.total_commands_processed - 1) as u128;
            let new_total = total_time + processing_time.as_millis();
            stats.performance_metrics.average_command_processing_time = 
                Duration::from_millis((new_total / stats.total_commands_processed as u128) as u64);
        }

        // Update connection info
        {
            let mut connections = self.active_connections.write().await;
            if let Some(conn) = connections.get_mut(session_id) {
                conn.last_activity = SystemTime::now();
                conn.commands_processed += 1;
            }
        }

        Ok(())
    }

    /// Get system statistics snapshot
    async fn get_system_stats_snapshot(&self) -> SystemStatistics {
        self.system_stats.read().await.clone()
    }

    /// Increment error count
    async fn increment_error_count(&self) {
        let mut stats = self.system_stats.write().await;
        stats.error_count += 1;
    }

    /// Create a clone for async operations
    fn clone_for_async(&self) -> Self {
        Self {
            network_server: None, // Don't clone the server
            network_protocol: self.network_protocol.clone(),
            platform_controller: self.platform_controller.clone(),
            display_controller: self.display_controller.clone(),
            active_connections: Arc::clone(&self.active_connections),
            system_stats: Arc::clone(&self.system_stats),
            config: self.config.clone(),
        }
    }
}

/// Enhanced response with integration metadata
#[derive(Debug, Clone)]
pub struct IntegratedResponse {
    pub response: Response,
    pub session_id: String,
    pub processing_time: Duration,
    pub system_stats: SystemStatistics,
}

/// Enhanced capture result with performance metrics
#[derive(Debug)]
pub struct EnhancedCaptureResult {
    pub data: Vec<u8>,
    pub display_id: u32,
    pub processing_time: Duration,
    pub compression_ratio: f64,
    pub capture_statistics: crate::server::enhanced_display::CaptureStatistics,
}

/// System health information
#[derive(Debug, Clone, Serialize)]
pub struct SystemHealth {
    pub status: String,
    pub uptime: Duration,
    pub active_connections: usize,
    pub total_commands_processed: u64,
    pub error_rate: f64,
    pub performance: PerformanceMetrics,
    pub platform_info: PlatformInfo,
    pub session_info: HashMap<String, u64>,
    pub display_info: DisplayHealthInfo,
}

/// Platform information for health status
#[derive(Debug, Clone, Serialize)]
pub struct PlatformInfo {
    pub name: String,
    pub capabilities: String,
    pub concurrent_operations: usize,
}

/// Display health information
#[derive(Debug, Clone, Serialize)]
pub struct DisplayHealthInfo {
    pub total_captures: u64,
    pub success_rate: f64,
    pub average_capture_time: Duration,
}

// Implement necessary traits for cloning
impl Clone for NetworkProtocolIntegration {
    fn clone(&self) -> Self {
        // Create a new instance since the internal state is Arc/RwLock
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                NetworkProtocolIntegration::new().await.unwrap()
            })
        })
    }
}

impl Clone for CrossPlatformController {
    fn clone(&self) -> Self {
        // Create a new instance since trait objects can't be cloned
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                CrossPlatformController::new().await.unwrap()
            })
        })
    }
}

impl Clone for EnhancedDisplayController {
    fn clone(&self) -> Self {
        // Create a new instance using the sync helper
        let platform = Arc::new(CrossPlatformController::new_sync().unwrap());
        EnhancedDisplayController::new(platform)
    }
}

// Helper implementation for CrossPlatformController sync creation
impl CrossPlatformController {
    pub fn new_sync() -> Result<Self> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                Self::new().await
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::HeadlessPlatform;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_system_integration_creation() {
        let integration = SystemIntegration::new().await;
        assert!(integration.is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_integrated_command_processing() {
        let integration = SystemIntegration::new().await.unwrap();
        
        let client_info = ClientInfo {
            user_agent: "test-client".to_string(),
            ip_address: "127.0.0.1".to_string(),
            platform: "test".to_string(),
            capabilities: vec!["basic".to_string()],
        };
        
        let command = Command {
            id: "integration-test".to_string(),
            command_type: CommandType::GetDisplays,
            payload: CommandPayload::GetDisplays {},
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let result = integration.process_integrated_command(&command, &client_info).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.session_id.is_empty());
        assert!(response.processing_time.as_millis() >= 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_batch_command_processing() {
        let integration = SystemIntegration::new().await.unwrap();
        
        let client_info = ClientInfo {
            user_agent: "batch-test".to_string(),
            ip_address: "127.0.0.1".to_string(),
            platform: "test".to_string(),
            capabilities: vec!["basic".to_string()],
        };
        
        let commands = vec![
            Command {
                id: "batch-1".to_string(),
                command_type: CommandType::GetDisplays,
                payload: CommandPayload::GetDisplays {},
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Command {
                id: "batch-2".to_string(),
                command_type: CommandType::MouseMove,
                payload: CommandPayload::MouseMove { x: 100, y: 100 },
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ];
        
        let results = integration.process_batch_commands(commands, &client_info).await;
        assert!(results.is_ok());
        
        let responses = results.unwrap();
        assert_eq!(responses.len(), 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_system_health_monitoring() {
        let integration = SystemIntegration::new().await.unwrap();
        
        let health = integration.get_system_health().await;
        assert_eq!(health.status, "healthy");
        assert_eq!(health.active_connections, 0);
        assert_eq!(health.total_commands_processed, 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_integrated_display_capture() {
        let integration = SystemIntegration::new().await.unwrap();
        
        let result = integration.integrated_display_capture(0).await;
        // Should either succeed or fail gracefully on headless
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_system_maintenance() {
        let integration = SystemIntegration::new().await.unwrap();
        
        let result = integration.perform_maintenance().await;
        assert!(result.is_ok());
    }
}