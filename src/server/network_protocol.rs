use crate::platform::traits::*;
use crate::protocol::messages::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Enhanced network protocol integration for cross-platform support
pub struct NetworkProtocolIntegration {
    command_handlers: Arc<RwLock<HashMap<String, CommandHandler>>>,
    session_manager: Arc<RwLock<SessionManager>>,
    rate_limiter: Arc<RwLock<NetworkRateLimiter>>,
}

/// Command handler function type
pub type CommandHandler = Box<dyn Fn(&Command, Arc<dyn PlatformController + Send + Sync>) -> Result<Response> + Send + Sync>;

/// Session management for network connections
#[derive(Debug)]
pub struct SessionManager {
    active_sessions: HashMap<String, NetworkSession>,
    session_timeout: Duration,
}

/// Network session information
#[derive(Debug, Clone)]
pub struct NetworkSession {
    pub session_id: String,
    pub client_info: ClientInfo,
    pub permissions: Vec<String>,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub command_count: u64,
    pub authenticated: bool,
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub user_agent: String,
    pub ip_address: String,
    pub platform: String,
    pub capabilities: Vec<String>,
}

/// Network rate limiter
#[derive(Debug)]
pub struct NetworkRateLimiter {
    limits: HashMap<String, RateLimit>,
    default_limit: RateLimit,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub max_requests: u64,
    pub window_duration: Duration,
    pub current_count: u64,
    pub window_start: SystemTime,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_duration: Duration::from_secs(60),
            current_count: 0,
            window_start: SystemTime::now(),
        }
    }
}

impl NetworkProtocolIntegration {
    /// Create a new network protocol integration
    pub async fn new() -> Result<Self> {
        Ok(Self {
            command_handlers: Arc::new(RwLock::new(HashMap::new())),
            session_manager: Arc::new(RwLock::new(SessionManager::new())),
            rate_limiter: Arc::new(RwLock::new(NetworkRateLimiter::new())),
        })
    }

    /// Register a command handler
    pub async fn register_command_handler(&self, command_type: String, handler: CommandHandler) {
        self.command_handlers.write().await.insert(command_type, handler);
    }

