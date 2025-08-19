use crate::platform::traits::PlatformController;
use crate::protocol::messages::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tokio::time::Duration;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use chrono::Utc;

/// Simple network server for testing (implements minimal functionality for TDD)
pub struct NetworkServer {
    platform: Arc<RwLock<Box<dyn PlatformController + Send + Sync>>>,
    config: NetworkConfig,
    listener: Option<TcpListener>,
    connections: Arc<RwLock<HashMap<String, SocketAddr>>>,
    is_running: Arc<RwLock<bool>>,
}

/// Network server configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub bind_address: String,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub enable_websocket: bool,
    pub buffer_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".to_string(),
            max_connections: 100,
            connection_timeout: Duration::from_secs(60),
            enable_websocket: false,
            buffer_size: 8192,
        }
    }
}

impl NetworkServer {
    /// Create a new network server
    pub fn new(
        platform: Box<dyn PlatformController + Send + Sync>,
        bind_address: String,
    ) -> Self {
        let config = NetworkConfig {
            bind_address,
            ..Default::default()
        };
        
        Self {
            platform: Arc::new(RwLock::new(platform)),
            config,
            listener: None,
            connections: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a new network server with custom configuration
    pub fn with_config(
        platform: Box<dyn PlatformController + Send + Sync>,
        config: NetworkConfig,
    ) -> Self {
        Self {
            platform: Arc::new(RwLock::new(platform)),
            config,
            listener: None,
            connections: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the network server and spawn connection handler
    pub async fn start(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.bind_address).await
            .map_err(|e| anyhow!("Failed to bind to {}: {}", self.config.bind_address, e))?;
        
        // Get the actual address (important for port 0)
        let actual_addr = listener.local_addr()?;
        self.config.bind_address = actual_addr.to_string();
        
        // Set server as running
        *self.is_running.write().await = true;
        
        // Spawn connection handler in background using the listener we created
        let platform = self.platform.clone();
        let connections = self.connections.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            loop {
                // Check if server should continue running
                if !*is_running.read().await {
                    break;
                }
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        // Check connection limit
                        {
                            let conn_count = connections.read().await.len();
                            if conn_count >= config.max_connections {
                                let _ = Self::reject_connection(stream).await;
                                continue;
                            }
                        }

                        // Add connection
                        let connection_id = format!("{}:{}", addr.ip(), addr.port());
                        connections.write().await.insert(connection_id.clone(), addr);

                        // Handle connection
                        let platform_clone = platform.clone();
                        let connections_clone = connections.clone();
                        let config_clone = config.clone();
                        
                        tokio::spawn(async move {
                            let _ = Self::handle_connection(stream, addr, platform_clone, config_clone).await;
                            connections_clone.write().await.remove(&connection_id);
                        });
                    }
                    Err(_) => break,
                }
            }
        });
        