    /// Process a command with full integration support
    pub async fn process_command_integrated(
        &self,
        command: &Command,
        platform: Arc<dyn PlatformController + Send + Sync>,
        session_id: &str,
    ) -> Result<Response> {
        // Check rate limiting
        {
            let mut limiter = self.rate_limiter.write().await;
            if !limiter.check_rate_limit(session_id) {
                return Ok(Response {
                    command_id: command.id.clone(),
                    status: ResponseStatus::Error,
                    error: Some("Rate limit exceeded".to_string()),
                    data: None,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
            }
        }

        // Check session and permissions
        let has_permission = {
            let mut session_manager = self.session_manager.write().await;
            if let Some(session) = session_manager.active_sessions.get_mut(session_id) {
                session.last_activity = SystemTime::now();
                session.command_count += 1;
                
                // Check if session is authenticated for protected commands
                if self.requires_authentication(&command.command_type) && !session.authenticated {
                    return Ok(Response {
                        command_id: command.id.clone(),
                        status: ResponseStatus::Error,
                        error: Some("Authentication required".to_string()),
                        data: None,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
                
                self.check_command_permission(&command.command_type, &session.permissions)
            } else {
                false
            }
        };

        if !has_permission {
            return Ok(Response {
                command_id: command.id.clone(),
                status: ResponseStatus::Error,
                error: Some("Insufficient permissions".to_string()),
                data: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }

        // Execute the command
        self.execute_command_with_handlers(command, platform).await
    }

    /// Create a new session
    pub async fn create_session(&self, client_info: ClientInfo) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = NetworkSession {
            session_id: session_id.clone(),
            client_info,
            permissions: vec!["basic".to_string()], // Default permissions
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            command_count: 0,
            authenticated: false,
        };
        
        self.session_manager.write().await.active_sessions.insert(session_id.clone(), session);
        session_id
    }

    /// Authenticate a session (simplified for testing)
    pub async fn authenticate_session(&self, session_id: &str, _token: &str) -> Result<bool> {
        if let Some(session) = self.session_manager.write().await.active_sessions.get_mut(session_id) {
            session.authenticated = true;
            session.permissions.extend(vec![
                "mouse_control".to_string(),
                "keyboard_control".to_string(),
                "screen_capture".to_string(),
                "window_management".to_string(),
            ]);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Execute command with registered handlers or default implementation
    async fn execute_command_with_handlers(
        &self,
        command: &Command,
        platform: Arc<dyn PlatformController + Send + Sync>,
    ) -> Result<Response> {
        // Check for custom handlers first
        {
            let handlers = self.command_handlers.read().await;
            let command_type_str = format!("{:?}", command.command_type);
            if let Some(handler) = handlers.get(&command_type_str) {
                return handler(command, platform);
            }
        }

        // Default command execution
        execute_command_on_platform(platform, command).await
    }

    /// Check if command requires authentication
    fn requires_authentication(&self, command_type: &CommandType) -> bool {
        matches!(command_type, 
            CommandType::MouseMove | CommandType::MouseClick | 
            CommandType::KeyPress | CommandType::TypeText |
            CommandType::CaptureScreen)
    }

    /// Check if session has permission for command
    fn check_command_permission(&self, command_type: &CommandType, permissions: &[String]) -> bool {
        match command_type {
            CommandType::MouseMove | CommandType::MouseClick => {
                permissions.contains(&"mouse_control".to_string()) || permissions.contains(&"admin".to_string())
            }
            CommandType::KeyPress | CommandType::TypeText => {
                permissions.contains(&"keyboard_control".to_string()) || permissions.contains(&"admin".to_string())
            }
            CommandType::CaptureScreen => {
                permissions.contains(&"screen_capture".to_string()) || permissions.contains(&"admin".to_string())
            }
            CommandType::GetDisplays | CommandType::ListWindows => {
                permissions.contains(&"basic".to_string()) || permissions.contains(&"admin".to_string())
            }
            _ => permissions.contains(&"admin".to_string()),
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let mut session_manager = self.session_manager.write().await;
        let now = SystemTime::now();
        let timeout = session_manager.session_timeout;
        
        session_manager.active_sessions.retain(|_, session| {
            now.duration_since(session.last_activity)
                .unwrap_or(Duration::MAX) < timeout
        });
    }

    /// Get session statistics
    pub async fn get_session_stats(&self) -> HashMap<String, u64> {
        let session_manager = self.session_manager.read().await;
        let mut stats = HashMap::new();
        
        stats.insert("active_sessions".to_string(), session_manager.active_sessions.len() as u64);
        stats.insert("total_commands".to_string(), 
            session_manager.active_sessions.values().map(|s| s.command_count).sum());
        
        let authenticated_count = session_manager.active_sessions.values()
            .filter(|s| s.authenticated).count() as u64;
        stats.insert("authenticated_sessions".to_string(), authenticated_count);
        
        stats
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
            session_timeout: Duration::from_secs(3600), // 1 hour default
        }
    }
}

impl NetworkRateLimiter {
    pub fn new() -> Self {
        Self {
            limits: HashMap::new(),
            default_limit: RateLimit::default(),
        }
    }

    pub fn check_rate_limit(&mut self, session_id: &str) -> bool {
        let limit = self.limits.entry(session_id.to_string())
            .or_insert_with(|| self.default_limit.clone());
        
        let now = SystemTime::now();
        
        // Reset window if expired
        if now.duration_since(limit.window_start).unwrap_or(Duration::ZERO) >= limit.window_duration {
            limit.current_count = 0;
            limit.window_start = now;
        }
        
        // Check limit
        if limit.current_count >= limit.max_requests {
            false
        } else {
            limit.current_count += 1;
            true
        }
    }

    pub fn set_custom_limit(&mut self, session_id: String, limit: RateLimit) {
        self.limits.insert(session_id, limit);
    }
}

/// Execute a command on a platform (used by tests and network integration)
pub async fn execute_command_on_platform(
    platform: Arc<dyn PlatformController + Send + Sync>,
    command: &Command,
) -> Result<Response> {
    let result = match &command.payload {
        CommandPayload::MouseMove { x, y } => {
            platform.mouse_move(*x, *y)
        }
        CommandPayload::MouseClick { button, x, y } => {
            platform.mouse_click(button.clone(), *x, *y)
        }
        CommandPayload::KeyPress { key } => {
            platform.key_press(key.clone())
        }
        CommandPayload::TypeText { text } => {
            platform.type_text(text.clone())
        }
        CommandPayload::CaptureScreen { display_id } => {
            match platform.capture_screen(*display_id) {
                Ok(data) => {
                    return Ok(Response {
                        command_id: command.id.clone(),
                        status: ResponseStatus::Success,
                        error: None,
                        data: Some(ResponseData::ScreenCapture {
                            size: data.len(),
                            format: "png".to_string(),
                        }),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
                Err(e) => Err(e),
            }
        }
        CommandPayload::GetDisplays {} => {
            match platform.get_displays() {
                Ok(displays) => {
                    return Ok(Response {
                        command_id: command.id.clone(),
                        status: ResponseStatus::Success,
                        error: None,
                        data: Some(ResponseData::DisplayInfo { 
                            displays: displays.into_iter().map(|d| DisplayData {
                                id: d.id,
                                name: d.name,
                                width: d.width,
                                height: d.height,
                                x: d.x,
                                y: d.y,
                                is_primary: d.is_primary,
                            }).collect()
                        }),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
                Err(e) => Err(e),
            }
        }
        CommandPayload::ListWindows {} => {
            match platform.list_windows() {
                Ok(windows) => {
                    return Ok(Response {
                        command_id: command.id.clone(),
                        status: ResponseStatus::Success,
                        error: None,
                        data: Some(ResponseData::WindowInfo { 
                            windows: windows.into_iter().map(|w| WindowData {
                                id: w.id,
                                title: w.title,
                                x: w.x,
                                y: w.y,
                                width: w.width,
                                height: w.height,
                                process_name: w.process_name,
                            }).collect()
                        }),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
                Err(e) => Err(e),
            }
        }
        _ => Ok(()), // Unknown commands succeed silently
    };

    let response = match result {
        Ok(_) => Response {
            command_id: command.id.clone(),
            status: ResponseStatus::Success,
            error: None,
            data: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        Err(e) => Response {
            command_id: command.id.clone(),
            status: ResponseStatus::Error,
            error: Some(e.to_string()),
            data: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    };

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::HeadlessPlatform;

    #[tokio::test]
    async fn test_network_protocol_integration_creation() {
        let integration = NetworkProtocolIntegration::new().await;
        assert!(integration.is_ok());
    }

    #[tokio::test]
    async fn test_session_management() {
        let integration = NetworkProtocolIntegration::new().await.unwrap();
        
        let client_info = ClientInfo {
            user_agent: "test-client".to_string(),
            ip_address: "127.0.0.1".to_string(),
            platform: "linux".to_string(),
            capabilities: vec!["basic".to_string()],
        };
        
        let session_id = integration.create_session(client_info).await;
        assert!(!session_id.is_empty());
        
        let stats = integration.get_session_stats().await;
        assert_eq!(stats.get("active_sessions"), Some(&1));
    }

    #[tokio::test]
    async fn test_command_execution() {
        let platform: Arc<dyn PlatformController + Send + Sync> = Arc::new(HeadlessPlatform::new());
        
        let command = Command {
            id: "test-1".to_string(),
            command_type: CommandType::MouseMove,
            payload: CommandPayload::MouseMove { x: 100, y: 100 },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let result = execute_command_on_platform(platform, &command).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mut limiter = NetworkRateLimiter::new();
        
        // Set a very low limit for testing
        limiter.set_custom_limit("test-session".to_string(), RateLimit {
            max_requests: 2,
            window_duration: Duration::from_secs(60),
            current_count: 0,
            window_start: SystemTime::now(),
        });
        
        // First two requests should succeed
        assert!(limiter.check_rate_limit("test-session"));
        assert!(limiter.check_rate_limit("test-session"));
        
        // Third request should fail
        assert!(!limiter.check_rate_limit("test-session"));
    }

    #[tokio::test]
    async fn test_authentication_flow() {
        let integration = NetworkProtocolIntegration::new().await.unwrap();
        
        let client_info = ClientInfo {
            user_agent: "test-client".to_string(),
            ip_address: "127.0.0.1".to_string(),
            platform: "linux".to_string(),
            capabilities: vec!["basic".to_string()],
        };
        
        let session_id = integration.create_session(client_info).await;
        
        let auth_result = integration.authenticate_session(&session_id, "mock-token").await;
        assert!(auth_result.is_ok());
        assert!(auth_result.unwrap());
    }

    #[tokio::test]
    async fn test_permission_checking() {
        let integration = NetworkProtocolIntegration::new().await.unwrap();
        
        // Test that basic permissions don't allow mouse control
        assert!(!integration.check_command_permission(
            &CommandType::MouseMove, 
            &["basic".to_string()]
        ));
        
        // Test that mouse_control permission allows mouse commands
        assert!(integration.check_command_permission(
            &CommandType::MouseMove, 
            &["mouse_control".to_string()]
        ));
        
        // Test that admin permission allows everything
        assert!(integration.check_command_permission(
            &CommandType::MouseMove, 
            &["admin".to_string()]
        ));
    }
}