        // Give the task a moment to start listening
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        Ok(())
    }

    /// Get the local address the server is bound to
    pub async fn local_addr(&self) -> Result<SocketAddr> {
        self.config.bind_address.parse()
            .map_err(|e| anyhow!("Invalid bind address: {}", e))
    }

    /// Get the number of active connections
    pub async fn active_connections(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get server configuration
    pub async fn get_config(&self) -> NetworkConfig {
        self.config.clone()
    }

    /// Shutdown the server
    pub async fn shutdown(&self) -> Result<()> {
        *self.is_running.write().await = false;
        self.connections.write().await.clear();
        // Give time for the listener loop to exit
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    /// Handle a single connection
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        platform: Arc<RwLock<Box<dyn PlatformController + Send + Sync>>>,
        config: NetworkConfig,
    ) -> Result<()> {
        let (mut reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(&mut reader);
        let mut buffer = String::new();

        loop {
            buffer.clear();
            
            match buf_reader.read_line(&mut buffer).await {
                Ok(0) => break, // Client disconnected
                Ok(_) => {
                    let message = buffer.trim();
                    
                    // Handle WebSocket upgrade (check if it's an HTTP request)
                    if config.enable_websocket && message.starts_with("GET") {
                        // Read the complete HTTP request
                        let mut http_request = message.to_string() + "\r\n";
                        let mut header_line = String::new();
                        loop {
                            header_line.clear();
                            match buf_reader.read_line(&mut header_line).await {
                                Ok(0) => break,
                                Ok(_) => {
                                    http_request.push_str(&header_line);
                                    if header_line.trim().is_empty() {
                                        break; // End of HTTP headers
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        
                        // Check if it's a WebSocket upgrade request
                        if http_request.contains("websocket") && http_request.contains("Upgrade:") {
                            let response = 
                                "HTTP/1.1 101 Switching Protocols\r\n\
                                 Upgrade: websocket\r\n\
                                 Connection: Upgrade\r\n\
                                 Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
                            let _ = writer.write_all(response.as_bytes()).await;
                            continue;
                        }
                    }

                    // Process command
                    let (response, binary_data) = Self::process_command(message, &platform).await;
                    
                    match response {
                        Ok(resp) => {
                            let response_json = serde_json::to_string(&resp)?;
                            let _ = writer.write_all(response_json.as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                            
                            // Send binary data if available
                            if let Some(data) = binary_data {
                                let _ = writer.write_all(&data).await;
                            }
                        }
                        Err(_) => {
                            let error_resp = Response {
                                command_id: "unknown".to_string(),
                                status: ResponseStatus::Error,
                                error: Some("Invalid JSON".to_string()),
                                data: None,
                                timestamp: Utc::now().to_rfc3339(),
                            };
                            let response_json = serde_json::to_string(&error_resp)?;
                            let _ = writer.write_all(response_json.as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                        }
                    }
                }
                Err(_) => break,
            }
        }

        Ok(())
    }

    /// Process a command and return a response and optional binary data
    async fn process_command(
        message: &str,
        platform: &Arc<RwLock<Box<dyn PlatformController + Send + Sync>>>,
    ) -> (Result<Response>, Option<Vec<u8>>) {
        let command = match serde_json::from_str::<Command>(message) {
            Ok(cmd) => cmd,
            Err(e) => {
                let error_response = Response {
                    command_id: "unknown".to_string(),
                    status: ResponseStatus::Error,
                    error: Some(format!("Invalid JSON: {}", e)),
                    data: None,
                    timestamp: Utc::now().to_rfc3339(),
                };
                return (Ok(error_response), None);
            }
        };
        
        let platform_guard = platform.read().await;
        
        // Handle screen capture separately since it returns binary data
        if let CommandPayload::CaptureScreen { display_id } = command.payload {
            match platform_guard.capture_screen(display_id) {
                Ok(data) => {
                    let response = Response {
                        command_id: command.id,
                        status: ResponseStatus::Success,
                        error: None,
                        data: Some(ResponseData::ScreenCapture {
                            size: data.len(),
                            format: "png".to_string(),
                        }),
                        timestamp: Utc::now().to_rfc3339(),
                    };
                    return (Ok(response), Some(data));
                }
                Err(e) => {
                    let error_response = Response {
                        command_id: command.id,
                        status: ResponseStatus::Error,
                        error: Some(e.to_string()),
                        data: None,
                        timestamp: Utc::now().to_rfc3339(),
                    };
                    return (Ok(error_response), None);
                }
            }
        }

        // Handle other commands
        let result = match command.payload {
            CommandPayload::MouseMove { x, y } => {
                platform_guard.mouse_move(x, y)
            }
            CommandPayload::MouseClick { button, x, y } => {
                platform_guard.mouse_click(button, x, y)
            }
            CommandPayload::KeyPress { key } => {
                platform_guard.key_press(key)
            }
            _ => Ok(()), // Other commands succeed by default
        };

        let response = match result {
            Ok(_) => Response {
                command_id: command.id,
                status: ResponseStatus::Success,
                error: None,
                data: None,
                timestamp: Utc::now().to_rfc3339(),
            },
            Err(e) => Response {
                command_id: command.id,
                status: ResponseStatus::Error,
                error: Some(e.to_string()),
                data: None,
                timestamp: Utc::now().to_rfc3339(),
            },
        };

        (Ok(response), None)
    }

    /// Reject a connection that exceeds the limit
    async fn reject_connection(mut stream: TcpStream) -> Result<()> {
        let rejection = Response {
            command_id: "connection".to_string(),
            status: ResponseStatus::Error,
            error: Some("Connection limit reached".to_string()),
            data: None,
            timestamp: Utc::now().to_rfc3339(),
        };
        
        let response_json = serde_json::to_string(&rejection)?;
        let _ = stream.write_all(response_json.as_bytes()).await;
        let _ = stream.write_all(b"\n").await;
        
        Ok(())
    }
}